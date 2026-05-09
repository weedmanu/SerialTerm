//! ############################################################################
//! @file    `terminal_panel/display.rs`
//! @author  manu
//! @brief   Comportements UI du [`TerminalPanel`] : construction des widgets GTK,
//!          affichage des données (ANSI, TX, système, erreurs), gestion du
//!          scrollback, barre de recherche intégrée (Ctrl+F).
//! @version    1.0.0
//! @date    2026-03-05
//! @copyright GPL-3.0-or-later.
//! ############################################################################
//!
//! ## Responsabilités de ce fichier
//!
//! Ce fichier contient uniquement les `impl TerminalPanel` qui dépendent de GTK.
//! La struct et le parseur ANSI sont définis dans [`super::ansi`].
//!
//! Fonctions publiques exposées :
//! - [`TerminalPanel::new`]                 — construction complète du widget
//! - [`TerminalPanel::toggle_search`]       — afficher/cacher la barre Ctrl+F
//! - [`TerminalPanel::find_next`]           — occurrence suivante
//! - [`TerminalPanel::find_prev`]           — occurrence précédente
//! - [`TerminalPanel::append_ansi`]         — données RX (séquences ANSI)
//! - [`TerminalPanel::append_sent`]         — écho TX (texte envoyé)
//! - [`TerminalPanel::append_error`]        — message d'erreur horodaté
//! - [`TerminalPanel::clear`]               — effacer le terminal
//! - [`TerminalPanel::get_text`]            — lire tout le contenu
//! - [`TerminalPanel::set_auto_scroll_enabled`] — activer/désactiver l'auto-scroll

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Orientation, ScrolledWindow, SearchBar, SearchEntry, TextBuffer, TextMark,
    TextSearchFlags, TextTagTable, TextView,
};
use vte::Parser;

use super::ansi::{
    AnsiPerformer, LogExportMode, LogLineKind, TerminalPanel, TimestampedLogRecorder,
};
use crate::ui::i18n::UiLang;

// =============================================================================
// Construction du panneau terminal (new + signaux de recherche)
// =============================================================================

fn metric_pixels(value: i32) -> i32 {
    let scale = gtk4::pango::SCALE;
    let rounded = value.saturating_add(scale / 2);
    rounded.checked_div(scale).unwrap_or(0).max(1)
}

fn cells_for_pixels(pixels: i32, cell_pixels: i32) -> u32 {
    let pixels = pixels.max(cell_pixels).max(1);
    let cell_pixels = cell_pixels.max(1);
    let cells = pixels.checked_div(cell_pixels).unwrap_or(0).max(1);
    u32::try_from(cells).unwrap_or(1)
}

fn short_timestamp() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

