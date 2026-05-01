//! Façade du module terminal (UI).
//!
//! - `ansi`    : struct `TerminalPanel`, parseur ANSI, état interne,
//! - `display` : implémentation des comportements UI (`impl TerminalPanel`).

mod ansi;
mod display;

pub use ansi::LogExportMode;
pub use ansi::TerminalPanel;
