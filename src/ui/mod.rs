//! @file       ui/mod.rs
//! @author     M@nu
//! @brief      Façade du module `ui` — interface GTK4/Libadwaita.
//! @version    1.0.0
//! @date       2025-03-05
//! @copyright  GPL-3.0-or-later
//!
//! Le module `ui` regroupe tous les composants de l'interface graphique :
//! - `connection_panel` : formulaire série, ports, profils.
//! - `header_bar`       : barre de titre et boutons d'action rapide.
//! - `i18n`             : traductions FR/EN (enum `UiLang`).
//! - `input_panel`      : saisie de commandes et fin de ligne.
//! - `log_viewer`       : visualiseur de logs avec filtres et tri.
//! - `terminal_panel`   : terminal ANSI avec parseur vte.
//! - `theme`            : thèmes Clair/Sombre/Hacker + CSS GTK.
//! - `tools_dialog`     : calculatrice et convertisseur DEC/HEX/BIN.
//! - `window`           : fenêtre principale (shell, actions, signaux…).

pub mod connection_panel; // Formulaire de connexion, sélection des ports série et des profils.
pub mod header_bar; // Barre de titre, boutons de contrôle et indicateurs de statut.
pub mod i18n; // Gestion de la langue et des traductions.
pub mod input_panel; // Zone de saisie et boutons d'envoi.
pub mod log_viewer; // Visualisation, filtrage et export des logs.
pub mod runtime; // Runtime UI partagé: compatibilité desktop et support de tests GTK.
pub mod terminal_panel; // Affichage du terminal, gestion des couleurs ANSI.
pub mod theme; // Gestion des thèmes graphiques et des styles CSS.
pub mod tools_dialog; // Dialogues d'outils (calculatrice, convertisseur).
pub mod window; // Fenêtres principale et secondaires.
