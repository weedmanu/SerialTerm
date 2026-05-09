//! ############################################################################
//! @file    `terminal_panel/ansi.rs`
//! @author  manu
//! @brief   Struct [`TerminalPanel`] + parseur ANSI VT100/xterm-256color/truecolor.
//!          Supporte : 16 couleurs ANSI standard, cube xterm-256color (index 16–255),
//!                     RGB truecolor 24 bits, bold / italic / underline / reset.
//!          Utilise   : crate [`vte`] pour le parsing bas niveau des séquences ANSI,
//!                      [`gtk4::TextBuffer`] + [`gtk4::TextTag`] pour l'affichage coloré.
//! @version    1.0.0
//! @date    2026-03-05
//! @copyright GPL-3.0-or-later.
//! ############################################################################
//!
//! ## Architecture
//!
//! Ce fichier contient deux éléments principaux :
//! 1. **[`TerminalPanel`]** — struct publique exposée au reste de l'UI.
//!    Ses champs `pub(crate)` sont accédés depuis [`super::display`]
//!    et [`crate::ui::window`] pour la sauvegarde de logs.
//! 2. **[`AnsiPerformer`]** — récepteur interne du parseur VTE,
//!    implémente [`vte::Perform`] et applique les styles au buffer GTK.
//!
//! ## Stratégie de performance
//!
//! - 16 tags ANSI standard résolus **une seule fois** dans [`AnsiPerformer::init_tags`].
//! - Tags étendus (256/truecolor) créés **paresseusement** et mis en cache (clé `u32`).
//! - Texte accumulé dans `pending_text`, inséré en **une seule** opération GTK par flush.
//! - `tag_buf` réutilisé à chaque flush (évite une allocation de `Vec` par trame).

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use chrono::Local;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, SearchBar, SearchEntry, TextBuffer, TextMark, TextTag, TextTagTable, TextView,
};
use vte::{Parser, Perform};

use crate::ui::i18n::UiLang;

// =============================================================================
// Struct publique TerminalPanel
// =============================================================================

/// Panneau d'affichage du terminal.
///
/// Contient un [`TextView`] en lecture seule avec :
/// - **auto-scroll** configurable vers le bas à chaque nouvelle donnée,
/// - **scrollback** borné (`max_lines`) : les lignes les plus anciennes sont purgées,
/// - **parseur ANSI** complet (16 couleurs, xterm-256color, RGB truecolor, bold/italic/underline),
/// - **barre de recherche** intégrée (Ctrl+F), navigation avant/arrière.
///
/// Les champs `pub(crate)` sont accessibles depuis [`super::display`] et
/// depuis [`crate::ui::window`] pour la sauvegarde des logs.
/// Les champs `pub(super)` sont réservés à [`super::display`].
#[derive(Clone)]
pub struct TerminalPanel {
    /// Widget racine à insérer dans le layout parent (orientation verticale).
    /// Contient la [`SearchBar`] en haut et le [`gtk4::ScrolledWindow`] en dessous.
    pub(crate) container: GtkBox,

    /// Zone de texte GTK en lecture seule (`editable = false`).
    /// Affiche le contenu du terminal avec coloration et styles ANSI.
    pub(crate) text_view: TextView,

    /// Buffer de texte GTK associé au `text_view`.
    /// Contient le texte scrollable et la [`TextTagTable`] de styles.
    pub(crate) buffer: TextBuffer,

    /// Nombre maximum de lignes conservées dans le scrollback.
    /// Au-delà, les lignes les plus anciennes sont supprimées par `trim_scrollback`.
    pub(crate) max_lines: u32,

    /// Activation du défilement automatique vers le bas à chaque nouvelle donnée.
    /// `Rc<Cell<bool>>` permet le partage sans mutation entre plusieurs closures GTK.
    pub(super) auto_scroll_enabled: Rc<Cell<bool>>,

    /// Indique si un scroll différé est déjà en attente sur la boucle GTK.
    pub(super) scroll_pending: Rc<Cell<bool>>,

    /// Parseur d'état VTE — convertit les octets bruts en appels du trait [`Perform`].
    /// `Rc<RefCell<_>>` nécessaire pour le partage entre `append_ansi` et les closures GTK.
    pub(super) ansi_parser: Rc<RefCell<Parser>>,

    /// Récepteur des événements VTE — applique les styles ANSI au [`TextBuffer`].
    pub(super) ansi_performer: Rc<RefCell<AnsiPerformer>>,

    /// Mark persistant pour `scroll_to_bottom`.
    /// Réutilisé à chaque trame : évite `create_mark` + `delete_mark` à chaque appel.
    pub(crate) scroll_mark: TextMark,

    /// Compteur d'insertions pour espacer les appels coûteux à `trim_scrollback`.
    /// `trim_scrollback` n'est déclenché que toutes les 32 insertions.
    pub(super) trim_counter: Cell<u32>,

    /// Barre de recherche GTK (cachée par défaut, affichée via Ctrl+F).
    pub(super) search_bar: SearchBar,

    /// Champ de saisie de la recherche, connecté à `search_bar`.
    pub(super) search_entry: SearchEntry,

    /// Langue de l'interface pour les messages système (préfixes d'erreur, etc.).
    pub(super) lang: UiLang,

    /// Capture des lignes exportables avec horodatage réel à l'émission.
    pub(super) timestamped_log: Rc<RefCell<TimestampedLogRecorder>>,
}

/// Ligne complète capturée pour l'export horodaté.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogExportMode {
    Raw,
    Timestamped,
    Split,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LogLineKind {
    Rx,
    Tx,
    System,
}

struct TimestampedLine {
    kind: LogLineKind,
    timestamp: String,
    text: String,
}

/// Accumulateur des lignes du terminal avec leur horodatage réel.
#[derive(Default)]
pub(super) struct TimestampedLogRecorder {
    completed_lines: VecDeque<TimestampedLine>,
    current_line: String,
    current_kind: Option<LogLineKind>,
    current_timestamp: Option<String>,
}

impl TimestampedLogRecorder {
    pub(super) fn append_fragment(&mut self, kind: LogLineKind, text: &str) {
        if !self.current_line.is_empty() && self.current_kind != Some(kind) {
            self.finish_current_line();
        }

        for ch in text.chars() {
            if self.current_timestamp.is_none() {
                self.current_timestamp = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
            }
            if self.current_kind.is_none() {
                self.current_kind = Some(kind);
            }

            self.current_line.push(ch);

            if ch == '\n' {
                self.finish_current_line();
            }
        }
    }

