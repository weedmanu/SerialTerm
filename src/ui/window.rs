//! @file       ui/window.rs
//! @author     M@nu
//! @brief      Façade du module `window` — fenêtre principale `SerialTerm`.
//! @version    1.0.0
//! @date       2025-03-05
//! @copyright  GPL-3.0-or-later
//!
//! Ce module orchestre la fenêtre principale en découpant les responsabilités
//! en sous-modules privés selon le principe de responsabilité unique (SRP) :
//!
//! - `shell`      : struct `MainWindow`, construction GTK (widgets, layout).
//! - `actions`    : actions GIO + raccourcis clavier.
//! - `signals`    : connexions signaux GTK (boutons, entrées, close-request).
//! - `lifecycle`  : connexion/déconnexion, pompe d'événements, statuts.
//! - `operations` : opérations métier (envoi, sauvegarde, logs).
//! - `dialogs`    : dialogues utilitaires et raccourcis clavier.
//!
//! Seul `MainWindow` est ré-exporté comme API publique du module.

mod actions; // Actions GIO et raccourcis clavier globaux.
mod dialogs; // Dialogues utilitaires + fenêtre des raccourcis.
mod lifecycle; // Connexion/déconnexion async, pompe d'événements GLib.
mod operations; // Envoi de données, sauvegarde logs et opérations terminal.
mod shell; // Struct `MainWindow` et construction complète de la fenêtre GTK.
mod signals; // Connexions signaux GTK (boutons, entrées, close-request).

pub use shell::MainWindow; // Seul type public : point d'entrée de la fenêtre.