impl TerminalPanel {
    /// Crée un nouveau panneau terminal complet et prêt à afficher.
    ///
    /// Construit dans l'ordre :
    /// 1. La [`TextTagTable`] avec tous les tags de style nécessaires.
    /// 2. Le [`TextBuffer`] associé à la table.
    /// 3. Le [`TextView`] en lecture seule avec police monospace.
    /// 4. Le [`ScrolledWindow`] enveloppant le `TextView`.
    /// 5. La [`SearchBar`] + [`SearchEntry`] pour Ctrl+F.
    /// 6. Le conteneur vertical `container` (`SearchBar` + `ScrolledWindow`).
    /// 7. L'[`AnsiPerformer`] avec ses caches de tags pré-résolus.
    /// 8. Le [`TextMark`] persistant pour le scroll vers le bas.
    /// 9. La connexion de tous les signaux de recherche.
    ///
    /// # Paramètres
    /// - `max_lines` : limite de lignes dans le scrollback (ex. `10_000`).
    /// - `lang` : langue de l'interface (pour les préfixes système).
    #[allow(clippy::too_many_lines)] // constructeur légitime : initialise tous les tags + widgets
    pub fn new(max_lines: u32, lang: UiLang) -> Self {
        // ── Table de tags GTK ──────────────────────────────────────────────────
        // Tous les TextTag doivent être créés et enregistrés AVANT la création
        // du TextBuffer pour être disponibles lors de l'insertion de texte.
        let tag_table = TextTagTable::new();

        // Tag TX : données envoyées par l'utilisateur (écho local en orange)
        let tx_tag = gtk4::TextTag::builder()
            .name("tx")
            .foreground("orange") // couleur distincte pour distinguer TX de RX
            .build();
        tag_table.add(&tx_tag);

        // Tag RX : données reçues (couleur par défaut du thème = texte normal)
        let rx_tag = gtk4::TextTag::builder().name("rx").build();
        tag_table.add(&rx_tag);

        // Tag système : messages internes horodatés (gris italique)
        let sys_tag = gtk4::TextTag::builder()
            .name("system")
            .foreground("#888888") // gris moyen — discret mais lisible
            .style(gtk4::pango::Style::Italic) // italique pour distinguer des données réseau
            .build();
        tag_table.add(&sys_tag);

        // Tag erreur : messages d'erreur horodatés (rouge gras)
        let err_tag = gtk4::TextTag::builder()
            .name("error")
            .foreground("#ff4444") // rouge vif — attire l'attention
            .weight(700) // gras (700 = Bold dans Pango)
            .build();
        tag_table.add(&err_tag);

        // Tag surbrillance de recherche : fond or, texte noir — contraste élevé
        let search_tag = gtk4::TextTag::builder()
            .name("search-highlight")
            .background("#FFD700") // doré — visible sur thème clair et sombre
            .foreground("#000000") // noir — contraste maximal sur fond doré
            .build();
        tag_table.add(&search_tag);

        // ── Tags ANSI 16 couleurs standard ────────────────────────────────────
        // Deux ensembles : fg_0…fg_15 (avant-plan) et bg_0…bg_15 (arrière-plan).
        // L'ordre suit la norme ANSI/xterm : 0=noir, 1=rouge, 2=vert, 3=jaune,
        // 4=bleu, 5=magenta, 6=cyan, 7=blanc ; 8–15 = variantes brillantes.
        let colors = [
            "#000000", "#CD0000", "#00CD00", "#CDCD00", "#0000EE", "#CD00CD", "#00CDCD",
            "#E5E5E5", // couleurs normales 0–7 (sombre)
            "#7F7F7F", "#FF0000", "#00FF00", "#FFFF00", "#5C5CFF", "#FF00FF", "#00FFFF",
            "#FFFFFF", // couleurs brillantes 8–15 (bright)
        ];

        for (i, color) in colors.iter().enumerate() {
            // Tag avant-plan fg_i : change la couleur du texte
            let fg_tag = gtk4::TextTag::builder()
                .name(format!("fg_{i}")) // nom indexé pour le cache AnsiPerformer
                .foreground(*color)
                .build();
            tag_table.add(&fg_tag);

            // Tag arrière-plan bg_i : change la couleur de fond
            let bg_tag = gtk4::TextTag::builder()
                .name(format!("bg_{i}")) // nom indexé pour le cache AnsiPerformer
                .background(*color)
                .build();
            tag_table.add(&bg_tag);
        }

        // ── Tags d'attributs de style ─────────────────────────────────────────

        // Tag gras : poids 700 = Bold dans Pango
        let bold_tag = gtk4::TextTag::builder().name("bold").weight(700).build();
        tag_table.add(&bold_tag);

        // Tag italique : style Italic Pango
        let italic_tag = gtk4::TextTag::builder()
            .name("italic")
            .style(gtk4::pango::Style::Italic)
            .build();
        tag_table.add(&italic_tag);

        // Tag souligné : underline simple (Single = trait fin sous le texte)
        let underline_tag = gtk4::TextTag::builder()
            .name("underline")
            .underline(gtk4::pango::Underline::Single)
            .build();
        tag_table.add(&underline_tag);

        // ── TextBuffer ────────────────────────────────────────────────────────
        // Le buffer est créé avec la table de tags pour que les insertions
        // puissent immédiatement référencer les tags ci-dessus.
        let buffer = TextBuffer::new(Some(&tag_table));
        let timestamped_log = Rc::new(RefCell::new(TimestampedLogRecorder::default()));

        // ── TextView ──────────────────────────────────────────────────────────
        let text_view = TextView::builder()
            .buffer(&buffer)
            .editable(false) // lecture seule : l'utilisateur ne peut pas modifier
            .cursor_visible(false) // curseur masqué (terminal, pas éditeur)
            .wrap_mode(gtk4::WrapMode::Char) // retour à la ligne au niveau caractère
            .monospace(true) // police monospace obligatoire pour aligner ASCII art
            .top_margin(4) // marges internes pour l'aération visuelle
            .bottom_margin(4)
            .left_margin(8)
            .right_margin(8)
            .vexpand(true) // s'étend verticalement pour occuper tout l'espace disponible
            .hexpand(true) // s'étend horizontalement
            .build();

        text_view.add_css_class("terminal-view"); // classe CSS pour le style personnalisé (thèmes)

        // ── ScrolledWindow ────────────────────────────────────────────────────
        // Enveloppe le TextView pour permettre le défilement vertical/horizontal.
        let scroll_window = ScrolledWindow::builder()
            .vexpand(true) // suit l'expansion du TextView
            .hexpand(true)
            .child(&text_view)
            .build();

        // ── Barre de recherche (Ctrl+F) ───────────────────────────────────────
        let search_entry = SearchEntry::builder()
            .placeholder_text(lang.search_terminal_placeholder())
            .hexpand(true)
            .build();

        let search_bar = SearchBar::builder()
            .show_close_button(true) // bouton ✕ pour fermer la barre sans souris
            .child(&search_entry)
            .build();
        search_bar.connect_entry(&search_entry); // lie la SearchEntry à la SearchBar (focus, raccourcis)

        // ── Conteneur vertical racine ─────────────────────────────────────────
        // Structure : SearchBar (cachée par défaut) en haut, ScrolledWindow en dessous.
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .margin_top(8) // Add margin to prevent overlap with the new header bar
            .spacing(0) // pas d'espace entre SearchBar et ScrolledWindow
            .vexpand(true)
            .hexpand(true)
            .build();
        container.append(&search_bar); // ordre : recherche en haut
        container.append(&scroll_window); // puis le terminal

        // ── AnsiPerformer + parseur VTE ───────────────────────────────────────
        let auto_scroll_enabled = Rc::new(Cell::new(true)); // auto-scroll activé par défaut
        let scroll_pending = Rc::new(Cell::new(false));

        let ansi_parser = Rc::new(RefCell::new(Parser::new())); // parseur VTE sans état initial

        let mut performer = AnsiPerformer::new(buffer.clone(), timestamped_log.clone());
        // Résolution unique de tous les tags depuis la table :
        // après init_tags, AnsiPerformer n'effectue plus aucun lookup de chaîne.
        performer.init_tags(&tag_table);
        let ansi_performer = Rc::new(RefCell::new(performer));

        // ── Mark persistant pour scroll_to_bottom ─────────────────────────────
        // Créé une seule fois et déplacé à chaque trame — évite create_mark/delete_mark.
        // `gravity = false` = le mark reste à la position courante lors d'insertions.
        let scroll_mark = buffer.create_mark(Some("scroll-bottom"), &buffer.end_iter(), false);

        // ── Construction de la struct ─────────────────────────────────────────
        let this = Self {
            container,
            text_view,
            buffer,
            max_lines,
            auto_scroll_enabled,
            scroll_pending,
            ansi_parser,
            ansi_performer,
            scroll_mark,
            trim_counter: Cell::new(0), // compteur de throttle initialisé à 0
            search_bar,
            search_entry,
            lang,
            timestamped_log,
        };

        // Les signaux sont connectés APRÈS construction (closures capturent `this` par clone)
        this.connect_search_signals();
        this // retourne le panneau complet et opérationnel
    }