    fn finish_current_line(&mut self) {
        if self.current_line.is_empty() {
            self.current_timestamp = None;
            return;
        }

        let timestamp = self
            .current_timestamp
            .take()
            .unwrap_or_else(|| Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
        let kind = self.current_kind.take().unwrap_or(LogLineKind::Rx);
        let text = std::mem::take(&mut self.current_line);

        self.completed_lines.push_back(TimestampedLine {
            kind,
            timestamp,
            text,
        });
    }

    pub(super) fn trim_to_max_lines(&mut self, max_lines: usize) {
        while self.completed_lines.len() > max_lines {
            self.completed_lines.pop_front();
        }
    }

    pub(super) fn clear(&mut self) {
        self.completed_lines.clear();
        self.current_line.clear();
        self.current_kind = None;
        self.current_timestamp = None;
    }

    pub(super) fn export_timestamped_session_text(&self) -> String {
        let mut output = String::new();

        for line in &self.completed_lines {
            if line.kind == LogLineKind::System {
                continue;
            }
            output.push('[');
            output.push_str(&line.timestamp);
            output.push_str("] ");
            output.push_str(&line.text);
        }

        if !self.current_line.is_empty() && self.current_kind != Some(LogLineKind::System) {
            let timestamp = self
                .current_timestamp
                .as_deref()
                .unwrap_or("1970-01-01 00:00:00");

            output.push('[');
            output.push_str(timestamp);
            output.push_str("] ");
            output.push_str(&self.current_line);
        }

        output
    }

    pub(super) fn export_raw_text(&self) -> String {
        let mut output = String::new();

        for line in &self.completed_lines {
            if line.kind == LogLineKind::System {
                continue;
            }
            output.push_str(&line.text);
        }

        if !self.current_line.is_empty() && self.current_kind != Some(LogLineKind::System) {
            output.push_str(&self.current_line);
        }

        output
    }

    pub(super) fn export_split_text(&self) -> String {
        let mut rx = String::new();
        let mut tx = String::new();
        let mut system = String::new();

        for line in &self.completed_lines {
            let target = match line.kind {
                LogLineKind::Rx => &mut rx,
                LogLineKind::Tx => &mut tx,
                LogLineKind::System => &mut system,
            };
            target.push('[');
            target.push_str(&line.timestamp);
            target.push_str("] ");
            target.push_str(&line.text);
        }

        if !self.current_line.is_empty() {
            let timestamp = self
                .current_timestamp
                .as_deref()
                .unwrap_or("1970-01-01 00:00:00");
            let target = match self.current_kind.unwrap_or(LogLineKind::Rx) {
                LogLineKind::Rx => &mut rx,
                LogLineKind::Tx => &mut tx,
                LogLineKind::System => &mut system,
            };
            target.push('[');
            target.push_str(timestamp);
            target.push_str("] ");
            target.push_str(&self.current_line);
        }

        let mut output = String::new();
        append_split_section(&mut output, "RX", &rx);
        append_split_section(&mut output, "TX", &tx);
        append_split_section(&mut output, "SYSTEM", &system);
        output
    }
}

fn append_split_section(output: &mut String, title: &str, content: &str) {
    if content.is_empty() {
        return;
    }

    if !output.is_empty() {
        output.push('\n');
    }

    output.push_str("=== ");
    output.push_str(title);
    output.push_str(" ===\n");
    output.push_str(content);
}

// =============================================================================
// Couleurs ANSI étendues (xterm-256color + truecolor)
// =============================================================================

/// Résultat du décodage d'une couleur étendue SGR 38 (avant-plan) ou 48 (arrière-plan).
///
/// Distingue les 16 premières couleurs ANSI (cache rapide via tableau de tags)
/// des couleurs RGB arbitraires (créées paresseusement dans un `HashMap`).
enum ExtColor {
    /// Couleur appartenant aux 16 premières de la palette ANSI standard (indices 0–15).
    /// Ces couleurs utilisent le cache de tags pré-résolu pour éviter tout lookup de chaîne.
    Ansi(u8),

    /// Couleur RGB 24 bits issue du cube xterm-256 (index 16–255) ou du mode truecolor.
    /// Un [`TextTag`] GTK dédié est créé paresseusement et mis en cache par clé `u32`.
    Rgb(u8, u8, u8),
}

/// Convertit un index xterm-256 (plage 16–255) en triplet RGB.
///
/// Le cube couleur xterm-256 est organisé en deux zones :
/// - **16–231** : cube 6×6×6 — chaque composante prend l'une des six valeurs
///   `{0, 95, 135, 175, 215, 255}` selon l'indice de position dans l'axe.
/// - **232–255** : rampe de 24 niveaux de gris, de 8 à 238 par pas de 10.
///
/// Les index 0–15 (palette ANSI de base) sont gérés séparément via le cache
/// de tags et **ne passent jamais** par cette fonction.
///
/// # Exemple
/// ```
/// // Rouge pur dans le cube 6×6×6 (position r=5, g=0, b=0)
/// assert_eq!(xterm256_color(196), (255, 0, 0));
/// // Premier gris de la rampe (index 232)
/// assert_eq!(xterm256_color(232), (8, 8, 8));
/// ```
const fn xterm256_color(idx: u8) -> (u8, u8, u8) {
    /// Convertit un indice d'axe du cube (0–5) en valeur de composante RGB.
    ///
    /// Les six niveaux officiels de la palette xterm-256 sont :
    /// `0, 95, 135, 175, 215, 255`.
    const fn cube_val(n: u8) -> u8 {
        match n {
            0 => 0,   // niveau 0 : noir complet
            1 => 95,  // niveau 1 : sombre
            2 => 135, // niveau 2 : moyen-sombre
            3 => 175, // niveau 3 : moyen-clair
            4 => 215, // niveau 4 : clair
            _ => 255, // niveau 5 (et au-delà) : plein blanc
        }
    }

    match idx {
        16..=231 => {
            // Soustraction sans débordement : le `match` garantit idx >= 16.
            let n = idx.saturating_sub(16); // position linéaire 0..215 dans le cube 6×6×6

            // Décomposition de la position linéaire en coordonnées (r_idx, g_idx, b_idx).
            // Rouge varie le plus lentement (paliers de 36), bleu le plus vite (paliers de 1).
            (
                cube_val(n / 36),       // indice rouge   : plan   dans le cube (0–5)
                cube_val((n % 36) / 6), // indice vert    : ligne  dans le cube (0–5)
                cube_val(n % 6),        // indice bleu    : colonne dans le cube (0–5)
            )
        }
        232..=255 => {
            // Soustraction sans débordement : le `match` garantit idx >= 232.
            // Rampe de 24 niveaux de gris : premier niveau = 8, pas = 10, dernier = 238.
            let level = idx
                .saturating_sub(232) // indice 0..23 dans la rampe gris
                .saturating_mul(10) // pas de 10 entre deux niveaux consécutifs
                .saturating_add(8); // décalage initial : le premier gris vaut 8

            (level, level, level) // R = G = B = gris pur
        }

        // Cas 0–15 : jamais appelé depuis parse_color_ext (redirigé vers ExtColor::Ansi).
        _ => (0, 0, 0),
    }
}

// =============================================================================
// AnsiPerformer — récepteur d'état pour les séquences VT/ANSI
// =============================================================================

/// Récepteur des événements VTE ; implémente [`vte::Perform`].
///
/// Traduit les codes ANSI (couleurs, styles) en [`TextTag`] GTK appliqués
/// au [`TextBuffer`]. Le texte imprimable est accumulé dans `pending_text`
/// jusqu'à un changement de style, puis inséré en une seule opération GTK.
///
/// ## Hiérarchie de priorité des couleurs
/// 1. Couleur étendue RGB (`current_fg_rgb` / `current_bg_rgb`) — prioritaire.
/// 2. Couleur ANSI standard (`current_fg` / `current_bg`) — utilisée si pas de RGB.
/// 3. Couleur par défaut du thème GTK — si aucune couleur n'est active.
const FLAG_LAST_WAS_CR: u8 = 1 << 0;
const FLAG_BOLD: u8 = 1 << 1;
const FLAG_ITALIC: u8 = 1 << 2;
const FLAG_UNDERLINE: u8 = 1 << 3;
const STYLE_FLAGS_MASK: u8 = FLAG_BOLD | FLAG_ITALIC | FLAG_UNDERLINE;

pub(super) struct AnsiPerformer {
    /// Buffer GTK dans lequel le texte et les tags sont insérés.
    buffer: TextBuffer,

