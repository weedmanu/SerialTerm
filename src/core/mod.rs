//! @file       core/mod.rs
//! @author     M@nu
//! @brief      Façade du module `core` — logique métier sans dépendance UI.
//! @version    0.95.0
//! @date       2026-04-28
//! @copyright  MIT License
//!
//! Le module `core` regroupe toute la logique métier indépendante de GTK :
//! - `connection`     : trait unifié et acteur asynchrone de connexion.
//! - `logger`         : initialisation du système de logging.
//! - `serial_manager` : gestionnaire de connexion série (tokio-serial).
//! - `settings`       : configuration persistante JSON (serde).
//!
//! Aucun import GTK/GLib ne doit figurer dans ce module — SOLID respecté.

pub mod connection; // Trait `Connection`, événements, commandes, acteur Tokio.
pub mod logger; // Initialisation env_logger avec horodatage.
pub mod serial_manager; // Connexion série asynchrone via tokio-serial.
pub mod settings; // Configuration JSON persistante (serde).
