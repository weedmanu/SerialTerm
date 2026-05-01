//! ############################################################################
//! @file    `log_viewer/model.rs`
//! @author  manu
//! @brief   Modèle de données du visualiseur de logs.
//!          Définit [`LogLevel`] (détection, badge, couleur, code, bit de filtre)
//!          et les fonctions d'encodage/décodage des lignes dans le format interne
//!          `"C|NNNNN|texte brut"` utilisé par la [`gtk4::StringList`].
//! @version    1.0.0
//! @date    2026-03-05
//! @copyright Libre sous licence MIT.
//! ############################################################################
//!
//! ## Format d'encodage des lignes
//!
//! Chaque ligne du terminal est encodée sous la forme :
//! ```text
//! "C|NNNNN|texte brut de la ligne"
//!  │  │     └─ contenu de la ligne (peut contenir des '|')
//!  │  └─ numéro de ligne sur 5 chiffres avec zéros de remplissage (00001…99999)
//!  └─ code de niveau sur 1 caractère : E W I D S N
//! ```
//!
//! Ce format compact est conçu pour être stocké dans une [`gtk4::StringList`]
//! et décodé rapidement dans la factory de la [`gtk4::ListView`].
//!
//! ## Masque de filtrage binaire
//!
//! Chaque niveau possède un bit unique (voir [`LogLevel::bit`]).
//! Le masque global (`level_mask: u8`) est un OR des bits actifs.
//! Un niveau est visible si `mask & level.bit() != 0`.

// =============================================================================
// Enum LogLevel
// =============================================================================

/// Niveau de sévérité d'une ligne de log.
///
/// Détecté automatiquement par mots-clés dans [`LogLevel::detect`].
/// Utilisé pour la coloration, le badge affiché et le filtrage par bit.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum LogLevel {
    /// Ligne contenant ERROR / ERREUR / FATAL / CRITICAL / CRITIQUE.
    Error,
    /// Ligne contenant WARN / AVERT.
    Warning,
    /// Ligne contenant INFO / [INFO.
    Info,
    /// Ligne contenant DEBUG / TRACE.
    Debug,
    /// Ligne système (commence par `[` et contient Système / [SYS).
    System,
    /// Aucun mot-clé reconnu — ligne ordinaire.
    Normal,
}

impl LogLevel {
    /// Détecte le niveau d'une ligne de texte par mots-clés (insensible à la casse).
    ///
    /// L'ordre de test détermine la priorité : `Error` est testé avant `Warning`,
    /// `Warning` avant `Info`, etc. Une ligne `FATAL ERROR` sera classée `Error`.
    ///
    /// # Paramètre
    /// - `line` : ligne de texte brute (peut contenir des codes ANSI résiduels).
    pub(super) fn detect(line: &str) -> Self {
        let up = line.to_ascii_uppercase(); // copie en majuscules pour la comparaison insensible à la casse

        // Niveau Error : mots-clés les plus graves testés en premier
        if up.contains("ERROR")
            || up.contains("ERREUR")   // français
            || up.contains("FATAL")
            || up.contains("CRITICAL")
            || up.contains("CRITIQUE")
        // français
        {
            return Self::Error;
        }

        // Niveau Warning : avertissements en anglais et en français
        if up.contains("WARN") || up.contains("AVERT") {
            return Self::Warning;
        }

        // Niveau Info : plusieurs formes courantes du mot-clé INFO
        if up.contains(" INFO ") || up.starts_with("INFO") || up.contains("[INFO") {
            return Self::Info;
        }

        // Niveau Debug : debug et trace traités au même niveau de sévérité
        if up.contains("DEBUG") || up.contains("TRACE") {
            return Self::Debug;
        }

        // Niveau System : lignes internes horodatées enregistrées côté terminal
        // Format attendu : `[HH:MM:SS] Système …` ou `[SYS] …`
        if line.starts_with('[')
            && (line.contains("Système") || line.contains("Systeme") || line.contains("[SYS"))
        {
            return Self::System;
        }

        // Aucun mot-clé reconnu : ligne normale sans niveau particulier
        Self::Normal
    }