    /// Texte en attente d'insertion.
    /// Accumulé depuis `print()` et `execute()` jusqu'au prochain flush.
    pending_text: String,

    /// Registre d'état compact : CR traité + styles ANSI actifs.
    ///
    /// On évite quatre booléens séparés tout en gardant des accès explicites via
    /// `FLAG_LAST_WAS_CR`, `FLAG_BOLD`, `FLAG_ITALIC` et `FLAG_UNDERLINE`.
    flags: u8,

    /// Index (0–15) de la couleur avant-plan ANSI courante.
    /// `None` = couleur par défaut du thème GTK.
    current_fg: Option<u8>,

    /// Index (0–15) de la couleur arrière-plan ANSI courante.
    /// `None` = arrière-plan par défaut du thème GTK.
    current_bg: Option<u8>,

    /// Couleur avant-plan étendue (xterm-256 ou truecolor 24 bits).
    /// Prioritaire sur `current_fg` si `Some`.
    current_fg_rgb: Option<(u8, u8, u8)>,

    /// Couleur arrière-plan étendue (xterm-256 ou truecolor 24 bits).
    /// Prioritaire sur `current_bg` si `Some`.
    current_bg_rgb: Option<(u8, u8, u8)>,

    /// Cache des 16 tags de couleur avant-plan ANSI standard (`fg_0`…`fg_15`).
    /// Rempli une seule fois par [`AnsiPerformer::init_tags`] — aucun lookup de chaîne en chemin chaud.
    fg_tags: [Option<TextTag>; 16],

    /// Cache des 16 tags de couleur arrière-plan ANSI standard (`bg_0`…`bg_15`).
    /// Rempli une seule fois par [`AnsiPerformer::init_tags`].
    bg_tags: [Option<TextTag>; 16],

    /// Cache paresseux des tags de couleur avant-plan étendus.
    /// Clé : `0x00RRGGBB` encodé en `u32` — évite une allocation de `String` par lookup.
    fg_ext_tags: HashMap<u32, TextTag>,

    /// Cache paresseux des tags de couleur arrière-plan étendus.
    bg_ext_tags: HashMap<u32, TextTag>,

    /// Tag GTK pour le gras (`weight = 700`).
    bold_tag: Option<TextTag>,

    /// Tag GTK pour l'italique (`style = Italic`).
    italic_tag: Option<TextTag>,

    /// Tag GTK pour le soulignement (`underline = Single`).
    underline_tag: Option<TextTag>,

    /// Buffer temporaire de tags réutilisé à chaque [`AnsiPerformer::flush`].
    /// Capacité initiale de 5 (fg + bg + bold + italic + underline) — évite toute réallocation courante.
    tag_buf: Vec<TextTag>,

    /// Capture texte brut pour l'export horodaté fidèle.
    timestamped_log: Rc<RefCell<TimestampedLogRecorder>>,

    /// Position du curseur d'insertion dans le buffer GTK.
    ///
    /// Persistant entre les appels `append_ansi()` : contrairement à l'ancienne
    /// approche `cursor_in_pending` (remise à 0 à chaque `flush()`), ce mark
    /// survit aux limites de chunk réseau.
    /// Gravité droite (`left_gravity = false`) : le mark avance après chaque
    /// insertion, ce qui correspond au comportement naturel d'un curseur de texte.
    cursor_mark: TextMark,
}

impl AnsiPerformer {
    /// Crée un nouveau `AnsiPerformer` associé au buffer donné.
    ///
    /// Tous les attributs sont à leurs valeurs par défaut : aucune couleur active,
    /// aucun style actif. [`AnsiPerformer::init_tags`] **doit** être appelé avant toute utilisation.
    pub(super) fn new(
        buffer: TextBuffer,
        timestamped_log: Rc<RefCell<TimestampedLogRecorder>>,
    ) -> Self {
        // Créer le mark de curseur en fin de buffer vide (left_gravity=false → gravité droite).
        // Doit être fait AVANT de déplacer `buffer` dans le struct.
        let cursor_mark = buffer.create_mark(None, &buffer.end_iter(), false);
        Self {
            buffer,
            pending_text: String::with_capacity(256), // capacité initiale confortable pour éviter les réallocations fréquentes
            flags: 0,
            current_fg: None,     // pas de couleur avant-plan : thème GTK par défaut
            current_bg: None,     // pas de couleur arrière-plan
            current_fg_rgb: None, // pas de couleur étendue avant-plan
            current_bg_rgb: None, // pas de couleur étendue arrière-plan
            fg_tags: std::array::from_fn(|_| None), // tableau de 16 Options vides (rempli par init_tags)
            bg_tags: std::array::from_fn(|_| None), // idem pour l'arrière-plan
            fg_ext_tags: HashMap::new(),            // cache étendu avant-plan vide
            bg_ext_tags: HashMap::new(),            // cache étendu arrière-plan vide
            bold_tag: None,                         // résolu par init_tags
            italic_tag: None,                       // résolu par init_tags
            underline_tag: None,                    // résolu par init_tags
            tag_buf: Vec::with_capacity(5), // 5 = nombre max de tags simultanés (fg+bg+bold+italic+underline)
            timestamped_log,
            cursor_mark,
        }
    }

    const fn has_flag(&self, flag: u8) -> bool {
        self.flags & flag != 0
    }