    /// Calcule une estimation cohérente de la grille visible du terminal.
    ///
    /// Le [`TextView`] GTK n'étant pas un vrai PTY, on approxime la grille
    /// distante à partir de la taille allouée au widget et des métriques Pango
    /// de la police monospace active.
    pub fn visible_grid_size(&self) -> Option<(u32, u32, u32, u32)> {
        let width = self.text_view.width();
        let height = self.text_view.height();
        if width <= 0 || height <= 0 {
            return None;
        }

        let inner_width = width
            .saturating_sub(self.text_view.left_margin())
            .saturating_sub(self.text_view.right_margin());
        let inner_height = height
            .saturating_sub(self.text_view.top_margin())
            .saturating_sub(self.text_view.bottom_margin());
        if inner_width <= 0 || inner_height <= 0 {
            return None;
        }

        let context = self.text_view.pango_context();
        let metrics = context.metrics(None, None);
        let cell_width = metric_pixels(metrics.approximate_char_width());
        let cell_height = metric_pixels(metrics.ascent().saturating_add(metrics.descent()));

        Some((
            cells_for_pixels(inner_width, cell_width),
            cells_for_pixels(inner_height, cell_height),
            u32::try_from(inner_width).ok()?,
            u32::try_from(inner_height).ok()?,
        ))
    }