    /// Retourne le code caractère unique représentant ce niveau.
    ///
    /// Utilisé comme premier octet du format d'encodage `"C|NNNNN|texte"`.
    /// Les codes sont : `E` (Error), `W` (Warning), `I` (Info),
    /// `D` (Debug), `S` (System), `N` (Normal).
    const fn code(self) -> char {
        match self {
            Self::Error => 'E',   // Error
            Self::Warning => 'W', // Warning
            Self::Info => 'I',    // Info
            Self::Debug => 'D',   // Debug
            Self::System => 'S',  // System
            Self::Normal => 'N',  // Normal
        }
    }

    /// Retourne le bit de filtrage unique associé à ce niveau.
    ///
    /// Chaque niveau occupe un bit distinct dans le masque `u8`.
    /// Le masque global est un OR des bits des niveaux actifs.
    /// Un niveau est visible si `mask & level.bit() != 0`.
    ///
    /// ```text
    /// bit 0 (0b00_0001) = Error
    /// bit 1 (0b00_0010) = Warning
    /// bit 2 (0b00_0100) = Info
    /// bit 3 (0b00_1000) = Debug
    /// bit 4 (0b01_0000) = System
    /// bit 5 (0b10_0000) = Normal
    /// ```
    const fn bit(self) -> u8 {
        match self {
            Self::Error => 0b00_0001,   // bit 0 : erreurs
            Self::Warning => 0b00_0010, // bit 1 : avertissements
            Self::Info => 0b00_0100,    // bit 2 : informations
            Self::Debug => 0b00_1000,   // bit 3 : debug / trace
            Self::System => 0b01_0000,  // bit 4 : messages système
            Self::Normal => 0b10_0000,  // bit 5 : lignes normales
        }
    }

    /// Retourne le badge texte à afficher dans la colonne de niveau.
    ///
    /// Tous les badges font exactement 4 caractères pour l'alignement dans la `ListView`.
    pub(super) const fn badge(self) -> &'static str {
        match self {
            Self::Error => "ERR ",   // 4 chars : espace de rembourrage à droite
            Self::Warning => "WARN", // 4 chars
            Self::Info => "INFO",    // 4 chars
            Self::Debug => "DBG ",   // 4 chars
            Self::System => "SYS ",  // 4 chars
            Self::Normal => " -- ",  // 4 chars : tirets pour les lignes sans niveau
        }
    }

    /// Retourne la couleur CSS avant-plan associée à ce niveau.
    ///
    /// Couleurs issues de la palette GNOME HIG pour la cohérence visuelle
    /// avec l'environnement GTK4/Libadwaita.
    pub(super) const fn fg_color(self) -> &'static str {
        match self {
            Self::Error => "#e01b24",   // rouge GNOME HIG (Red 5)
            Self::Warning => "#e66100", // orange GNOME HIG (Orange 5)
            Self::Info => "#1c71d8",    // bleu GNOME HIG (Blue 5)
            Self::Debug => "#9141ac",   // violet GNOME HIG (Purple 5)
            Self::System => "#26a269",  // vert GNOME HIG (Green 5)
            Self::Normal => "#aaaaaa",  // gris neutre pour les lignes sans niveau
        }
    }

    /// Reconstruit un `LogLevel` depuis son code caractère unique.
    ///
    /// Inverse de [`Self::code`]. Tout caractère inconnu retourne [`Self::Normal`].
    pub(super) const fn from_code(c: char) -> Self {
        match c {
            'E' => Self::Error,   // Error
            'W' => Self::Warning, // Warning
            'I' => Self::Info,    // Info
            'D' => Self::Debug,   // Debug
            'S' => Self::System,  // System
            _ => Self::Normal,    // tout autre code → Normal (défaut sûr)
        }
    }

    /// Retourne le bit de filtrage depuis un code caractère directement.
    ///
    /// Raccourci pour `LogLevel::from_code(c).bit()`.
    /// Utilisé dans le filtre de la [`gtk4::CustomFilter`] pour éviter
    /// la création d'un objet intermédiaire inutile.
    pub(super) const fn bit_for_code(c: char) -> u8 {
        Self::from_code(c).bit() // délégation : conversion code → niveau → bit
    }
}

// =============================================================================
// Encodage / décodage des lignes
// =============================================================================

// Format : `"C|NNNNN|texte brut"` (1 char code + '|' + 5 chiffres + '|' + texte)
//
// Ce format est stocké dans une gtk4::StringList et décodé par la factory de ListView.
// La longueur fixe du champ numéro (5 chiffres) permet de lire les caractères 2..7
// directement sans parsing, et supporte jusqu'à 99 999 lignes sans troncature.