    fn set_flag(&mut self, flag: u8, enabled: bool) {
        if enabled {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }

    /// Résout tous les [`TextTag`] depuis la table et les met en cache.
    ///
    /// À appeler **une seule fois** après construction, avant tout affichage.
    /// Les tags `fg_0`…`fg_15`, `bg_0`…`bg_15`, `bold`, `italic`, `underline`
    /// doivent exister dans `tag_table` (créés dans [`crate::ui::terminal_panel::TerminalPanel::new`]).
    ///
    /// Après cet appel, **aucun lookup de chaîne** n'est effectué en chemin chaud.
    #[allow(clippy::indexing_slicing)] // indices 0..16 garantis dans un tableau [T; 16]
    pub(super) fn init_tags(&mut self, tag_table: &TextTagTable) {
        for i in 0u8..16 {
            self.fg_tags[usize::from(i)] = tag_table.lookup(&format!("fg_{i}")); // tag avant-plan couleur ANSI i
            self.bg_tags[usize::from(i)] = tag_table.lookup(&format!("bg_{i}"));
            // tag arrière-plan couleur ANSI i
        }
        self.bold_tag = tag_table.lookup("bold"); // tag gras
        self.italic_tag = tag_table.lookup("italic"); // tag italique
        self.underline_tag = tag_table.lookup("underline"); // tag souligné
    }

    /// Réinitialise l'état complet entre deux sessions.
    ///
    /// Vide le texte en attente et remet tous les attributs à leur valeur par défaut.
    /// À appeler au début de chaque nouvelle connexion pour éviter que l'état partiel
    /// d'une session précédente (séquence ANSI tronquée, couleur restante) ne
    /// contamine la suivante.
    pub(super) fn reset_session(&mut self) {
        self.pending_text.clear();
        self.reset_attrs();
        // Replacer le curseur en fin de buffer pour la nouvelle session.
        let end = self.buffer.end_iter();
        self.buffer.move_mark(&self.cursor_mark, &end);
    }

    /// Réinitialise tous les attributs de style au défaut terminal (SGR 0).
    ///
    /// Appelé lors d'un code SGR `0` (reset complet) ou d'une séquence `\e[m`.
    fn reset_attrs(&mut self) {
        self.current_fg = None; // supprime la couleur avant-plan ANSI
        self.current_bg = None; // supprime la couleur arrière-plan ANSI
        self.current_fg_rgb = None; // supprime la couleur étendue avant-plan
        self.current_bg_rgb = None; // supprime la couleur étendue arrière-plan
        self.flags &= !STYLE_FLAGS_MASK; // désactive les styles ANSI sans toucher à l'état CR/LF
    }

    /// Obtient ou crée un [`TextTag`] de couleur **avant-plan** pour un triplet RGB.
    ///
    /// La clé `u32 = 0x00RRGGBB` permet un lookup O(1) sans allocation de `String`.
    /// Le tag est créé une seule fois, ajouté à la `TextTagTable` du buffer, puis mis en cache.
    fn get_or_create_fg_rgb_tag(&mut self, r: u8, g: u8, b: u8) -> Option<TextTag> {
        let key = (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b); // paquetage RGB → clé u32

        if !self.fg_ext_tags.contains_key(&key) {
            // Premier accès à cette couleur : création du tag GTK avant-plan
            let color = format!("#{r:02X}{g:02X}{b:02X}"); // représentation CSS hexadécimale
            let tag = gtk4::TextTag::builder()
                .name(format!("fg_rgb_{r:02x}{g:02x}{b:02x}")) // nom unique (utilisable pour debug)
                .foreground(&color) // couleur du texte
                .build();
            self.buffer.tag_table().add(&tag); // enregistrement dans la TextTagTable
            self.fg_ext_tags.insert(key, tag); // mise en cache pour les prochains appels
        }

        self.fg_ext_tags.get(&key).cloned() // clone GObject ref-counted — coût O(1)
    }

    /// Obtient ou crée un [`TextTag`] de couleur **arrière-plan** pour un triplet RGB.
    ///
    /// Même logique que [`AnsiPerformer::get_or_create_fg_rgb_tag`] mais applique `background`
    /// au lieu de `foreground`.
    fn get_or_create_bg_rgb_tag(&mut self, r: u8, g: u8, b: u8) -> Option<TextTag> {
        let key = (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b); // paquetage RGB → clé u32

        if !self.bg_ext_tags.contains_key(&key) {
            // Premier accès à cette couleur : création du tag GTK arrière-plan
            let color = format!("#{r:02X}{g:02X}{b:02X}"); // représentation CSS hexadécimale
            let tag = gtk4::TextTag::builder()
                .name(format!("bg_rgb_{r:02x}{g:02x}{b:02x}")) // nom unique (utilisable pour debug)
                .background(&color) // couleur de fond
                .build();
            self.buffer.tag_table().add(&tag); // enregistrement dans la TextTagTable
            self.bg_ext_tags.insert(key, tag); // mise en cache
        }

        self.bg_ext_tags.get(&key).cloned() // clone GObject ref-counted
    }

    /// Décode une couleur étendue (SGR 38 ou 48) depuis une liste aplatie de paramètres.
    ///
    /// ## Notations supportées
    /// - **Point-virgule** `38;5;n`   / `38;2;r;g;b` → groupes individuels dans `flat`.
    /// - **Deux-points**   `38:5:n`   / `38:2:r:g:b` → sous-paramètres déjà aplatis.
    ///
    /// ## Paramètres
    /// - `flat` : liste aplatie des paramètres SGR (ex. `[38, 5, 196, 0, 1]`).
    /// - `i`    : index de l'élément `38` ou `48` dans `flat`.
    ///
    /// ## Retour
    /// `Some((ExtColor, n_consommés))` si la séquence est valide, `None` sinon.
    /// `n_consommés` vaut 3 pour le mode 256-couleurs, 5 pour le truecolor.
    fn parse_color_ext(flat: &[u16], i: usize) -> Option<(ExtColor, usize)> {
        // flat[i+1] contient le mode : 5 = 256 couleurs, 2 = truecolor
        let mode = flat.get(i.saturating_add(1)).copied()?; // None si la séquence est tronquée

        match mode {
            5 => {
                // Mode 256 couleurs : format `38 ; 5 ; idx` — 3 éléments consommés
                let raw = flat.get(i.saturating_add(2)).copied()?; // index couleur 0–255
                let idx = u8::try_from(raw).unwrap_or(255); // conversion sûre u16 → u8 (255 si hors plage)

                if idx < 16 {
                    // Index 0–15 : couleur ANSI standard → cache rapide
                    Some((ExtColor::Ansi(idx), 3))
                } else {
                    // Index 16–255 : cube xterm ou rampe gris → conversion RGB
                    let (r, g, b) = xterm256_color(idx);
                    Some((ExtColor::Rgb(r, g, b), 3))
                }
            }
            2 => {
                // Mode truecolor : format `38 ; 2 ; r ; g ; b` — 5 éléments consommés
                let r = u8::try_from(flat.get(i.saturating_add(2)).copied()?).unwrap_or(0); // composante rouge   (0–255)
                let g = u8::try_from(flat.get(i.saturating_add(3)).copied()?).unwrap_or(0); // composante verte   (0–255)
                let b = u8::try_from(flat.get(i.saturating_add(4)).copied()?).unwrap_or(0); // composante bleue   (0–255)
                Some((ExtColor::Rgb(r, g, b), 5))
            }
            _ => None, // mode inconnu (ni 2 ni 5) : séquence ignorée silencieusement
        }
    }

    /// Gère un `\r` bare (retour chariot sans saut de ligne qui suit).
    ///
    /// Appelé en début de `print()`, `execute('\t')`, `execute('\x08')` et
    /// `csi_dispatch()` : si `FLAG_LAST_WAS_CR` est armé, cela signifie qu'un `\r`
    /// non suivi de `\n` vient d'être reçu (ex. readline qui réécrit le prompt après
    /// un `SIGWINCH`). On repositionne alors `cursor_mark` en début de la ligne
    /// courante et on efface son contenu, de sorte que le prochain texte inséré
    /// écrase l'ancienne ligne au lieu de s'y accoler.
    ///
    /// Ne fait rien si `FLAG_LAST_WAS_CR` n'est pas actif (appel no-op dans le cas commun).
    fn apply_pending_cr_if_any(&mut self) {
        if !self.has_flag(FLAG_LAST_WAS_CR) {
            return;
        }
        self.set_flag(FLAG_LAST_WAS_CR, false);

        // Trouver le début de la ligne courante.
        let cursor_iter = self.buffer.iter_at_mark(&self.cursor_mark);
        let mut line_start = cursor_iter;
        line_start.set_line_offset(0);

        // Effacer du début de ligne jusqu'à la fin de ligne (sans supprimer le \n).
        if !line_start.ends_line() {
            let mut line_end = line_start; // TextIter est Copy
            line_end.forward_to_line_end();
            self.buffer.delete(&mut line_start, &mut line_end);
            // Après suppression, line_start pointe à l'emplacement effacé.
        }
        // Repositionner cursor_mark en début de la ligne (maintenant vide).
        self.buffer.move_mark(&self.cursor_mark, &line_start);
    }

    /// Insère le texte en attente dans le buffer GTK avec les tags de style actifs.
    ///
    /// Collecte tous les tags actifs dans `tag_buf` puis effectue une **seule** insertion
    /// GTK — minimise les appels à `insert_with_tags`. Vide `pending_text` après l'insertion.
    ///
    /// Si `pending_text` est vide, retourne immédiatement sans aucune opération GTK.
    #[allow(clippy::indexing_slicing)] // indices 0..16 garantis dans un tableau [T; 16]
    pub(super) fn flush(&mut self) {
        if self.pending_text.is_empty() {
            return; // rien à insérer : sortie rapide sans aucun accès GTK
        }

        self.tag_buf.clear(); // réinitialise le buffer de tags sans réallocation

        // ── Couleur avant-plan ─────────────────────────────────────────────────
        if let Some((r, g, b)) = self.current_fg_rgb {
            // Couleur étendue RGB prioritaire sur la couleur ANSI standard
            if let Some(tag) = self.get_or_create_fg_rgb_tag(r, g, b) {
                self.tag_buf.push(tag); // ajout du tag avant-plan RGB
            }
        } else if let Some(idx) = self.current_fg {
            // Couleur ANSI standard (0–15) : accès direct au tableau sans lookup de chaîne
            if let Some(ref t) = self.fg_tags[usize::from(idx)] {
                self.tag_buf.push(t.clone()); // clone GObject ref-counted — O(1)
            }
        }
        // Si ni RGB ni ANSI : aucun tag avant-plan → couleur par défaut du thème

        // ── Couleur arrière-plan ───────────────────────────────────────────────
        if let Some((r, g, b)) = self.current_bg_rgb {
            // Couleur étendue RGB arrière-plan prioritaire
            if let Some(tag) = self.get_or_create_bg_rgb_tag(r, g, b) {
                self.tag_buf.push(tag); // ajout du tag arrière-plan RGB
            }
        } else if let Some(idx) = self.current_bg {
            // Couleur ANSI standard arrière-plan (0–15)
            if let Some(ref t) = self.bg_tags[usize::from(idx)] {
                self.tag_buf.push(t.clone()); // clone GObject ref-counted
            }
        }

        // ── Attributs de style ─────────────────────────────────────────────────
        if self.has_flag(FLAG_BOLD) {
            if let Some(ref t) = self.bold_tag {
                self.tag_buf.push(t.clone()); // tag gras actif
            }
        }
        if self.has_flag(FLAG_ITALIC) {
            if let Some(ref t) = self.italic_tag {
                self.tag_buf.push(t.clone()); // tag italique actif
            }
        }
        if self.has_flag(FLAG_UNDERLINE) {
            if let Some(ref t) = self.underline_tag {
                self.tag_buf.push(t.clone()); // tag soulignement actif
            }
        }

        // ── Mode remplacement (terminal overwrite) ────────────────────────────
        // Un vrai terminal écrit chaque caractère dans une cellule fixe de la
        // grille : le nouveau glyphe remplace celui qui y était.  GTK TextBuffer
        // est en mode insertion par défaut, ce qui doublerait les caractères
        // lorsque readline renvoie la fin de la ligne après un déplacement de
        // curseur.  On supprime donc d'abord, sur la ligne courante, autant de
        // caractères qu'on va en insérer (arrêt au saut de ligne ou à la fin).
        {
            let cursor_iter = self.buffer.iter_at_mark(&self.cursor_mark);
            if !cursor_iter.ends_line() {
                let overwrite_count = self.pending_text.chars().take_while(|&c| c != '\n').count();
                if overwrite_count > 0 {
                    let mut del_start = cursor_iter; // TextIter est Copy
                    let mut del_end = cursor_iter;
                    for _ in 0..overwrite_count {
                        if del_end.ends_line() {
                            break;
                        }
                        del_end.forward_char();
                    }
                    if del_start != del_end {
                        self.buffer.delete(&mut del_start, &mut del_end);
                        // cursor_mark (gravité droite) retombe sur la position P
                        // de la suppression — prêt pour l'insertion ci-dessous.
                    }
                }
            }
        }

        // ── Insertion GTK à la position du curseur ────────────────────────────
        // On insère au cursor_mark (pas forcément en fin de buffer) pour que les
        // déplacements de curseur (CSI D/C) effectués entre deux chunks réseau
        // soient honorés même après un flush intermédiaire.
        // Gravité droite : cursor_mark avance automatiquement après l'insertion.
        let mut cursor_iter = self.buffer.iter_at_mark(&self.cursor_mark);

        if self.tag_buf.is_empty() {
            // Aucun style actif : chemin rapide sans construction de tableau de références
            self.buffer.insert(&mut cursor_iter, &self.pending_text);
        } else {
            // Au moins un style : construction du slice de références `&[&TextTag]` pour GTK
            let refs: Vec<&TextTag> = self.tag_buf.iter().collect();
            self.buffer
                .insert_with_tags(&mut cursor_iter, &self.pending_text, &refs);
        }

        self.timestamped_log
            .borrow_mut()
            .append_fragment(LogLineKind::Rx, &self.pending_text);

        self.pending_text.clear(); // vide le texte en attente (conserve la capacité allouée pour la prochaine trame)
                                   // cursor_mark avancé automatiquement par GTK (gravité droite) — pas besoin de le déplacer manuellement
    }
}

// =============================================================================
// Implémentation du trait vte::Perform — callbacks du parseur VTE
// =============================================================================
//
// Seuls trois callbacks sont implémentés (les autres ont une implémentation par
// défaut no-op dans le trait) :
//
// - `print`       : caractère imprimable Unicode → accumulé dans `pending_text`.
// - `execute`     : octet de contrôle C0 (LF, CR, TAB, BS) → ajouté à `pending_text`.
// - `csi_dispatch`: séquence CSI (couleurs SGR, etc.) → interprétée et flush déclenché.
//
// Les autres séquences (OSC, DCS, APC, ESC seul, séquences de curseur CUP/CUU/CUD…)
// sont volontairement ignorées : l'affichage est en lecture seule, sans curseur mobile.
// Ajouter le support du curseur nécessiterait un modèle complet de grille de cellules
// (hors périmètre pour un panneau de log/terminal passif).

/// Extrait le premier paramètre CSI (entier non signé), ou `default` si absent/nul.
///
/// Conforme à VT100 : un paramètre absent ou égal à 0 vaut la valeur par défaut.
fn first_csi_param(params: &vte::Params, default: usize) -> usize {
    params
        .iter()
        .next()
        .and_then(|g| g.first().copied())
        .map_or(default, |v| if v == 0 { default } else { usize::from(v) })
}

impl Perform for AnsiPerformer {
    /// Reçoit un caractère imprimable et l'accumule dans `pending_text`.
    ///
    /// L'accumulation différée permet de regrouper tous les caractères consécutifs
    /// de même style en une seule insertion GTK lors du prochain [`AnsiPerformer::flush`].
    fn print(&mut self, c: char) {
        // Un \r bare (non suivi de \n) signale un retour chariot sans saut de ligne :
        // readline l'utilise pour réécrire le prompt en place (ex. après SIGWINCH).
        // On repositionne cursor_mark en début de ligne et on efface la ligne courante
        // pour que le prochain texte écrase l'ancien au lieu de s'y accoler.
        self.apply_pending_cr_if_any();
        // Accumulation simple : cursor_mark (dans le buffer GTK) gère la position.
        // À flush(), le texte sera inséré à la position courante de cursor_mark.
        self.pending_text.push(c);
    }