    /// Connecte les signaux GTK de la barre de recherche.
    ///
    /// Séparé de [`Self::new`] pour rester sous la limite `clippy::too_many_lines`.
    /// Connecte trois comportements :
    /// 1. Saisie de texte → surbrillance en temps réel de toutes les occurrences.
    /// 2. Touche Entrée → navigation vers l'occurrence suivante.
    /// 3. Fermeture de la barre → suppression de toutes les surbrillances.
    fn connect_search_signals(&self) {
        // Signal 1 : changement de texte → surbrillance en temps réel
        {
            let tv = self.text_view.clone(); // clone GObject ref-counted pour la closure
            let buf = self.buffer.clone();
            self.search_entry.connect_search_changed(move |entry| {
                let query = entry.text().to_string(); // texte saisi par l'utilisateur
                Self::highlight_matches_static(&buf, &tv, &query); // surlignage dynamique
            });
        }

        // Signal 2 : touche Entrée → occurrence suivante
        {
            let buf = self.buffer.clone();
            let tv = self.text_view.clone();
            let mark = self.scroll_mark.clone(); // clone du TextMark pour la closure
            self.search_entry.connect_activate(move |entry| {
                let query = entry.text().to_string(); // texte courant de la barre
                Self::find_next_static(&buf, &tv, &mark, &query); // navigation avant
            });
        }

        // Signal 3 : fermeture de la barre → nettoyage des surbrillances
        {
            let buf = self.buffer.clone();
            self.search_bar
                .connect_search_mode_enabled_notify(move |bar| {
                    if !bar.is_search_mode() {
                        // La barre vient d'être fermée : retirer tous les tags de surbrillance
                        let start = buf.start_iter(); // début absolu du buffer
                        let end = buf.end_iter(); // fin absolue du buffer
                        if let Some(tag) = buf.tag_table().lookup("search-highlight") {
                            buf.remove_tag(&tag, &start, &end); // suppression sur tout le buffer
                        }
                    }
                });
        }
    }

    // =========================================================================
    // Interface publique — recherche
    // =========================================================================

    /// Bascule la visibilité de la barre de recherche intégrée.
    ///
    /// - Si la barre est cachée → l'affiche et donne le focus au champ de saisie.
    /// - Si la barre est visible → la cache et efface toutes les surbrillances.
    pub fn toggle_search(&self) {
        let enabled = !self.search_bar.is_search_mode(); // inverse l'état courant
        self.search_bar.set_search_mode(enabled);

        if enabled {
            self.search_entry.grab_focus(); // focus automatique pour saisir immédiatement
        } else {
            // Fermeture : nettoyage des surbrillances résiduelles dans tout le buffer
            let start = self.buffer.start_iter();
            let end = self.buffer.end_iter();
            if let Some(tag) = self.buffer.tag_table().lookup("search-highlight") {
                self.buffer.remove_tag(&tag, &start, &end); // suppression globale du surbrillage
            }
        }
    }

    /// Navigue vers l'**occurrence suivante** de la recherche courante.
    ///
    /// Délègue à [`Self::find_next_static`] en passant les références du buffer, view et mark.
    /// Gère le wrap-around (retour au début si fin de buffer atteinte).
    pub fn find_next(&self) {
        let query = self.search_entry.text().to_string(); // texte courant dans la barre
        Self::find_next_static(&self.buffer, &self.text_view, &self.scroll_mark, &query);
    }

    /// Navigue vers l'**occurrence précédente** de la recherche courante.
    ///
    /// Recherche en arrière depuis la borne gauche de la sélection courante.
    /// Gère le wrap-around (retour à la fin si début de buffer atteint).
    pub fn find_prev(&self) {
        let query = self.search_entry.text().to_string(); // texte courant dans la barre
        if query.is_empty() {
            return; // rien à chercher : sortie immédiate
        }

        // Point de départ : borne gauche de la sélection courante, ou fin du buffer si aucune sélection
        let search_start = self
            .buffer
            .selection_bounds()
            .map_or_else(|| self.buffer.end_iter(), |(start, _)| start);

        let found = search_start.backward_search(&query, TextSearchFlags::CASE_INSENSITIVE, None);

        if let Some((match_start, match_end)) = found {
            // Occurrence précédente trouvée : sélection et défilement vers elle
            self.buffer.select_range(&match_start, &match_end);
            self.buffer.move_mark(&self.scroll_mark, &match_start); // déplace le mark au début du match
            self.text_view
                .scroll_to_mark(&self.scroll_mark, 0.0, true, 0.5, 0.5); // centré verticalement et horizontalement
        } else {
            // Wrap-around : aucune occurrence avant → revenir à la dernière occurrence depuis la fin
            let end = self.buffer.end_iter();
            if let Some((ms, me)) =
                end.backward_search(&query, TextSearchFlags::CASE_INSENSITIVE, None)
            {
                self.buffer.select_range(&ms, &me);
                self.buffer.move_mark(&self.scroll_mark, &ms);
                self.text_view
                    .scroll_to_mark(&self.scroll_mark, 0.0, true, 0.5, 0.5);
            }
            // Si toujours rien : le buffer ne contient aucune occurrence, on ne fait rien
        }
    }

