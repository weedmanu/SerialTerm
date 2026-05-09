//! @file       `ui/header_bar.rs`
//! @author     M@nu
//! @brief      Barre d'en-tête GTK avec boutons d'action rapide.
//! @version    1.0.0
//! @date       2025-03-05
//! @copyright  GPL-3.0-or-later
//!
//! Ce module définit [`AppHeaderBar`], la barre de titre de l'application.
//!
//! ## Design
//!
//! La barre de titre est volontairement minimaliste :
//! - Elle affiche uniquement le titre et les contrôles fenêtre (×, □, −).
//! - Les boutons fonctionnels (`toggle_sidebar_button`, `save_log_button`) sont
//!   créés ici mais **placés dans la toolbar applicative** par `shell.rs`.
//!
//! Ce découplage permet à `shell.rs` de les positionner librement dans le
//! layout sans dépendre de l'implémentation interne de la barre de titre.

use gtk4::Button; // Bouton standard GTK4.
use libadwaita::HeaderBar; // Barre de titre Adwaita (intégration CSD).

use crate::ui::i18n::UiLang; // Traductions FR/EN pour les tooltips.

/// Barre d'en-tête de l'application.
///
/// Contient la [`HeaderBar`] Adwaita et les boutons d'action rapide
/// qui seront placés dans la toolbar par `crate::ui::window::shell`.
pub struct AppHeaderBar {
    /// Barre de titre Adwaita (CSD — Client-Side Decorations).
    pub header_bar: HeaderBar,
    /// Bouton de sauvegarde des logs dans un fichier texte.
    pub save_log_button: Button,
    /// Bouton bascule pour afficher/masquer le panneau latéral de connexion.
    pub toggle_sidebar_button: gtk4::ToggleButton,
}

impl AppHeaderBar {
    /// Construit la barre d'en-tête et ses boutons.
    ///
    /// # Arguments
    ///
    /// * `lang` — Langue de l'interface pour les tooltips localisés.
    pub fn new(lang: UiLang) -> Self {
        // Barre de titre minimaliste : titre + contrôles fenêtre (X _ □) uniquement.
        // Tous les boutons fonctionnels sont placés dans la toolbar applicative (window/shell.rs).
        let header_bar = HeaderBar::new();

        // Bouton toggle sidebar — créé ici, positionné dans la toolbar par shell.rs.
        let toggle_sidebar_button = gtk4::ToggleButton::builder()
            .icon_name("sidebar-show-symbolic") // Icône standard Adwaita (panneau latéral).
            .tooltip_text(lang.toggle_sidebar_tooltip()) // Tooltip localisé FR/EN.
            .active(false) // Panneau MASQUÉ au démarrage.
            .build();

        // Bouton de sauvegarde — créé ici, positionné dans la toolbar par shell.rs.
        let save_log_button = Button::builder()
            .icon_name("document-save-symbolic") // Icône standard GTK (sauvegarder).
            .tooltip_text(lang.save_logs_tooltip()) // Tooltip localisé FR/EN.
            .build();

        Self {
            header_bar,            // Barre de titre CSD.
            save_log_button,       // Bouton sauvegarde logs.
            toggle_sidebar_button, // Bouton toggle panneau latéral.
        }
    }
}