/// Encode une ligne de log dans le format interne `"C|NNNNN|texte"`.
///
/// # Paramètres
/// - `line_no` : numéro de ligne (1-indexé), affiché dans la colonne de gauche.
/// - `line`    : texte brut de la ligne (peut contenir des `|`).
///
/// # Exemple
/// ```
/// let encoded = encode_line(42, "ERROR: connexion refusée");
/// // → "E|00042|ERROR: connexion refusée"
/// ```
pub(super) fn encode_line(line_no: usize, line: &str) -> String {
    // Détection du niveau par mots-clés, puis formatage :
    // - code sur 1 char (ex. 'E')
    // - '|' séparateur
    // - numéro sur 5 chiffres avec zéros de remplissage (%05d)
    // - '|' séparateur
    // - texte brut de la ligne
    format!("{}|{:05}|{}", LogLevel::detect(line).code(), line_no, line)
}

/// Décode une ligne encodée et retourne `(code_niveau, numéro_ligne, texte)`.
///
/// # Format attendu
/// `"C|NNNNN|texte brut"` où :
/// - `C`     : code niveau sur 1 char,
/// - `NNNNN` : numéro de ligne sur 5 chiffres,
/// - `texte` : contenu brut (peut contenir des `|`).
///
/// # Retour
/// `(code: char, line_no: &str, text: &str)` — vues empruntées sur `encoded`.
/// En cas de format invalide, retourne des valeurs de repli sûres.
pub(super) fn decode(encoded: &str) -> (char, &str, &str) {
    // Premier caractère = code niveau ('E', 'W', 'I', 'D', 'S', 'N')
    let code = encoded.chars().next().unwrap_or('N'); // 'N' = Normal si chaîne vide

    // Skip les 2 premiers octets : code + '|' → `rest` commence à NNNNN
    let rest = encoded.get(2..).unwrap_or(""); // slice sûre : "" si trop court

    // Cherche le '|' séparateur entre le numéro et le texte
    rest.find('|').map_or(
        (code, "?????", rest), // séparateur absent : numéro inconnu, reste = tout
        |pos| {
            (
                code,
                &rest[..pos], // numéro de ligne : tranche 0..pos
                rest.get(pos.saturating_add(1)..).unwrap_or(""), // texte : après le '|'
            )
        },
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    // ── LogLevel::detect — mots-clés anglais ─────────────────────────────────

    #[test]
    fn detect_error_en() {
        assert_eq!(
            LogLevel::detect("ERROR: connexion refusée"),
            LogLevel::Error
        );
        assert_eq!(LogLevel::detect("FATAL: panique système"), LogLevel::Error);
        assert_eq!(
            LogLevel::detect("CRITICAL section violated"),
            LogLevel::Error
        );
    }

    #[test]
    fn detect_error_fr() {
        assert_eq!(LogLevel::detect("ERREUR: timeout"), LogLevel::Error);
        assert_eq!(LogLevel::detect("erreur fatale"), LogLevel::Error);
        assert_eq!(
            LogLevel::detect("CRITIQUE: données corrompues"),
            LogLevel::Error
        );
    }

    #[test]
    fn detect_warning_en() {
        assert_eq!(LogLevel::detect("WARNING: déprécié"), LogLevel::Warning);
        assert_eq!(LogLevel::detect("WARN retrying"), LogLevel::Warning);
    }

    #[test]
    fn detect_warning_fr() {
        assert_eq!(
            LogLevel::detect("AVERT: connexion lente"),
            LogLevel::Warning
        );
        assert_eq!(
            LogLevel::detect("avertissement important"),
            LogLevel::Warning
        );
    }

    #[test]
    fn detect_info() {
        assert_eq!(LogLevel::detect("INFO démarrage"), LogLevel::Info);
        assert_eq!(LogLevel::detect("some INFO text"), LogLevel::Info);
        assert_eq!(LogLevel::detect("[INFO] connexion établie"), LogLevel::Info);
    }

    #[test]
    fn detect_debug() {
        assert_eq!(LogLevel::detect("DEBUG: valeur = 42"), LogLevel::Debug);
        assert_eq!(
            LogLevel::detect("TRACE: entrée dans la fonction"),
            LogLevel::Debug
        );
    }

    #[test]
    fn detect_system() {
        assert_eq!(
            LogLevel::detect("[12:34:56] Système connecté"),
            LogLevel::System
        );
        assert_eq!(LogLevel::detect("[SYS] initialisation"), LogLevel::System);
        assert_eq!(
            LogLevel::detect("[00:00:00] Systeme ready"),
            LogLevel::System
        );
    }

    #[test]
    fn detect_normal_no_keyword() {
        assert_eq!(LogLevel::detect("bonjour le monde"), LogLevel::Normal);
        assert_eq!(LogLevel::detect(""), LogLevel::Normal);
        assert_eq!(LogLevel::detect("   "), LogLevel::Normal);
    }

    #[test]
    fn detect_error_wins_over_warning() {
        assert_eq!(LogLevel::detect("FATAL WARN mixed"), LogLevel::Error);
    }

    // ── LogLevel::bit — bits uniques ─────────────────────────────────────────

    #[test]
    fn all_bits_are_distinct() {
        let bits = [
            LogLevel::Error.bit(),
            LogLevel::Warning.bit(),
            LogLevel::Info.bit(),
            LogLevel::Debug.bit(),
            LogLevel::System.bit(),
            LogLevel::Normal.bit(),
        ];
        for i in 0..bits.len() {
            for j in (i + 1)..bits.len() {
                assert_ne!(bits[i], bits[j], "bits[{i}] == bits[{j}]");
                assert_eq!(
                    bits[i] & bits[j],
                    0,
                    "bits[{i}] et bits[{j}] se chevauchent"
                );
            }
        }
    }

    // ── LogLevel::badge — 4 caractères fixes ─────────────────────────────────

    #[test]
    fn all_badges_are_4_chars() {
        for level in [
            LogLevel::Error,
            LogLevel::Warning,
            LogLevel::Info,
            LogLevel::Debug,
            LogLevel::System,
            LogLevel::Normal,
        ] {
            assert_eq!(
                level.badge().chars().count(),
                4,
                "badge de {:?} ne fait pas 4 chars",
                level.code()
            );
        }
    }

    // ── LogLevel::from_code / bit_for_code ───────────────────────────────────

    #[test]
    fn from_code_round_trips() {
        for level in [
            LogLevel::Error,
            LogLevel::Warning,
            LogLevel::Info,
            LogLevel::Debug,
            LogLevel::System,
            LogLevel::Normal,
        ] {
            assert_eq!(LogLevel::from_code(level.code()), level);
        }
    }

    #[test]
    fn from_code_unknown_returns_normal() {
        assert_eq!(LogLevel::from_code('?'), LogLevel::Normal);
        assert_eq!(LogLevel::from_code('X'), LogLevel::Normal);
    }

    #[test]
    fn bit_for_code_matches_level_bit() {
        assert_eq!(LogLevel::bit_for_code('E'), LogLevel::Error.bit());
        assert_eq!(LogLevel::bit_for_code('W'), LogLevel::Warning.bit());
        assert_eq!(LogLevel::bit_for_code('N'), LogLevel::Normal.bit());
    }

    // ── encode_line / decode — format "C|NNNNN|texte" ────────────────────────

    #[test]
    fn encode_line_format_and_padding() {
        let encoded = encode_line(1, "INFO démarrage");
        assert_eq!(encoded, "I|00001|INFO démarrage");
    }

    #[test]
    fn encode_line_99999_no_truncation() {
        let encoded = encode_line(99_999, "DEBUG limite haute");
        assert!(encoded.starts_with("D|99999|"));
    }

    #[test]
    fn encode_line_with_pipe_in_text() {
        let encoded = encode_line(5, "a|b|c");
        let (code, no, text) = decode(&encoded);
        assert_eq!(code, 'N');
        assert_eq!(no, "00005");
        assert_eq!(text, "a|b|c");
    }

    #[test]
    fn decode_empty_string_returns_safe_defaults() {
        let (code, no, text) = decode("");
        assert_eq!(code, 'N');
        assert_eq!(no, "?????");
        assert_eq!(text, "");
    }

    #[test]
    fn round_trip_encode_decode() {
        let original = "ERROR: crash fatal";
        let encoded = encode_line(42, original);
        let (code, no, text) = decode(&encoded);
        assert_eq!(code, 'E');
        assert_eq!(no, "00042");
        assert_eq!(text, original);
    }
}
