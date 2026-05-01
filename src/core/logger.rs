//! @file       core/logger.rs
//! @author     M@nu
//! @brief      Initialisation et configuration du système de logging.
//! @version    1.0.0
//! @date       2025-03-05
//! @copyright  MIT License
//!
//! Ce module fournit deux fonctions d'initialisation :
//!
//! - [`init_logger`] : initialisation directe avec un [`LevelFilter`].
//! - [`init_logger_with_config`] : initialisation depuis la configuration persistante
//!   (`LogSettings`) ; branche `enabled`, `level`, `log_to_file` et `log_directory`.
//!
//! ```text
//! [2025-03-05 14:32:01] INFO  serial_term::core::serial_manager - Connexion série ouverte sur /dev/ttyUSB0.
//! ```
//!
//! - Le niveau de log est paramétrable (`Trace`, `Debug`, `Info`, `Warn`, `Error`).
//! - Le format utilise `chrono::Local` pour l'heure locale.
//! - `{level:<5}` aligne les niveaux sur 5 caractères pour faciliter la lecture.

use std::io::Write; // Trait nécessaire pour `writeln!(buf, …)` dans le formateur.

use chrono::Local; // Horodatage en heure locale.
use env_logger::Builder; // Constructeur du logger configurable.
use log::LevelFilter; // Enum de niveau de filtrage des messages.

/// Initialise le système de logging global avec le niveau spécifié.
///
/// # Format de sortie
///
/// ```text
/// [YYYY-MM-DD HH:MM:SS] LEVEL  module::path - message
/// ```
///
/// # Arguments
///
/// * `level` — Niveau minimum des messages loggués (ex : `LevelFilter::Info`).
///
/// # Panics
///
/// Ne paniquera pas : `writeln!` sur `buf` ne peut pas échouer dans ce contexte.
pub fn init_logger(level: LevelFilter) {
    Builder::new()
        .filter_level(level) // Appliquer le filtre de niveau global.
        .format(|buf, record| {
            // Fermeture de formatage appelée pour chaque message.
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S"); // Heure locale formatée.
            let level = record.level(); // Niveau du message (INFO, WARN, ERROR…).
            let target = record.target(); // Module source du message (ex: `core::serial_manager`).
                                          // Écrire la ligne formatée ; `{level:<5}` aligne sur 5 caractères.
            writeln!(buf, "[{timestamp}] {level:<5} {target} - {}", record.args())
        })
        .init(); // Enregistrer le logger comme logger global (peut être appelé une seule fois).
}

/// Convertit une chaîne de niveau en [`LevelFilter`].
///
/// Insensible à la casse. Retourne `LevelFilter::Info` pour toute valeur inconnue.
#[must_use]
pub fn parse_level_filter(level: &str) -> LevelFilter {
    match level.trim().to_ascii_uppercase().as_str() {
        "TRACE" => LevelFilter::Trace,
        "DEBUG" => LevelFilter::Debug,
        "WARN" | "WARNING" => LevelFilter::Warn,
        "ERROR" => LevelFilter::Error,
        "OFF" => LevelFilter::Off,
        _ => LevelFilter::Info,
    }
}

/// Tente d'ouvrir un fichier de log horodaté dans `log_directory`.
///
/// Crée le répertoire si nécessaire. Retourne `None` si l'opération échoue.
fn try_open_log_file(log_directory: &str) -> Option<Box<dyn Write + Send + 'static>> {
    let dir = std::path::Path::new(log_directory);
    std::fs::create_dir_all(dir).ok()?;
    let filename = Local::now()
        .format("serial_term_%Y%m%d_%H%M%S.log")
        .to_string();
    let file = std::fs::File::create(dir.join(filename)).ok()?;
    Some(Box::new(std::io::BufWriter::new(file)))
}

/// Initialise le logger depuis la configuration persistante.
///
/// - `enabled = false` → `LevelFilter::Off` (aucun log émis).
/// - `level_str` → parsé via [`parse_level_filter`] (insensible à la casse).
/// - `log_to_file = true` → redirige tous les logs vers un fichier horodaté
///   dans `log_directory` ; en cas d'échec d'ouverture, bascule sur stderr.
///
/// # Arguments
///
/// * `enabled`       — Active ou désactive entièrement le logging.
/// * `level_str`     — Niveau textuel (`"INFO"`, `"DEBUG"`, `"WARN"`, …).
/// * `log_to_file`   — Redirige la sortie vers un fichier si `true`.
/// * `log_directory` — Répertoire cible pour le fichier de log.
pub fn init_logger_with_config(
    enabled: bool,
    level_str: &str,
    log_to_file: bool,
    log_directory: &str,
) {
    let level = if enabled {
        parse_level_filter(level_str)
    } else {
        LevelFilter::Off
    };

    if log_to_file {
        if let Some(file_writer) = try_open_log_file(log_directory) {
            Builder::new()
                .filter_level(level)
                .target(env_logger::Target::Pipe(file_writer))
                .format(|buf, record| {
                    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
                    let lvl = record.level();
                    let target = record.target();
                    writeln!(buf, "[{timestamp}] {lvl:<5} {target} - {}", record.args())
                })
                .init();
            return;
        }
        // Ouverture du fichier impossible : fallback stderr avec avertissement immédiat.
        eprintln!(
            "[WARN] Impossible d'ouvrir le fichier de log dans '{log_directory}', sortie sur stderr."
        );
    }

    init_logger(level);
}

// =============================================================================
// Tests unitaires
// =============================================================================

#[cfg(test)]
mod tests {
    use log::LevelFilter;

    use super::parse_level_filter;

    #[test]
    fn parses_info_case_insensitive() {
        assert_eq!(parse_level_filter("info"), LevelFilter::Info);
        assert_eq!(parse_level_filter("INFO"), LevelFilter::Info);
        assert_eq!(parse_level_filter("Info"), LevelFilter::Info);
    }

    #[test]
    fn parses_all_named_levels() {
        assert_eq!(parse_level_filter("TRACE"), LevelFilter::Trace);
        assert_eq!(parse_level_filter("DEBUG"), LevelFilter::Debug);
        assert_eq!(parse_level_filter("WARN"), LevelFilter::Warn);
        assert_eq!(parse_level_filter("WARNING"), LevelFilter::Warn);
        assert_eq!(parse_level_filter("ERROR"), LevelFilter::Error);
        assert_eq!(parse_level_filter("OFF"), LevelFilter::Off);
    }

    #[test]
    fn unknown_level_defaults_to_info() {
        assert_eq!(parse_level_filter(""), LevelFilter::Info);
        assert_eq!(parse_level_filter("VERBOSE"), LevelFilter::Info);
        assert_eq!(parse_level_filter("42"), LevelFilter::Info);
    }

    #[test]
    fn trims_surrounding_whitespace() {
        assert_eq!(parse_level_filter("  DEBUG  "), LevelFilter::Debug);
        assert_eq!(parse_level_filter("\tERROR\n"), LevelFilter::Error);
    }
}