    /// Reçoit un octet de contrôle ASCII (C0, plage 0x00–0x1F).
    ///
    /// Seuls les quatre octets courants d'un terminal interactif sont transmis :
    /// - `\n` (0x0A) : saut de ligne (LF),
    /// - `\r` (0x0D) : retour chariot (CR),
    /// - `\t` (0x09) : tabulation horizontale,
    /// - `\x08` (0x08) : backspace.
    ///
    /// Tous les autres octets de contrôle sont ignorés silencieusement.
    fn execute(&mut self, byte: u8) {
        match byte {
            b'\r' => {
                // Retour chariot : flush le texte en attente et arme FLAG_LAST_WAS_CR.
                // On ne sait pas encore si \n suit (CRLF) ou non (bare \r).
                // - CRLF (\r\n) : le \n dans la branche suivante pousse '\n' normalement.
                // - bare \r     : le prochain print() / csi_dispatch() appelle
                //                 apply_pending_cr_if_any() qui repositionne cursor_mark
                //                 en début de ligne et efface la ligne courante.
                self.flush();
                self.set_flag(FLAG_LAST_WAS_CR, true);
            }
            b'\n' => {
                if self.has_flag(FLAG_LAST_WAS_CR) {
                    // CRLF : le \r a déjà flushé le texte précédent.
                    // Ajouter le saut de ligne pour descendre à la ligne suivante.
                    self.set_flag(FLAG_LAST_WAS_CR, false);
                    self.pending_text.push('\n');
                    return;
                }
                self.pending_text.push('\n');
            }
            b'\t' => {
                self.apply_pending_cr_if_any();
                self.pending_text.push('\t');
            }
            b'\x08' => {
                // BS : déplacer le curseur d'une position à gauche (VT100).
                // Le serveur envoie \x08\x20\x08 (BS+espace+BS) pour l'écho ECHOE :
                //   - BS  → curseur gauche,
                //   - ' ' → écrire espace (efface visuellement le caractère),
                //   - BS  → curseur gauche à nouveau.
                // Résultat net : le caractère à gauche est supprimé de l'affichage.
                self.apply_pending_cr_if_any();
                if self.pending_text.is_empty() {
                    // Caractère déjà dans le buffer GTK : supprimer le char à gauche de cursor_mark
                    let mut start = self.buffer.iter_at_mark(&self.cursor_mark);
                    if start.backward_char() {
                        // start pointe maintenant sur le char à supprimer
                        let mut end = start; // TextIter est Copy
                        end.forward_char(); // end = position originale de cursor_mark
                        self.buffer.delete(&mut start, &mut end);
                        // cursor_mark (gravité droite) se repositionne automatiquement
                    }
                } else {
                    // Caractère pas encore flushé : retrait simple depuis pending_text
                    self.pending_text.pop();
                }
            }
            _ => {} // octet de contrôle non géré (ex. BEL, ESC nu) : ignoré
        }
    }

