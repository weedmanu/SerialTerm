//! Traductions FR/EN — menus.
//!
//! Sous-module de `crate::ui::i18n`. Étend [`UiLang`] avec un
//! ensemble cohérent de méthodes thématiques.

use super::UiLang;

impl UiLang {
    /// Retourne le libellé du menu "Fichier".
    pub const fn app_menu_file(self) -> &'static str {
        match self {
            Self::Fr => "Fichier", // Menu principal Fichier.
            Self::En => "File",
        }
    }

    /// Retourne le libellé du menu "Édition".
    pub const fn app_menu_edit(self) -> &'static str {
        match self {
            Self::Fr => "Édition",
            Self::En => "Edit",
        }
    }

    /// Retourne le libellé du menu "Outils".
    pub const fn app_menu_tools(self) -> &'static str {
        match self {
            Self::Fr => "Outils",
            Self::En => "Tools",
        }
    }

    /// Retourne le libellé du menu "Aide".
    pub const fn app_menu_help(self) -> &'static str {
        match self {
            Self::Fr => "Aide",
            Self::En => "Help",
        }
    }

    /// Libellé menu «Nouveau terminal».
    pub const fn new_terminal_label(self) -> &'static str {
        match self {
            Self::Fr => "Nouveau terminal",
            Self::En => "New terminal",
        }
    }

    /// Libellé menu «Visualiser les logs».
    pub const fn view_logs_label(self) -> &'static str {
        match self {
            Self::Fr => "Visualiser les logs",
            Self::En => "View logs",
        }
    }

    /// Libellé menu «Copier la sélection du terminal».
    pub const fn copy_terminal_selection_label(self) -> &'static str {
        match self {
            Self::Fr => "Copier la sélection du terminal",
            Self::En => "Copy terminal selection",
        }
    }

    /// Libellé menu «Coller dans la zone de saisie».
    pub const fn paste_input_label(self) -> &'static str {
        match self {
            Self::Fr => "Coller dans la zone de saisie",
            Self::En => "Paste into input field",
        }
    }

    /// Libellé du sous-menu thème.
    pub const fn theme_label(self) -> &'static str {
        match self {
            Self::Fr => "Thème",
            Self::En => "Theme",
        }
    }

    /// Libellé du sous-menu langue.
    pub const fn language_label(self) -> &'static str {
        match self {
            Self::Fr => "Langue",
            Self::En => "Language",
        }
    }

    /// Libellé langue auto.
    pub const fn language_auto_system_label(self) -> &'static str {
        match self {
            Self::Fr => "Automatique (système)",
            Self::En => "Automatic (system)",
        }
    }

    /// Libellé langue française.
    pub const fn language_french_label(self) -> &'static str {
        match self {
            Self::Fr | Self::En => "Français",
        }
    }

    /// Libellé langue anglaise.
    pub const fn language_english_label(self) -> &'static str {
        match self {
            Self::Fr | Self::En => "English",
        }
    }

    /// Libellé menu Contact.
    pub const fn contact_label(self) -> &'static str {
        match self {
            Self::Fr | Self::En => "Contact",
        }
    }

    /// Libellé menu signalement bug/amélioration.
    pub const fn report_issue_label(self) -> &'static str {
        match self {
            Self::Fr => "Signaler un bug / Amélioration",
            Self::En => "Report a Bug / Improvement",
        }
    }

    /// Libellé menu À propos.
    pub const fn about_label(self) -> &'static str {
        match self {
            Self::Fr => "À propos",
            Self::En => "About",
        }
    }

    /// Préfixe message «Thème changé».
    pub const fn theme_changed_prefix(self) -> &'static str {
        match self {
            Self::Fr => "Thème changé :",
            Self::En => "Theme changed:",
        }
    }

    /// Toast confirmation sauvegarde de langue.
    pub const fn language_saved_restart(self) -> &'static str {
        match self {
            Self::Fr => "Langue sauvegardée. Redémarrez l'application pour appliquer.",
            Self::En => "Language saved. Restart the application to apply.",
        }
    }

    /// Erreur de configuration avant connexion.
    pub const fn configuration_error(self) -> &'static str {
        match self {
            Self::Fr => "Erreur de configuration",
            Self::En => "Configuration error",
        }
    }

    /// Corps du dialogue de contact.
    pub const fn contact_dialog_body(self) -> &'static str {
        match self {
            Self::Fr => {
                "Pour toute question, retour ou suggestion,\ncontactez-moi par e-mail :\n\ntutoelectroweb@gmail.com"
            }
            Self::En => {
                "For any question, feedback or suggestion,\ncontact me by e-mail:\n\ntutoelectroweb@gmail.com"
            }
        }
    }

    /// Libellé bouton ouverture client mail.
    pub const fn open_mail_client_label(self) -> &'static str {
        match self {
            Self::Fr => "Ouvrir le client mail",
            Self::En => "Open mail client",
        }
    }

    /// Libellé bouton fermer.
    pub const fn close_label(self) -> &'static str {
        match self {
            Self::Fr => "Fermer",
            Self::En => "Close",
        }
    }

    /// Libellé du bouton Quitter.
    pub const fn quit_label(self) -> &'static str {
        match self {
            Self::Fr => "Quitter",
            Self::En => "Quit",
        }
    }

    // -------------------------------------------------------------------------
    // Erreurs du menu actions (actions.rs)
    // -------------------------------------------------------------------------

    /// Préfixe d'erreur pour le lancement d'une nouvelle instance.
    pub const fn new_terminal_spawn_error(self) -> &'static str {
        match self {
            Self::Fr => "Impossible de lancer une instance séparée :",
            Self::En => "Cannot launch a separate instance:",
        }
    }

    /// Préfixe d'erreur pour la localisation de l'exécutable.
    pub const fn new_terminal_exe_error(self) -> &'static str {
        match self {
            Self::Fr => "Impossible de localiser l'exécutable courant :",
            Self::En => "Cannot locate current executable:",
        }
    }

    /// Description dans la boîte À propos.
    pub const fn about_comments(self) -> &'static str {
        match self {
            Self::Fr => "Terminal série\nÉcrit en Rust + GTK4/Libadwaita",
            Self::En => "Serial Terminal\nWritten in Rust + GTK4/Libadwaita",
        }
    }

    /// Préfixe d'erreur pour l'ouverture du navigateur.
    pub const fn browser_open_error(self) -> &'static str {
        match self {
            Self::Fr => "Impossible d'ouvrir le navigateur :",
            Self::En => "Cannot open browser:",
        }
    }
}