    /// Navigue vers l'occurrence suivante (version statique, utilisable dans les closures).
    ///
    /// Statique car les closures GTK ne peuvent pas capturer `&self` directement.
    /// Gère le wrap-around : si aucune occurrence après le curseur, reprend depuis le début.
    fn find_next_static(buf: &TextBuffer, tv: &TextView, mark: &TextMark, query: &str) {
        if query.is_empty() {
            return; // chaîne vide : rien à chercher
        }

        // Point de départ : borne droite de la sélection courante, ou début si aucune sélection
        let search_start = buf
            .selection_bounds()
            .map_or_else(|| buf.start_iter(), |(_, end)| end);

        let found = search_start.forward_search(query, TextSearchFlags::CASE_INSENSITIVE, None);

        if let Some((ms, me)) = found {
            // Occurrence trouvée en avant : sélection et scroll
            buf.select_range(&ms, &me);
            buf.move_mark(mark, &ms); // déplace le mark au début du match
            tv.scroll_to_mark(mark, 0.0, true, 0.5, 0.5); // centré
        } else {
            // Wrap-around : reprend depuis le début du buffer
            let start = buf.start_iter();
            if let Some((ms, me)) =
                start.forward_search(query, TextSearchFlags::CASE_INSENSITIVE, None)
            {
                buf.select_range(&ms, &me);
                buf.move_mark(mark, &ms);
                tv.scroll_to_mark(mark, 0.0, true, 0.5, 0.5);
            }
            // Si toujours rien : buffer sans occurrence, on ne fait rien
        }
    }

    /// Surligne toutes les occurrences de `query` dans le buffer (insensible à la casse).
    ///
    /// Commence par effacer toute surbrillance existante, puis applique le tag
    /// `search-highlight` sur chaque occurrence trouvée de gauche à droite.
    /// Version statique pour être appelable depuis les closures GTK.
    fn highlight_matches_static(buf: &TextBuffer, _tv: &TextView, query: &str) {
        let start = buf.start_iter(); // début absolu du buffer
        let end = buf.end_iter(); // fin absolue du buffer

        // Effacement de la surbrillance précédente sur tout le buffer
        if let Some(tag) = buf.tag_table().lookup("search-highlight") {
            buf.remove_tag(&tag, &start, &end); // suppression du tag sur toute la plage
        }

        if query.is_empty() {
            return; // pas de texte à surligner : sortie après nettoyage
        }

        // Récupération du tag de surbrillance (doit exister, créé dans new())
        let Some(tag) = buf.tag_table().lookup("search-highlight") else {
            return; // défense : tag absent (ne devrait jamais arriver)
        };

        // Parcours séquentiel du buffer pour surligner toutes les occurrences
        let mut iter = buf.start_iter(); // curseur de recherche, part du début
        while let Some((ms, me)) =
            iter.forward_search(query, TextSearchFlags::CASE_INSENSITIVE, None)
        {
            buf.apply_tag(&tag, &ms, &me); // applique le surbrillage sur cette occurrence
            iter = me; // avance l'itérateur juste après le match (évite les boucles infinies)
        }
    }

    // =========================================================================
    // Interface publique — affichage des données
    // =========================================================================

    /// Réinitialise l'état du parseur ANSI et les attributs courants entre deux sessions.
    ///
    /// À appeler au début de chaque nouvelle connexion pour éviter que l'état partiel
    /// d'une session précédente (séquence d'échappement tronquée, couleur active) ne
    /// contamine l'affichage de la session suivante.
    pub fn reset_ansi_state(&self) {
        *self.ansi_parser.borrow_mut() = vte::Parser::new();
        self.ansi_performer.borrow_mut().reset_session();
    }