    /// Début d'une séquence DCS (Device Control String) — non utilisée, ignorée.
    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {
    }

    /// Octet reçu à l'intérieur d'une séquence DCS — non utilisé, ignoré.
    fn put(&mut self, _byte: u8) {}

    /// Fin d'une séquence DCS — non utilisée, ignorée.
    fn unhook(&mut self) {}

    /// Séquence OSC (Operating System Command, ex. titre de fenêtre) — ignorée.
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    /// Séquence CSI (Control Sequence Introducer) — gère uniquement SGR (`m`).
    ///
    /// **SGR** (Select Graphic Rendition, `\e[...m`) contrôle :
    /// couleurs avant/arrière-plan (ANSI 16, xterm-256, truecolor), gras, italique,
    /// souligné, et leur réinitialisation.
    ///
    /// Toute séquence CSI avec `action ≠ 'm'` est ignorée silencieusement.
    #[allow(clippy::too_many_lines)] // dispatch SGR complet : longueur inhérente au protocole VT100
    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        match action {
            // ── Déplacement curseur dans le buffer GTK ────────────────────────
            'D' => {
                // CSI n D : curseur gauche de n positions (défaut 1).
                // Flush d'abord pour que cursor_mark soit après le texte en attente,
                // puis déplacer le mark en arrière dans le buffer GTK.
                let n = first_csi_param(params, 1);
                self.apply_pending_cr_if_any();
                self.flush();
                let mut iter = self.buffer.iter_at_mark(&self.cursor_mark);
                iter.backward_chars(i32::try_from(n).unwrap_or(i32::MAX));
                self.buffer.move_mark(&self.cursor_mark, &iter);
                return;
            }
            'C' => {
                // CSI n C : curseur droite de n positions (défaut 1).
                let n = first_csi_param(params, 1);
                self.apply_pending_cr_if_any();
                self.flush();
                let mut iter = self.buffer.iter_at_mark(&self.cursor_mark);
                iter.forward_chars(i32::try_from(n).unwrap_or(i32::MAX));
                self.buffer.move_mark(&self.cursor_mark, &iter);
                return;
            }
            'K' => {
                // CSI K (ou CSI 0 K) : effacer du curseur jusqu'à la fin de la ligne courante.
                self.apply_pending_cr_if_any();
                self.flush();
                let mut start = self.buffer.iter_at_mark(&self.cursor_mark);
                if !start.ends_line() {
                    let mut end = start; // TextIter est Copy
                    end.forward_to_line_end();
                    self.buffer.delete(&mut start, &mut end);
                    // cursor_mark reste à sa position (début de la zone effacée)
                }
                return;
            }
            'm' => {}    // SGR : traité ci-dessous après le flush
            _ => return, // autres séquences CSI (déplacement absolu, etc.) : ignorées
        }

        // Flush obligatoire avant changement de style :
        // le texte déjà accumulé doit être inséré avec le STYLE PRÉCÉDENT.
        self.flush();

