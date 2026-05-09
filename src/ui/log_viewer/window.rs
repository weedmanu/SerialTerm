//! ############################################################################
//! @file    `log_viewer/window.rs`
//! @author  manu
//! @brief   Fenêtre de visualisation des logs : filtrage par niveau, tri,
//!          recherche textuelle, copie et export CSV/TXT.
//!          Point d'entrée public : [`open_log_viewer`].
//! @version    1.0.0
//! @date    2026-03-05
//! @copyright GPL-3.0-or-later.
//! ############################################################################
//!
//! ## Architecture de la fenêtre
//!
//! ```text
//! ┌─ libadwaita::Window ─────────────────────────────────────────────────────┐
//! │  ┌─ ToolbarView ──────────────────────────────────────────────────────┐  │
//! │  │  ┌─ HeaderBar ─────────────────────────────────────────────────┐   │  │
//! │  │  │  [SearchEntry (titre)]  [copy] [export] [refresh]           │   │  │
//! │  │  └─────────────────────────────────────────────────────────────┘   │  │
//! │  │  ┌─ content (GtkBox vertical) ─────────────────────────────────┐   │  │
//! │  │  │  ┌─ filter_bar ─────────────────────────────────────────────┐│  │  │
//! │  │  │  │  Niveau : [ERR][WARN][INFO][DBG][SYS][ -- ]  Tout  Aucun ││  │  │
//! │  │  │  │  [↑↓ tri]                          [N / M lignes — X err]││  │  │
//! │  │  │  └─────────────────────────────────────────────────────────┘│  │  │
//! │  │  │  ┌─ ScrolledWindow → ListView ──────────────────────────────┐│  │  │
//! │  │  │  │  [N°][BADGE][texte de la ligne…]                         ││  │  │
//! │  │  │  └─────────────────────────────────────────────────────────┘│  │  │
//! │  │  └─────────────────────────────────────────────────────────────┘   │  │
//! │  │  ┌─ status_bar ────────────────────────────────────────────────┐   │  │
//! │  │  │  Clic : sélectionner · Triple-clic · Ctrl+C : copier        │   │  │
//! │  │  └─────────────────────────────────────────────────────────────┘   │  │
//! │  └────────────────────────────────────────────────────────────────┘   │  │
//! └──────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Pipeline de données GTK
//!
//! ```text
//! StringList  →  FilterListModel (CustomFilter)
//!             →  SortListModel   (CustomSorter)
//!             →  SingleSelection
//!             →  ListView        (SignalListItemFactory)
//! ```

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{glib, Box as GtkBox, Button, Label, Orientation, Separator, StringObject};
use libadwaita::prelude::*;

use super::model::{decode, encode_line, LogLevel};
use crate::ui::i18n::UiLang;

fn normalize_system_log_viewer_line(text: &str) -> &str {
    let Some(rest) = text.strip_prefix('[') else {
        return text;
    };

    let Some((prefix, suffix)) = rest.split_once("] ") else {
        return text;
    };

    if prefix.len() != 8 {
        return text;
    }

    let mut chars = prefix.chars();
    let is_short_timestamp = matches!(chars.next(), Some(a) if a.is_ascii_digit())
        && matches!(chars.next(), Some(b) if b.is_ascii_digit())
        && matches!(chars.next(), Some(':'))
        && matches!(chars.next(), Some(c) if c.is_ascii_digit())
        && matches!(chars.next(), Some(d) if d.is_ascii_digit())
        && matches!(chars.next(), Some(':'))
        && matches!(chars.next(), Some(e) if e.is_ascii_digit())
        && matches!(chars.next(), Some(f) if f.is_ascii_digit())
        && chars.next().is_none();

    if is_short_timestamp {
        suffix
    } else {
        text
    }
}

fn exportable_log_viewer_line(level_code: char, text: &str) -> String {
    let rendered = if LogLevel::from_code(level_code) == LogLevel::System {
        normalize_system_log_viewer_line(text)
    } else {
        text
    };

    rendered.to_string()
}