    /// Ajoute des données reçues (RX) au terminal en parsant les séquences ANSI.
    ///
    /// Passe les octets bruts au parseur VTE qui appelle [`vte::Perform::print`],
    /// [`vte::Perform::execute`] et [`vte::Perform::csi_dispatch`] selon les séquences.
    /// Appelle [`AnsiPerformer::flush`] pour vider le texte accumulé dans le buffer GTK.
    ///
    /// Déclenche le trim du scrollback toutes les 32 trames et l'auto-scroll si activé.
    pub fn append_ansi(&self, data: &[u8]) {
        let mut parser = self.ansi_parser.borrow_mut(); // accès exclusif au parseur VTE
        let mut performer = self.ansi_performer.borrow_mut(); // accès exclusif au récepteur

        parser.advance(&mut *performer, data); // traitement des octets → callbacks Perform
        performer.flush(); // insertion finale du texte accumulé dans le buffer GTK
        self.timestamped_log
            .borrow_mut()
            .trim_to_max_lines(usize::try_from(self.max_lines).unwrap_or(usize::MAX));

        // Throttle : trim_scrollback n'est appelé que toutes les 32 trames (coûteux O(n))
        self.maybe_trim_scrollback();

        if self.auto_scroll_enabled.get() {
            self.schedule_scroll_to_bottom(); // défilement automatique différé si activé
        }
    }

    /// Ajoute du texte **envoyé** (TX) au terminal — écho local en orange.
    ///
    /// Utilise le tag `tx` (couleur orange) pour distinguer les données envoyées
    /// des données reçues (RX, couleur par défaut).
    pub fn append_sent(&self, text: &str) {
        self.timestamped_log
            .borrow_mut()
            .append_fragment(LogLineKind::Tx, text);
        self.append_with_tag(text, "tx"); // tag orange pour l'écho TX
    }

    #[cfg(test)]
    /// Helper de test : ajoute un **message système** horodaté au terminal.
    pub fn append_system(&self, text: &str) {
        let timestamp = short_timestamp();
        self.timestamped_log
            .borrow_mut()
            .append_fragment(LogLineKind::System, &format!("{text}\n"));
        self.append_with_tag(&format!("[{timestamp}] {text}\n"), "system"); // tag gris italique
    }

    /// Ajoute un **message d'erreur** horodaté au terminal.
    ///
    /// Format : `[HH:MM:SS] ERREUR: message\n` — texte rouge gras.
    /// Utilisé pour les erreurs de connexion, d'envoi, de configuration…
    pub fn append_error(&self, text: &str) {
        let timestamp = short_timestamp();
        self.timestamped_log.borrow_mut().append_fragment(
            LogLineKind::System,
            &format!("{} {text}\n", self.lang.error_prefix()),
        );
        self.append_with_tag(
            &format!("[{timestamp}] {} {text}\n", self.lang.error_prefix()),
            "error",
        );
        // tag rouge gras
    }

    /// Insère du texte avec le tag nommé `tag_name` et déclenche le scroll/trim.
    ///
    /// Si le tag est introuvable dans la table (cas défensif), insère sans style.
    /// Factorise la logique commune à [`Self::append_sent`] et [`Self::append_error`].
    fn append_with_tag(&self, text: &str, tag_name: &str) {
        let mut end_iter = self.buffer.end_iter(); // curseur en fin de buffer

        let tag_table = self.buffer.tag_table();
        if let Some(tag) = tag_table.lookup(tag_name) {
            // Tag trouvé : insertion avec style
            self.buffer.insert_with_tags(&mut end_iter, text, &[&tag]);
        } else {
            // Tag absent (ne devrait jamais arriver) : insertion sans style par défaut
            self.buffer.insert(&mut end_iter, text);
        }

        // Throttle scrollback + auto-scroll (même logique que append_ansi)
        self.maybe_trim_scrollback();

        if self.auto_scroll_enabled.get() {
            self.schedule_scroll_to_bottom(); // défilement automatique différé si activé
        }

        self.timestamped_log
            .borrow_mut()
            .trim_to_max_lines(usize::try_from(self.max_lines).unwrap_or(usize::MAX));
    }

    // =========================================================================
    // Gestion du scrollback (throttle + purge)
    // =========================================================================