        // ── Aplatissement des paramètres SGR ──────────────────────────────────
        // VTE produit deux types de groupes selon la notation ANSI utilisée :
        //
        //   Notation ';' (la plus courante) :
        //     `\e[38;5;196m` → groupes [38][5][196]     → flat [38, 5, 196]
        //
        //   Notation ':' (sous-paramètres dans un groupe) :
        //     `\e[38:5:196m` → groupe  [38, 5, 196]     → flat [38, 5, 196]
        //     `\e[38:2:R:G:Bm` → groupe [38, 2, R, G, B] → flat [38, 2, R, G, B]
        //
        // On construit une liste plate `flat` qui unifie les deux notations
        // afin que parse_color_ext puisse les traiter de façon identique.
        let mut flat: Vec<u16> = Vec::with_capacity(8); // 8 = capacité confortable (max 5 pour truecolor)
        let mut has_params = false; // vrai si au moins un groupe est présent

        for group in params {
            has_params = true; // au moins un groupe détecté

            if group.len() > 1 {
                // Notation ':' : le groupe contient plusieurs sous-paramètres déjà regroupés
                flat.extend_from_slice(group);
            } else {
                // Notation ';' : groupe d'un seul élément — 0 si le groupe est vide
                flat.push(group.first().copied().unwrap_or(0));
            }
        }

        if !has_params {
            // Séquence `\e[m` sans paramètre : équivalent à SGR 0 (reset complet)
            self.reset_attrs();
            return;
        }

        // ── Traitement séquentiel des paramètres aplatis ──────────────────────
        let mut i = 0usize; // index courant dans `flat`

        while let Some(&p) = flat.get(i) {
            match p {
                0 => self.reset_attrs(),             // SGR 0  : reset de tous les attributs
                1 => self.set_flag(FLAG_BOLD, true), // SGR 1  : activer le gras
                3 => self.set_flag(FLAG_ITALIC, true), // SGR 3  : activer l'italique
                4 => self.set_flag(FLAG_UNDERLINE, true), // SGR 4  : activer le soulignement
                22 => self.set_flag(FLAG_BOLD, false), // SGR 22 : désactiver le gras (bold off)
                23 => self.set_flag(FLAG_ITALIC, false), // SGR 23 : désactiver l'italique
                24 => self.set_flag(FLAG_UNDERLINE, false), // SGR 24 : désactiver le soulignement

                // ── Couleur avant-plan étendue : SGR 38 ───────────────────────
                38 => {
                    if let Some((color, skip)) = Self::parse_color_ext(&flat, i) {
                        match color {
                            ExtColor::Ansi(idx) => {
                                self.current_fg = Some(idx); // couleur ANSI 0–15 : cache rapide
                                self.current_fg_rgb = None; // efface toute couleur RGB précédente
                            }
                            ExtColor::Rgb(r, g, b) => {
                                self.current_fg_rgb = Some((r, g, b)); // couleur RGB 24 bits
                                self.current_fg = None; // efface l'index ANSI précédent
                            }
                        }
                        i = i.saturating_add(skip); // saute les `skip` éléments consommés par parse_color_ext
                        continue; // reprend la boucle sans l'incrément de fin
                    }
                    // parse_color_ext a retourné None : séquence malformée, ignorée
                }

                // ── Couleur arrière-plan étendue : SGR 48 ─────────────────────
                48 => {
                    if let Some((color, skip)) = Self::parse_color_ext(&flat, i) {
                        match color {
                            ExtColor::Ansi(idx) => {
                                self.current_bg = Some(idx); // couleur ANSI 0–15 arrière-plan
                                self.current_bg_rgb = None; // efface toute couleur RGB précédente
                            }
                            ExtColor::Rgb(r, g, b) => {
                                self.current_bg_rgb = Some((r, g, b)); // couleur RGB arrière-plan
                                self.current_bg = None; // efface l'index ANSI précédent
                            }
                        }
                        i = i.saturating_add(skip); // saute les éléments consommés
                        continue;
                    }
                    // Séquence malformée : ignorée
                }

                // SGR 39 : réinitialiser la couleur avant-plan (retour au défaut du thème)
                39 => {
                    self.current_fg = None; // efface l'index ANSI avant-plan
                    self.current_fg_rgb = None; // efface la couleur RGB avant-plan
                }

                // SGR 49 : réinitialiser la couleur arrière-plan (retour au défaut du thème)
                49 => {
                    self.current_bg = None; // efface l'index ANSI arrière-plan
                    self.current_bg_rgb = None; // efface la couleur RGB arrière-plan
                }

                // SGR 30–37 : couleurs avant-plan ANSI normales (noir … blanc clair)
                30..=37 => {
                    self.current_fg = Some(u8::try_from(p.saturating_sub(30)).unwrap_or(0)); // indice 0..7
                    self.current_fg_rgb = None; // efface toute couleur étendue précédente
                }

                // SGR 40–47 : couleurs arrière-plan ANSI normales
                40..=47 => {
                    self.current_bg = Some(u8::try_from(p.saturating_sub(40)).unwrap_or(0)); // indice 0..7
                    self.current_bg_rgb = None; // efface toute couleur étendue précédente
                }

                // SGR 90–97 : couleurs avant-plan brillantes (bright, indices 8–15 dans la palette)
                90..=97 => {
                    // bright = ANSI + 8 : ex. bright noir (90) → index 8, bright blanc (97) → index 15
                    self.current_fg =
                        Some(u8::try_from(p.saturating_sub(90).saturating_add(8)).unwrap_or(8));
                    self.current_fg_rgb = None; // efface toute couleur étendue précédente
                }

                // SGR 100–107 : couleurs arrière-plan brillantes (bright, indices 8–15)
                100..=107 => {
                    // bright arrière-plan : p - 100 + 8
                    self.current_bg =
                        Some(u8::try_from(p.saturating_sub(100).saturating_add(8)).unwrap_or(8));
                    self.current_bg_rgb = None; // efface toute couleur étendue précédente
                }

                _ => {} // paramètre SGR inconnu ou non implémenté : ignoré silencieusement
            }

            i = i.saturating_add(1); // passage au paramètre SGR suivant
        }
    }

    /// Séquence ESC simple (hors CSI/OSC/DCS) — non utilisée, ignorée.
    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

// =============================================================================
// Tests unitaires — logique pure sans dépendance GTK
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_timestamped_text_preserves_device_payload_prefix() {
        let mut recorder = TimestampedLogRecorder::default();

        recorder.completed_lines.push_back(TimestampedLine {
            kind: LogLineKind::Rx,
            timestamp: "2026-03-25 04:52:03".to_string(),
            text: "[04:51:16] Bring-up 1/3 : initialisation backend STM32\n".to_string(),
        });

        let exported = recorder.export_timestamped_session_text();

        assert!(exported.contains(
            "[2026-03-25 04:52:03] [04:51:16] Bring-up 1/3 : initialisation backend STM32"
        ));
    }