// =============================================================================
// Point d'entrée public
// =============================================================================

/// Ouvre la fenêtre de visualisation des logs.
///
/// La fenêtre est **non-modale** (l'utilisateur peut continuer à utiliser
/// le terminal pendant que la fenêtre est ouverte).
///
/// # Paramètres
/// - `parent`   : fenêtre parente pour le positionnement et la transience.
/// - `get_logs` : callback retournant le texte courant du terminal.
///   Appelé à l'ouverture et à chaque clic sur « Rafraîchir ».
#[allow(
    clippy::too_many_lines,
    clippy::clone_on_ref_ptr,
    clippy::redundant_clone,       // clones de Rc<GObject> nécessaires pour plusieurs closures GTK
    clippy::needless_pass_by_value // Rc déplacé dans les closures, passer par valeur est correct
)]
pub fn open_log_viewer(
    parent: &impl IsA<gtk4::Window>,
    lang: UiLang,
    get_logs: Rc<dyn Fn() -> String>,
) {
    // ── Parse initial du contenu terminal ─────────────────────────────────────
    // On commence par analyser le texte courant pour construire la StringList initiale.
    let parsed = parse_logs(&get_logs()); // analyse complète : détection niveaux + encodage

    // Compteurs partagés pour la barre de statut (mis à jour à chaque rafraîchissement)
    let err_count: Rc<Cell<u32>> = Rc::new(Cell::new(parsed.errors)); // nb d'erreurs détectées
    let warn_count: Rc<Cell<u32>> = Rc::new(Cell::new(parsed.warnings)); // nb d'avertissements
    let total_count: Rc<Cell<usize>> = Rc::new(Cell::new(parsed.encoded.len())); // nb total de lignes

    // Construction de la StringList GTK à partir des lignes encodées
    let string_list = gtk4::StringList::new(
        &parsed
            .encoded
            .iter()
            .map(String::as_str) // emprunt temporaire pour la construction
            .collect::<Vec<_>>(),
    );

    // ── Filtre de la ListView ─────────────────────────────────────────────────
    // Deux critères combinés par AND : masque de niveaux ET recherche textuelle.

    let search_str: Rc<RefCell<String>> = Rc::default(); // texte de recherche courant (vide = pas de filtre texte)
    let level_mask: Rc<Cell<u8>> = Rc::new(Cell::new(0b11_1111_u8)); // tous les niveaux actifs initialement

    // Clones pour la closure du filtre (closures GTK exigent 'static)
    let ss = search_str.clone();
    let lm = level_mask.clone();

    let custom_filter = gtk4::CustomFilter::new(move |obj| {
        // Récupération de l'objet StringObject de la liste
        let Some(so) = obj.downcast_ref::<StringObject>() else {
            return false; // type inattendu : caché
        };
        let encoded = so.string();
        let s = encoded.as_str();

        if s.is_empty() {
            return false; // ligne vide : jamais affichée
        }

        // Test 1 : le niveau de la ligne est-il dans le masque actif ?
        let code = s.chars().next().unwrap_or('N'); // premier char = code niveau
        if lm.get() & LogLevel::bit_for_code(code) == 0 {
            return false; // niveau filtré : ligne cachée
        }

        // Test 2 : le texte de recherche est-il présent dans la ligne ?
        let search = ss.borrow();
        if search.is_empty() {
            return true; // pas de filtre texte : toutes les lignes passent
        }

        // La partie texte commence à l'octet 8 (format "C|NNNNN|" = 8 chars)
        let text_part = s.get(8..).unwrap_or(s); // fallback sur la ligne entière si trop courte
        text_part
            .to_ascii_lowercase()
            .contains(search.to_ascii_lowercase().as_str()) // comparaison insensible à la casse
    });

    // Le FilterListModel applique le filtre à la StringList et réagit aux changements
    let filter_model =
        gtk4::FilterListModel::new(Some(string_list.clone()), Some(custom_filter.clone()));

    // ── Tri de la ListView ────────────────────────────────────────────────────
    // Tri par numéro de ligne (champ NNNNN aux positions 2..7 du format encodé).
    // Ordre configurable asc/desc via le bouton sort_btn.

    let sort_asc: Rc<Cell<bool>> = Rc::new(Cell::new(true)); // tri ascendant par défaut
    let sa = sort_asc.clone();

    let custom_sorter = gtk4::CustomSorter::new(move |a, b| {
        // Extraction du numéro de ligne depuis le format "C|NNNNN|texte"
        let ea = a
            .downcast_ref::<StringObject>()
            .map(StringObject::string)
            .unwrap_or_default();
        let eb = b
            .downcast_ref::<StringObject>()
            .map(StringObject::string)
            .unwrap_or_default();

        // Slice 2..7 = 5 chiffres du numéro de ligne
        let na = ea
            .get(2..7)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0); // numéro ligne a
        let nb = eb
            .get(2..7)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0); // numéro ligne b

        // Comparaison selon l'ordre courant (asc ou desc)
        if sa.get() { na.cmp(&nb) } else { nb.cmp(&na) }.into()
    });

    // Pipeline complet : filtre → tri → sélection unique
    let sort_model =
        gtk4::SortListModel::new(Some(filter_model.clone()), Some(custom_sorter.clone()));
    let selection = gtk4::SingleSelection::new(Some(sort_model.clone())); // sélection d'une seule ligne à la fois

    // ── Fenêtre principale ────────────────────────────────────────────────────
    let window = libadwaita::Window::builder()
        .transient_for(parent) // liée à la fenêtre parente (centrage, icône de barre de tâches)
        .modal(false) // non-modale : l'utilisateur peut continuer à travailler
        .title(lang.log_viewer_title())
        .default_width(1000) // largeur initiale en pixels
        .default_height(650) // hauteur initiale en pixels
        .build();

    // ── HeaderBar ─────────────────────────────────────────────────────────────
    let header = libadwaita::HeaderBar::new();

    // Champ de recherche dans le titre de la barre (centré)
    let search_entry = gtk4::SearchEntry::builder()
        .placeholder_text(lang.log_search_placeholder()) // texte indicatif quand vide
        .hexpand(true) // s'étend pour remplir le titre
        .build();
    header.set_title_widget(Some(&search_entry)); // positionné au centre de la HeaderBar

    // Boutons d'action dans la zone droite de la HeaderBar
    let refresh_btn = Button::builder()
        .icon_name("view-refresh-symbolic")
        .tooltip_text(lang.log_refresh_tooltip()) // tooltip affiché au survol
        .build();
    let export_btn = Button::builder()
        .icon_name("document-save-symbolic")
        .tooltip_text(lang.log_export_tooltip())
        .build();
    let copy_btn = Button::builder()
        .icon_name("edit-copy-symbolic")
        .tooltip_text(lang.log_copy_tooltip())
        .build();

    // Ajout dans l'ordre inverse (pack_end ajoute à gauche du précédent)
    header.pack_end(&refresh_btn); // le plus à droite
    header.pack_end(&export_btn);
    header.pack_end(&copy_btn); // le plus à gauche des trois boutons droits

    // ── Barre de filtres par niveau ───────────────────────────────────────────
    let filter_bar = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4) // espace entre les boutons
        .margin_start(8) // marges pour l'aération
        .margin_end(8)
        .margin_top(4)
        .margin_bottom(4)
        .build();
    filter_bar.add_css_class("toolbar"); // style barre d'outils GTK

    filter_bar.append(&Label::new(Some(lang.log_level_label()))); // étiquette fixe

    // Définition des boutons de filtre : (libellé, bit, tooltip)
    // Chaque bouton contrôle le bit correspondant dans `level_mask`.
    let tips = lang.log_level_tooltips();
    let level_defs = [
        ("ERR", 0b00_0001_u8, tips[0]),  // bit 0
        ("WARN", 0b00_0010_u8, tips[1]), // bit 1
        ("INFO", 0b00_0100_u8, tips[2]), // bit 2
        ("DBG", 0b00_1000_u8, tips[3]),  // bit 3
        ("SYS", 0b01_0000_u8, tips[4]),  // bit 4
        (" -- ", 0b10_0000_u8, tips[5]), // bit 5
    ];

    // Construction des ToggleButton de filtre par niveau
    let toggle_btns: Rc<Vec<gtk4::ToggleButton>> = Rc::new(
        level_defs
            .iter()
            .map(|(lbl, bit, tip)| {
                let btn = gtk4::ToggleButton::builder()
                    .label(*lbl)
                    .active(true) // tous activés par défaut (tous niveaux visibles)
                    .tooltip_text(*tip)
                    .build();
                btn.add_css_class("monospace"); // police fixe pour aligner les libellés

                // Clones pour la closure du signal
                let lm = level_mask.clone();
                let cf = custom_filter.clone();
                let b = *bit; // copie du bit (u8 = Copy)

                btn.connect_toggled(move |tb| {
                    let old = lm.get();
                    if tb.is_active() {
                        lm.set(old | b); // activation : ajoute le bit au masque
                    } else {
                        lm.set(old & !b); // désactivation : retire le bit du masque
                    }
                    cf.changed(gtk4::FilterChange::Different); // notifie la ListView du changement
                });

                filter_bar.append(&btn); // ajout du bouton dans la barre
                btn
            })
            .collect(),
    );

    filter_bar.append(&Separator::new(Orientation::Vertical)); // séparateur visuel

    // ── Boutons Tout / Aucun ──────────────────────────────────────────────────

    // Bouton "Tout" : active tous les niveaux simultanément
    let all_btn = Button::builder()
        .label(lang.log_all_label())
        .tooltip_text(lang.log_all_tooltip())
        .build();
    {
        let lm = level_mask.clone();
        let cf = custom_filter.clone();
        let tbs = toggle_btns.clone();
        all_btn.connect_clicked(move |_| {
            lm.set(0b11_1111); // tous les bits à 1
            cf.changed(gtk4::FilterChange::LessStrict); // filtre assoupli : peut afficher plus
            for t in tbs.iter() {
                t.set_active(true); // synchronisation visuelle de tous les ToggleButton
            }
        });
    }
    filter_bar.append(&all_btn);

    // Bouton "Aucun" : désactive tous les niveaux simultanément
    let none_btn = Button::builder()
        .label(lang.log_none_label())
        .tooltip_text(lang.log_none_tooltip())
        .build();
    {
        let lm = level_mask.clone();
        let cf = custom_filter.clone();
        let tbs = toggle_btns.clone();
        none_btn.connect_clicked(move |_| {
            lm.set(0); // tous les bits à 0
            cf.changed(gtk4::FilterChange::MoreStrict); // filtre resserré : cache tout
            for t in tbs.iter() {
                t.set_active(false); // synchronisation visuelle
            }
        });
    }
    filter_bar.append(&none_btn);
    filter_bar.append(&Separator::new(Orientation::Vertical)); // séparateur avant le bouton de tri

    // ── Bouton de tri asc/desc ────────────────────────────────────────────────
    let sort_btn = Button::builder()
        .icon_name("view-sort-ascending-symbolic") // icône initiale : tri ascendant
        .tooltip_text(lang.log_sort_tooltip())
        .build();
    {
        let sa = sort_asc.clone();
        let cs = custom_sorter.clone();
        let sb = sort_btn.clone(); // clone pour modifier l'icône depuis la closure
        sort_btn.connect_clicked(move |_| {
            let asc = !sa.get(); // inverse l'ordre courant
            sa.set(asc);
            cs.changed(gtk4::SorterChange::Different); // notifie la ListView du changement de tri
                                                       // Mise à jour de l'icône pour refléter l'ordre courant
            sb.set_icon_name(if asc {
                "view-sort-ascending-symbolic" // ordre croissant
            } else {
                "view-sort-descending-symbolic" // ordre décroissant
            });
        });
    }
    filter_bar.append(&sort_btn);

    // Compteur de lignes en fin de barre (aligné à droite grâce à hexpand)
    let count_lbl = Label::builder()
        .halign(gtk4::Align::End) // alignement à droite
        .hexpand(true) // pousse vers la droite en prenant tout l'espace disponible
        .build();
    count_lbl.add_css_class("dim-label"); // texte atténué (moins proéminent)
    filter_bar.append(&count_lbl);

    // ── Factory de la ListView (création et liaison des lignes) ───────────────
    // La SignalListItemFactory crée des widgets réutilisables (recycling pattern GTK4).
    // Chaque ligne est un GtkBox horizontal : [numéro][badge][texte].
    let factory = gtk4::SignalListItemFactory::new();

    // Signal setup : crée la structure du widget (appelé une fois par ligne visible)
    factory.connect_setup(|_, list_item| {
        let Some(item) = list_item.downcast_ref::<gtk4::ListItem>() else {
            return; // type inattendu : ignoré
        };

        // Conteneur horizontal de la ligne
        let row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8) // espace entre les colonnes
            .margin_start(4)
            .margin_end(4)
            .margin_top(1)
            .margin_bottom(1)
            .build();

        // Colonne [0] : numéro de ligne (6 chars, aligné à droite)
        let line_lbl = Label::builder()
            .width_chars(6) // largeur fixe pour aligner les numéros
            .xalign(1.0) // alignement à droite dans la colonne
            .build();
        line_lbl.add_css_class("dim-label"); // grisé (secondaire)
        line_lbl.add_css_class("monospace"); // police fixe

        // Colonne [1] : badge de niveau (6 chars, centré)
        let badge_lbl = Label::builder()
            .width_chars(6) // largeur fixe pour aligner les badges
            .xalign(0.5) // centré dans la colonne
            .build();
        badge_lbl.add_css_class("monospace"); // police fixe pour ERR/WARN/INFO/…

        // Colonne [2] : texte de la ligne (extensible, tronqué avec ellipse)
        let text_lbl = Label::builder()
            .hexpand(true) // prend tout l'espace horizontal restant
            .xalign(0.0) // aligné à gauche
            .wrap(false) // pas de retour à la ligne (ListView horizontale)
            .ellipsize(gtk4::pango::EllipsizeMode::End) // troncature en fin avec "…"
            .selectable(true) // l'utilisateur peut sélectionner le texte
            .build();
        text_lbl.add_css_class("monospace"); // police fixe pour l'alignement du terminal

        // Assemblage de la ligne : numéro → badge → texte
        row.append(&line_lbl);
        row.append(&badge_lbl);
        row.append(&text_lbl);
        item.set_child(Some(&row)); // liaison du widget à l'item de la liste
    });

    // Signal bind : remplit le widget avec les données de l'item courant
    // (appelé à chaque fois qu'un item est associé à un widget recyclé)
    factory.connect_bind(|_, list_item| {
        let Some(item) = list_item.downcast_ref::<gtk4::ListItem>() else {
            return; // type inattendu
        };

        // Récupération de la chaîne encodée depuis le modèle
        let encoded = item
            .item()
            .and_downcast::<StringObject>()
            .map(|o| o.string())
            .unwrap_or_default();

        // Décodage du format "C|NNNNN|texte"
        let (code, line_no, text) = decode(encoded.as_str());
        let level = LogLevel::from_code(code); // niveau associé au code
        let color = level.fg_color(); // couleur CSS du niveau
        let badge = level.badge(); // badge texte 4 chars

        // Navigation dans l'arborescence du widget pour accéder aux Labels
        let Some(row) = item.child().and_downcast::<GtkBox>() else {
            return; // widget non initialisé
        };
        let Some(first) = row.first_child() else {
            return; // line_lbl absent
        };
        let Ok(line_lbl) = first.downcast::<Label>() else {
            return; // type incorrect
        };
        let Some(second) = line_lbl.next_sibling() else {
            return; // badge_lbl absent
        };
        let Ok(badge_lbl) = second.downcast::<Label>() else {
            return;
        };
        let Some(third) = badge_lbl.next_sibling() else {
            return; // text_lbl absent
        };
        let Ok(text_lbl) = third.downcast::<Label>() else {
            return;
        };

        // Mise à jour du numéro de ligne (texte simple)
        line_lbl.set_label(line_no);

        // Badge coloré avec balisage Pango (gras + couleur du niveau)
        badge_lbl.set_markup(&format!("<span foreground='{color}'><b>{badge}</b></span>"));

        // Texte de la ligne avec échappement des caractères spéciaux Pango (<, >, &, etc.)
        let escaped = glib::markup_escape_text(text); // protection contre l'injection de balisage
        text_lbl.set_markup(&format!("<span foreground='{color}'>{escaped}</span>"));
    });

    // Signal unbind : nettoie le widget avant recyclage (évite les données fantômes)
    factory.connect_unbind(|_, list_item| {
        let Some(item) = list_item.downcast_ref::<gtk4::ListItem>() else {
            return;
        };
        let Some(row) = item.child().and_downcast::<GtkBox>() else {
            return;
        };

        // Parcours des enfants et remise à zéro de tous les Labels
        let mut child_opt = row.first_child();
        while let Some(child) = child_opt {
            let next = child.next_sibling(); // sauvegarde avant le downcast (consomme child)
            if let Ok(lbl) = child.downcast::<Label>() {
                lbl.set_label(""); // effacement du contenu
                child_opt = lbl.next_sibling();
            } else {
                child_opt = next; // enfant non-Label : passe au suivant
            }
        }
    });

    // Construction de la ListView
    let list_view = gtk4::ListView::builder()
        .model(&selection) // modèle : SingleSelection sur SortListModel
        .factory(&factory) // factory : SignalListItemFactory définie ci-dessus
        .vexpand(true) // s'étend verticalement
        .hexpand(true) // s'étend horizontalement
        .show_separators(true) // séparateurs visuels entre les lignes
        .build();
    list_view.add_css_class("rich-list"); // style Adwaita pour les listes riches

    // Enveloppe scrollable pour la ListView
    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Automatic) // scrollbar H si contenu trop large
        .vscrollbar_policy(gtk4::PolicyType::Automatic) // scrollbar V si contenu trop haut
        .vexpand(true)
        .child(&list_view)
        .build();

    // ── Barre de statut bas ────────────────────────────────────────────────────
    let status_bar = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .margin_start(10)
        .margin_end(10)
        .margin_top(3)
        .margin_bottom(3)
        .build();
    status_bar.add_css_class("toolbar"); // style barre d'outils

    let hint_lbl = Label::builder()
        .label(lang.log_status_hint())
        .halign(gtk4::Align::Start) // aligné à gauche
        .build();
    hint_lbl.add_css_class("dim-label"); // texte atténué (aide contextuelle discrète)
    status_bar.append(&hint_lbl);

    // ── Assemblage de la fenêtre ───────────────────────────────────────────────
    let content = GtkBox::builder().orientation(Orientation::Vertical).build();
    content.append(&filter_bar); // barre de filtres en haut
    content.append(&scrolled); // liste en dessous

    let toolbar_view = libadwaita::ToolbarView::new();
    toolbar_view.add_top_bar(&header); // HeaderBar en haut
    toolbar_view.set_content(Some(&content)); // zone centrale
    toolbar_view.add_bottom_bar(&status_bar); // barre de statut en bas
    window.set_content(Some(&toolbar_view));

    // ── Mise à jour du compteur de lignes ─────────────────────────────────────
    // Fonction locale (closure) qui formate le label de comptage.
    // Appelée une première fois pour l'état initial, puis à chaque changement du filtre.
    {
        let cl = count_lbl.clone();
        let ec = err_count.clone();
        let wc = warn_count.clone();
        let tc = total_count.clone();

        // Mise à jour initiale au lancement de la fenêtre
        let update = move |visible: u32| {
            cl.set_label(&lang.log_count_label(visible, tc.get(), ec.get(), wc.get()));
        };
        update(filter_model.n_items()); // affichage initial (avant tout filtrage)

        // Signal : items_changed déclenché à chaque modification du FilterListModel
        let cl2 = count_lbl.clone();
        let ec2 = err_count.clone();
        let wc2 = warn_count.clone();
        let tc2 = total_count.clone();
        filter_model.connect_items_changed(move |m, _, _, _| {
            cl2.set_label(&lang.log_count_label(m.n_items(), tc2.get(), ec2.get(), wc2.get()));
        });
    }

    // ── Signaux de la fenêtre ─────────────────────────────────────────────────

    // Signal recherche : mise à jour du texte de recherche et invalidation du filtre
    {
        let ss = search_str.clone();
        let cf = custom_filter.clone();
        search_entry.connect_search_changed(move |entry| {
            *ss.borrow_mut() = entry.text().to_string(); // mise à jour de la chaîne partagée
            cf.changed(gtk4::FilterChange::Different); // notifie le filtre (recalcule tout)
        });
    }

    // Raccourci Ctrl+F : ramène le focus sur le champ de recherche
    {
        let se = search_entry.clone();
        let ctrl = gtk4::EventControllerKey::new();
        ctrl.connect_key_pressed(move |_, key, _, mods| {
            if key == gtk4::gdk::Key::f && mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
                se.grab_focus(); // focus sur la SearchEntry
                return glib::Propagation::Stop; // consomme l'événement (empêche d'autres handlers)
            }
            glib::Propagation::Proceed // laisse passer les autres touches
        });
        window.add_controller(ctrl); // ajout du controller à la fenêtre
    }

    // Signal copier (bouton + futur Ctrl+C) : copie la ligne sélectionnée dans le presse-papiers
    {
        let sel = selection.clone();
        let win2 = window.clone();
        copy_btn.connect_clicked(move |_| {
            if let Some(obj) = sel.selected_item().and_downcast::<StringObject>() {
                let gstr = obj.string();
                let (level_code, line_no, text) = decode(gstr.as_str()); // décodage du format interne
                win2.clipboard().set_text(&format!(
                    "[{line_no}] {}",
                    exportable_log_viewer_line(level_code, text)
                ));
                // copie dans le presse-papiers
            }
            // Si aucune ligne sélectionnée : clic ignoré silencieusement
        });
    }

    // Signal rafraîchir : re-parse le terminal et reconstruit la StringList
    {
        let gl = get_logs.clone(); // callback vers le texte courant du terminal
        let sl = string_list.clone();
        let cf = custom_filter.clone();
        let ec = err_count.clone();
        let wc = warn_count.clone();
        let tc = total_count.clone();
        refresh_btn.connect_clicked(move |_| {
            let p = parse_logs(&gl()); // re-parse complet du terminal courant

            // Mise à jour des compteurs partagés
            ec.set(p.errors);
            wc.set(p.warnings);
            tc.set(p.encoded.len());

            // Remplacement atomique de tout le contenu de la StringList
            let strs: Vec<&str> = p.encoded.iter().map(String::as_str).collect();
            sl.splice(0, sl.n_items(), &strs); // splice(début, nb_supprimés, nouvelles_valeurs)

            cf.changed(gtk4::FilterChange::Different); // force un recalcul complet du filtre
        });
    }

    // Signal exporter : sauvegarde les lignes filtrées (dans l'ordre de tri courant) vers un fichier
    {
        let sm = sort_model.clone();
        let win2 = window.clone();
        export_btn.connect_clicked(move |_| {
            let n = sm.n_items(); // nombre de lignes filtrées et triées

            // Construction du contenu à exporter : format lisible "[NNNNN] texte"
            let lines: Vec<String> = (0..n)
                .filter_map(|i| {
                    sm.item(i).and_downcast::<StringObject>().map(|o| {
                        let gstr = o.string();
                        let (level_code, line_no, text) = decode(gstr.as_str());
                        format!(
                            "[{line_no}] {}",
                            exportable_log_viewer_line(level_code, text)
                        )
                        // format d'export lisible
                    })
                })
                .collect();

            let content_str = lines.join("\n"); // une ligne par ligne, séparées par \n

            // Dialogue de sélection du fichier de destination
            let fd = gtk4::FileDialog::builder()
                .title(lang.log_export_dialog_title())
                .initial_name("logs.txt") // nom suggéré par défaut
                .build();

            let w = win2.clone();
            fd.save(Some(&w), gtk4::gio::Cancellable::NONE, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        if let Err(e) = std::fs::write(&path, &content_str) {
                            // Écriture impossible (permissions, disque plein, etc.)
                            log::error!("Impossible d'écrire le fichier d'export : {e}");
                        }
                        // Succès : aucun feedback UI (la fenêtre de dialogue suffit)
                    }
                }
                // Annulation par l'utilisateur : result = Err → ignoré silencieusement
            });
        });
    }

    window.present(); // affichage final de la fenêtre (non-modal)
}