    /// Incrémente le compteur de throttle et appelle [`Self::trim_scrollback`] toutes les 32 fois.
    ///
    /// `buffer.line_count()` est une opération O(n) sur le thread GTK.
    /// L'appeler à chaque octet reçu serait catastrophique à haut débit.
    /// Ce throttle réduit la fréquence à 1/32 sans perte de sécurité.
    fn maybe_trim_scrollback(&self) {
        const TRIM_EVERY: u32 = 32; // déclenche le trim 1 fois sur 32 insertions

        let c = self.trim_counter.get().wrapping_add(1); // incrément avec wrap-around à 0 (pas de panique à u32::MAX)
        self.trim_counter.set(c);

        if c % TRIM_EVERY == 0 {
            self.trim_scrollback(); // seulement sur les multiples de 32
        }
    }

    /// Supprime les lignes les plus anciennes du buffer si `line_count > max_lines`.
    ///
    /// Calcule le nombre de lignes à retirer (`lines_to_remove`), positionne
    /// l'itérateur de fin sur la dernière ligne à supprimer, puis supprime le bloc.
    fn trim_scrollback(&self) {
        let line_count = self.buffer.line_count(); // nombre de lignes actuelles (O(n))
        let max_lines_i32 = i32::try_from(self.max_lines).unwrap_or(i32::MAX); // conversion sûre u32 → i32

        if line_count > max_lines_i32 {
            // Calcul du nombre de lignes excédentaires à supprimer
            // `saturating_sub` évite l'underflow : la branche garantit `line_count > max_lines_i32`
            // mais clippy::arithmetic_side_effects ne peut pas le prouver statiquement.
            let lines_to_remove = line_count.saturating_sub(max_lines_i32);

            let mut start = self.buffer.start_iter(); // début du buffer (ligne 0)
            let mut end = self.buffer.iter_at_line(lines_to_remove).unwrap_or(start); // itérateur à la ligne de coupure

            // Si l'itérateur est au milieu d'une ligne (line_offset != 0), avancer jusqu'à la fin
            // de cette ligne + le caractère de saut de ligne pour ne pas laisser de fragment.
            if end.line_offset() != 0 {
                end.forward_to_line_end(); // avance jusqu'au \n de fin de ligne
                end.forward_char(); // inclut le \n lui-même dans la suppression
            }

            self.buffer.delete(&mut start, &mut end); // suppression du bloc de lignes anciennes
            self.timestamped_log
                .borrow_mut()
                .trim_to_max_lines(usize::try_from(self.max_lines).unwrap_or(usize::MAX));
        }
    }

    // =========================================================================
    // Défilement
    // =========================================================================

    /// Défile le terminal vers le bas (dernière ligne visible).
    ///
    /// Déplace le [`TextMark`] persistant en fin de buffer puis appelle
    /// `scroll_to_mark`. Évite `create_mark`/`delete_mark` à chaque trame.
    fn schedule_scroll_to_bottom(&self) {
        if self.scroll_pending.get() {
            return;
        }

        self.scroll_pending.set(true);

        let buffer = self.buffer.clone();
        let text_view = self.text_view.clone();
        let scroll_mark = self.scroll_mark.clone();
        let scroll_pending = self.scroll_pending.clone();

        gtk4::glib::idle_add_local_once(move || {
            scroll_pending.set(false);

            if !text_view.is_visible() || !text_view.is_mapped() {
                return;
            }

            buffer.move_mark(&scroll_mark, &buffer.end_iter());
            text_view.scroll_to_mark(&scroll_mark, 0.0, false, 0.0, 1.0);
        });
    }

    // =========================================================================
    // Interface publique — utilitaires
    // =========================================================================

    /// Efface intégralement le contenu du terminal.
    ///
    /// Supprime tout le texte du buffer, y compris les styles et tags.
    pub fn clear(&self) {
        self.buffer
            .delete(&mut self.buffer.start_iter(), &mut self.buffer.end_iter());
        self.timestamped_log.borrow_mut().clear();
        // suppression [début, fin[
    }

    /// Retourne le contenu textuel complet du terminal (sans les tags de style).
    ///
    /// Utilisé pour la sauvegarde de logs et le visualiseur de logs.
    /// Le troisième paramètre `false` exclut les caractères "invisibles" GTK.
    pub fn get_text(&self) -> String {
        self.buffer
            .text(&self.buffer.start_iter(), &self.buffer.end_iter(), false)
            .to_string() // conversion GString → String Rust
    }

