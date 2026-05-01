//! Façade du module log viewer (UI).
//!
//! - `model`  : types `LogLevel`, encodage/décodage des lignes,
//! - `window` : fenêtre, filtres, tri, recherche et export.
//!
//! `open_log_viewer` est ré-exportée comme API publique du module.

mod model;
mod window;

pub use window::open_log_viewer;