// =============================================================================
// Parsing interne du texte terminal
// =============================================================================

/// Résultat d'un parse complet du contenu terminal.
struct ParseResult {
    /// Lignes encodées au format `"C|NNNNN|texte"`, prêtes pour la [`gtk4::StringList`].
    encoded: Vec<String>,

    /// Nombre de lignes classées niveau `Error`.
    errors: u32,

    /// Nombre de lignes classées niveau `Warning`.
    warnings: u32,
}

/// Analyse le texte brut du terminal et produit un [`ParseResult`].
///
/// Chaque ligne est détectée, encodée et les compteurs d'erreurs/avertissements
/// sont incrémentés en parallèle pour alimenter la barre de statut.
///
/// # Paramètre
/// - `text` : texte brut du terminal (retourné par `TerminalPanel::get_text()`).
fn parse_logs(text: &str) -> ParseResult {
    let mut errors: u32 = 0; // compteur d'erreurs (incrémenté lors du mapping)
    let mut warnings: u32 = 0; // compteur d'avertissements

    let encoded = text
        .lines() // découpe le texte en lignes (gère \n et \r\n)
        .enumerate() // (index_0basé, ligne)
        .map(|(i, line)| {
            let lvl = LogLevel::detect(line); // détection du niveau par mots-clés

            // Incrément des compteurs selon le niveau détecté
            match lvl {
                LogLevel::Error => errors = errors.saturating_add(1), // saturating_add : évite le dépassement u32
                LogLevel::Warning => warnings = warnings.saturating_add(1),
                _ => {} // autres niveaux : compteur non modifié
            }

            encode_line(i.saturating_add(1), line) // numérotation 1-basée : i+1
        })
        .collect(); // collecte toutes les lignes encodées dans un Vec<String>

    ParseResult {
        encoded,
        errors,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exportable_log_viewer_line_preserves_non_system_payload_timestamp() {
        let rendered = exportable_log_viewer_line('N', "[05:41:03] measured temperature=24.1C");

        assert_eq!(rendered, "[05:41:03] measured temperature=24.1C");
    }

    #[test]
    fn exportable_log_viewer_line_strips_short_timestamp_only_for_system_lines() {
        let rendered = exportable_log_viewer_line('S', "[05:41:03] Systeme pret");

        assert_eq!(rendered, "Systeme pret");
    }
}