    /// Retourne le texte exportable selon le mode demandé.
    pub fn export_text(&self, mode: LogExportMode) -> String {
        match mode {
            LogExportMode::Raw => self.timestamped_log.borrow().export_raw_text(),
            LogExportMode::Timestamped => self
                .timestamped_log
                .borrow()
                .export_timestamped_session_text(),
            LogExportMode::Split => self.timestamped_log.borrow().export_split_text(),
        }
    }

    /// Active ou désactive le défilement automatique vers le bas.
    ///
    /// Quand désactivé, l'utilisateur peut faire défiler librement vers le haut
    /// sans que le terminal ne revienne en bas à chaque nouvelle donnée reçue.
    pub fn set_auto_scroll_enabled(&self, enabled: bool) {
        self.auto_scroll_enabled.set(enabled); // mise à jour atomique (Cell)
    }

    /// Retourne un handle partagé `Rc<Cell<bool>>` sur l'état de l'auto-scroll.
    ///
    /// Permet à d'autres widgets (ex. bouton de la barre d'outils) de lire
    /// ou modifier l'état de l'auto-scroll sans accès direct à `self`.
    #[allow(dead_code, clippy::clone_on_ref_ptr)] // pattern GTK4 : clone de Rc intentionnel
    pub fn auto_scroll_handle(&self) -> Rc<Cell<bool>> {
        self.auto_scroll_enabled.clone() // clone du Rc (incrément du compteur de références)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gtk4::test]
    fn append_error_formats_terminal_message_with_error_prefix() {
        crate::ui::runtime::sanitize_problematic_desktop_theme();
        let terminal = TerminalPanel::new(256, UiLang::Fr);

        terminal.append_error("mot de passe incorrect");

        let content = terminal.get_text();
        assert!(content.contains("ERREUR: mot de passe incorrect"));

        let error_tag = terminal
            .buffer
            .tag_table()
            .lookup("error")
            .expect("le tag d'erreur doit exister");
        assert!(error_tag.property::<bool>("foreground-set"));
        assert_eq!(error_tag.property::<i32>("weight"), 700);
    }

    #[gtk4::test]
    fn export_text_with_timestamps_keeps_single_prefix_per_line() {
        crate::ui::runtime::sanitize_problematic_desktop_theme();
        let terminal = TerminalPanel::new(256, UiLang::Fr);

        terminal.append_system("connexion etablie");
        terminal.append_sent("help\n");
        terminal.append_ansi(b"OK\n");

        let exported = terminal.export_text(LogExportMode::Timestamped);

        assert!(!exported.contains("connexion etablie"));
        assert!(exported.contains("help\n"));
        assert!(exported.contains("OK\n"));
        assert!(!exported.contains("] ["));
    }

    #[gtk4::test]
    fn export_text_raw_strips_ui_short_timestamp_prefixes() {
        crate::ui::runtime::sanitize_problematic_desktop_theme();
        let terminal = TerminalPanel::new(256, UiLang::Fr);

        terminal.append_system("Terminal efface.");
        terminal.append_sent("AT\n");
        terminal.append_ansi(b"OK\n");

        let exported = terminal.export_text(LogExportMode::Raw);

        assert_eq!(exported, "AT\nOK\n");
        assert!(!exported.contains('['));
    }

    #[gtk4::test]
    fn append_ansi_normalizes_crlf_without_inserting_blank_lines() {
        crate::ui::runtime::sanitize_problematic_desktop_theme();
        let terminal = TerminalPanel::new(256, UiLang::Fr);

        terminal.append_ansi(b"ready\r\nOK\r\n");

        assert_eq!(terminal.get_text(), "ready\nOK\n");
        assert_eq!(terminal.export_text(LogExportMode::Raw), "ready\nOK\n");
    }

    #[gtk4::test]
    fn export_text_split_keeps_distinct_sections() {
        crate::ui::runtime::sanitize_problematic_desktop_theme();
        let terminal = TerminalPanel::new(256, UiLang::Fr);

        terminal.append_system("Terminal efface.");
        terminal.append_sent("AT\n");
        terminal.append_ansi(b"OK\n");

        let exported = terminal.export_text(LogExportMode::Split);

        assert!(exported.contains("=== RX ==="));
        assert!(exported.contains("=== TX ==="));
        assert!(exported.contains("=== SYSTEM ==="));
        assert!(exported.contains("OK\n"));
        assert!(exported.contains("AT\n"));
        assert!(exported.contains("Terminal efface."));
    }
}
