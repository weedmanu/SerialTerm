//! @file       main.rs
//! @author     M@nu
//! @brief      Point d'entrée de `SerialTerm` — lints globaux + bootstrap.
//! @version    1.0.0
//! @date       2025-03-05
//! @copyright  GPL-3.0-or-later
//!
//! # Architecture
//!
//! ```text
//! main()
//!  ├── core::logger::init_logger()   — initialise env_logger
//!  └── app::run()                    — construit et lance l'application GTK
//!       ├── core/   — logique métier pure (serial, settings, connection)
//!       ├── ui/     — interface GTK4/Libadwaita (window, panels, themes)
//!       └── app.rs  — bootstrap libadwaita::Application
//! ```
//!
//! # Lints activés
//!
//! `clippy::all + pedantic + nursery + perf` couvrent les best-practices Rust.
//! Des lints de sécurité supplémentaires (`cast_*`, `unwrap_used`, …) évitent
//! les conversions silencieuses et les panics non contrôlés.

// ─── Lints globaux ────────────────────────────────────────────────────────────
// `all + pedantic + nursery` : quasi-totalité des best-practices Rust.
// `perf`           : détecte les allocations et clones inutiles.
// `as_conversions` : interdit les casts `as` implicitement tronquants.
// `cast_*`         : force à utiliser `try_from` / `saturating_*` explicitement.
// `unwrap_used`    : oblige à gérer les erreurs (pas de panic caché).
// `indexing_slicing` : empêche les accès non bornés qui paniquent.
// `arithmetic_side_effects` : oblige à utiliser `saturating_*` ou `checked_*`.
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::perf,
    clippy::as_conversions,        // casts `as` potentiellement dangereux
    clippy::cast_possible_truncation, // i64 as i32 peut tronquer
    clippy::cast_possible_wrap,    // u32 as i32 peut devenir négatif
    clippy::cast_sign_loss,        // i32 as u32 perd le signe
    clippy::unwrap_used,           // `.unwrap()` peut paniquer
    clippy::indexing_slicing,      // `slice[i]` peut paniquer hors bornes
    clippy::arithmetic_side_effects, // débordements entiers non contrôlés
)]

use gtk4::glib; // Nécessaire pour le type de retour `glib::ExitCode`.

mod app; // Bootstrap de l'application GTK4/Libadwaita.
mod application; // Cas d'utilisation (logique métier applicative pure).
mod core; // Logique métier bas niveau (serial, settings…).
mod ui; // Interface graphique GTK4/Libadwaita.

/// Point d'entrée du programme.
///
/// Charge la configuration persistante pour initialiser le logger avec les
/// préférences utilisateur (`enabled`, `level`, `log_to_file`, `log_directory`),
/// puis délègue à [`app::run()`] qui crée et lance la boucle d'événements GTK.
fn main() -> glib::ExitCode {
    // Charger la configuration persistante avant GTK : SettingsManager ne dépend
    // que du système de fichiers (XDG), pas de la boucle d'événements.
    let log_config = {
        let settings = crate::core::settings::SettingsManager::new();
        settings.settings().log.clone()
    };

    crate::core::logger::init_logger_with_config(
        log_config.enabled,
        &log_config.level,
        log_config.log_to_file,
        &log_config.log_directory,
    );
    log::info!("Démarrage de serial-term v{}", env!("CARGO_PKG_VERSION"));

    app::run() // Lancer l'application ; retourne le code de sortie GTK.
}