    #[test]
    fn export_raw_text_keeps_payload_without_adding_local_timestamp_prefix() {
        let mut recorder = TimestampedLogRecorder::default();

        recorder.completed_lines.push_back(TimestampedLine {
            kind: LogLineKind::System,
            timestamp: "2026-03-25 05:33:05".to_string(),
            text: "[05:33:05] Terminal efface.\n".to_string(),
        });
        recorder.completed_lines.push_back(TimestampedLine {
            kind: LogLineKind::Tx,
            timestamp: "2026-03-25 05:42:04".to_string(),
            text: "==== Commande RAW : AT ====\n".to_string(),
        });

        let exported = recorder.export_raw_text();

        assert_eq!(exported, "==== Commande RAW : AT ====\n");
        assert!(!exported.contains("2026-03-25 05:33:05"));
    }

    #[test]
    fn export_raw_text_preserves_device_payload_starting_with_short_timestamp() {
        let mut recorder = TimestampedLogRecorder::default();

        recorder.completed_lines.push_back(TimestampedLine {
            kind: LogLineKind::Rx,
            timestamp: "2026-03-25 05:42:04".to_string(),
            text: "[05:41:03] measured temperature=24.1C\n".to_string(),
        });

        let exported = recorder.export_raw_text();

        assert_eq!(exported, "[05:41:03] measured temperature=24.1C\n");
    }

    #[test]
    fn export_split_text_separates_rx_tx_and_system_sections() {
        let mut recorder = TimestampedLogRecorder::default();

        recorder.completed_lines.push_back(TimestampedLine {
            kind: LogLineKind::Rx,
            timestamp: "2026-03-25 05:42:04".to_string(),
            text: "OK\n".to_string(),
        });
        recorder.completed_lines.push_back(TimestampedLine {
            kind: LogLineKind::Tx,
            timestamp: "2026-03-25 05:42:05".to_string(),
            text: "→ AT\n".to_string(),
        });
        recorder.completed_lines.push_back(TimestampedLine {
            kind: LogLineKind::System,
            timestamp: "2026-03-25 05:42:06".to_string(),
            text: "Terminal efface.\n".to_string(),
        });

        let exported = recorder.export_split_text();

        assert!(exported.contains("=== RX ==="));
        assert!(exported.contains("=== TX ==="));
        assert!(exported.contains("=== SYSTEM ==="));
        assert!(exported.contains("[2026-03-25 05:42:04] OK"));
        assert!(exported.contains("[2026-03-25 05:42:05] → AT"));
        assert!(exported.contains("[2026-03-25 05:42:06] Terminal efface."));
    }

    struct NoopPerformer;

    impl Perform for NoopPerformer {
        fn print(&mut self, _c: char) {}
    }

    // ── xterm256_color ────────────────────────────────────────────────────────

    #[test]
    fn xterm256_first_cube_entry() {
        // Index 16 = premier élément du cube 6×6×6 → (0, 0, 0)
        assert_eq!(xterm256_color(16), (0, 0, 0));
    }

    #[test]
    fn xterm256_pure_red() {
        // Index 196 = (5,0,0) dans le cube → RGB(255, 0, 0)
        assert_eq!(xterm256_color(196), (255, 0, 0));
    }

    #[test]
    fn xterm256_pure_green() {
        // Index 46 = (0,5,0) → RGB(0, 255, 0)
        assert_eq!(xterm256_color(46), (0, 255, 0));
    }

    #[test]
    fn xterm256_pure_blue() {
        // Index 21 = (0,0,5) → RGB(0, 0, 255)
        assert_eq!(xterm256_color(21), (0, 0, 255));
    }

    #[test]
    fn xterm256_last_cube_entry() {
        // Index 231 = (5,5,5) → RGB(255, 255, 255)
        assert_eq!(xterm256_color(231), (255, 255, 255));
    }

    #[test]
    fn xterm256_grayscale_first() {
        // Index 232 → niveau 8 (le plus sombre de la rampe gris)
        assert_eq!(xterm256_color(232), (8, 8, 8));
    }

    #[test]
    fn xterm256_grayscale_last() {
        // Index 255 → niveau 238 (le plus clair de la rampe)
        assert_eq!(xterm256_color(255), (238, 238, 238));
    }

    #[test]
    fn xterm256_out_of_range_returns_black() {
        // Index 0–15 : renvoie (0,0,0) — géré par le cache rapide en pratique
        assert_eq!(xterm256_color(0), (0, 0, 0));
        assert_eq!(xterm256_color(15), (0, 0, 0));
    }

    // ── parse_color_ext ───────────────────────────────────────────────────────

    #[test]
    fn parse_color_ext_256_index_lt16_returns_ansi() {
        // flat = [38, 5, 7] → couleur ANSI 7 (gris clair)
        let flat: &[u16] = &[38, 5, 7];
        let result = AnsiPerformer::parse_color_ext(flat, 0);
        assert!(matches!(result, Some((ExtColor::Ansi(7), 3))));
    }

    #[test]
    fn parse_color_ext_256_index_ge16_returns_rgb() {
        // flat = [38, 5, 196] → rouge pur xterm-256
        let flat: &[u16] = &[38, 5, 196];
        let result = AnsiPerformer::parse_color_ext(flat, 0);
        assert!(matches!(result, Some((ExtColor::Rgb(255, 0, 0), 3))));
    }

    #[test]
    fn parse_color_ext_truecolor_returns_rgb() {
        // flat = [38, 2, 100, 150, 200] → RGB(100, 150, 200)
        let flat: &[u16] = &[38, 2, 100, 150, 200];
        let result = AnsiPerformer::parse_color_ext(flat, 0);
        assert!(matches!(result, Some((ExtColor::Rgb(100, 150, 200), 5))));
    }

    #[test]
    fn parse_color_ext_truecolor_offset_consumed() {
        let flat: &[u16] = &[0, 38, 2, 10, 20, 30];
        let result = AnsiPerformer::parse_color_ext(flat, 1);
        assert!(matches!(result, Some((ExtColor::Rgb(10, 20, 30), 5))));
    }

    #[test]
    fn parse_color_ext_malformed_mode_unknown_returns_none() {
        // Mode 9 inconnu → None
        let flat: &[u16] = &[38, 9, 1];
        let result = AnsiPerformer::parse_color_ext(flat, 0);
        assert!(result.is_none());
    }

    #[test]
    fn parse_color_ext_truncated_truecolor_returns_none() {
        // flat trop court pour le truecolor (manque B)
        let flat: &[u16] = &[38, 2, 100, 150];
        let result = AnsiPerformer::parse_color_ext(flat, 0);
        assert!(result.is_none());
    }

    #[test]
    fn parse_color_ext_truncated_256_returns_none() {
        // flat trop court pour le 256-color (manque idx)
        let flat: &[u16] = &[38, 5];
        let result = AnsiPerformer::parse_color_ext(flat, 0);
        assert!(result.is_none());
    }

    #[test]
    fn parse_color_ext_missing_mode_returns_none() {
        // flat d'un seul élément → pas de mode
        let flat: &[u16] = &[38];
        let result = AnsiPerformer::parse_color_ext(flat, 0);
        assert!(result.is_none());
    }

    #[test]
    fn vte_parser_accepts_truncated_escape_sequence_without_panicking() {
        let mut parser = Parser::new();
        let mut performer = NoopPerformer;

        parser.advance(&mut performer, b"\x1b[38;2;255");
        parser.advance(&mut performer, b"\x1b[0m");
    }
}
