//! Traductions FR/EN — status.
//!
//! Sous-module de `crate::ui::i18n`. Étend [`UiLang`] avec un
//! ensemble cohérent de méthodes thématiques.

use super::UiLang;

impl UiLang {
    /// Retourne le texte de statut "Déconnecté".
    pub const fn status_disconnected(self) -> &'static str {
        match self {
            Self::Fr => "Déconnecté",
            Self::En => "Disconnected",
        }
    }

    /// Retourne le texte de statut "Connexion en cours…".
    pub const fn status_connecting(self) -> &'static str {
        match self {
            Self::Fr => "Connexion en cours...",
            Self::En => "Connecting...",
        }
    }

    /// Retourne le tooltip du bouton Connecter.
    pub const fn connect_tooltip(self) -> &'static str {
        match self {
            Self::Fr => "Connecter / Déconnecter",
            Self::En => "Connect / Disconnect",
        }
    }

    /// Retourne le tooltip du bouton Déconnecter (état connecté).
    pub const fn disconnect_tooltip(self) -> &'static str {
        match self {
            Self::Fr => "Déconnecter",
            Self::En => "Disconnect",
        }
    }

    /// Retourne le tooltip du bouton d'effacement du terminal.
    pub const fn clear_terminal_tooltip(self) -> &'static str {
        match self {
            Self::Fr => "Effacer le terminal (Ctrl+L)",
            Self::En => "Clear terminal (Ctrl+L)",
        }
    }

    /// Retourne le tooltip du bouton de bascule du panneau latéral.
    pub const fn toggle_sidebar_tooltip(self) -> &'static str {
        match self {
            Self::Fr => "Afficher/Masquer le panneau latéral",
            Self::En => "Show/Hide side panel",
        }
    }

    /// Retourne le message affiché après un effacement du terminal.
    pub const fn terminal_cleared(self) -> &'static str {
        match self {
            Self::Fr => "Terminal effacé.",
            Self::En => "Terminal cleared.",
        }
    }

    /// Message affiché dans la barre basse quand une connexion automatique est planifiée.
    pub const fn auto_reconnect_scheduled_notice(self) -> &'static str {
        match self {
            Self::Fr => "Connexion automatique dans",
            Self::En => "Auto-connecting in",
        }
    }

    /// Message affiché dans la barre basse lors de la tentative de connexion automatique.
    pub const fn auto_reconnect_attempt_notice(self) -> &'static str {
        match self {
            Self::Fr => "Tentative de connexion automatique...",
            Self::En => "Attempting automatic connection...",
        }
    }

    /// Message affiché quand le port n'est pas disponible lors d'une tentative automatique.
    pub const fn auto_reconnect_port_unavailable_notice(self) -> &'static str {
        match self {
            Self::Fr => "Port non disponible, nouvelle tentative...",
            Self::En => "Port unavailable, retrying...",
        }
    }

    /// Retourne le texte du toast affiché lors d'une déconnexion.
    pub const fn connection_closed_toast(self) -> &'static str {
        match self {
            Self::Fr => "Connexion terminée",
            Self::En => "Connection closed",
        }
    }

    /// Libellé menu «Rechercher dans terminal».
    pub const fn search_terminal_label(self) -> &'static str {
        match self {
            Self::Fr => "Rechercher dans terminal",
            Self::En => "Search in terminal",
        }
    }

    /// Retourne la première ligne du message de bienvenue.
    pub const fn welcome_line_1(self) -> &'static str {
        match self {
            Self::Fr => "Bienvenue dans SerialTerm !",
            Self::En => "Welcome to SerialTerm!",
        }
    }

    /// Retourne la deuxième ligne du message de bienvenue.
    pub const fn welcome_line_2(self) -> &'static str {
        match self {
            Self::Fr => {
                // Message d'aide expliquant les étapes de connexion.
                "Sélectionnez le port série puis cliquez sur Connecter."
            }
            Self::En => "Select the serial port and click Connect.",
        }
    }

    /// Libellé de l'onglet connexion série.
    pub const fn conn_serial_tab(self) -> &'static str {
        match self {
            Self::Fr => "🔌 Série",
            Self::En => "🔌 Serial",
        }
    }

    // -------------------------------------------------------------------------
    // Erreur fatale runtime Tokio (shell.rs)
    // -------------------------------------------------------------------------

    /// Titre du dialogue d'erreur fatale.
    pub const fn fatal_error_title(self) -> &'static str {
        match self {
            Self::Fr => "Erreur fatale",
            Self::En => "Fatal error",
        }
    }

    /// Corps du dialogue d'erreur fatale (préfixe, le détail `{e}` est ajouté par l'appelant).
    pub const fn fatal_error_body_prefix(self) -> &'static str {
        match self {
            Self::Fr => "Impossible d'initialiser le runtime asynchrone :\n",
            Self::En => "Cannot initialize async runtime:\n",
        }
    }

    // -------------------------------------------------------------------------
    // Messages opérations (operations.rs)
    // -------------------------------------------------------------------------

    /// Erreur : aucun port série sélectionné.
    pub const fn no_port_selected(self) -> &'static str {
        match self {
            Self::Fr => "Aucun port sélectionné",
            Self::En => "No port selected",
        }
    }

    /// Erreur non connecté.
    pub const fn not_connected_error(self) -> &'static str {
        match self {
            Self::Fr => "Non connecté — impossible d'envoyer.",
            Self::En => "Not connected — cannot send.",
        }
    }

    /// Message terminal rien à sauvegarder.
    pub const fn nothing_to_save(self) -> &'static str {
        match self {
            Self::Fr => "Rien à sauvegarder.",
            Self::En => "Nothing to save.",
        }
    }

    // -------------------------------------------------------------------------
    // Lifecycle (lifecycle.rs)
    // -------------------------------------------------------------------------

    /// Préfixe message système connexion établie.
    pub const fn connected_system(self) -> &'static str {
        match self {
            Self::Fr => "Connecté",
            Self::En => "Connected",
        }
    }

    // -------------------------------------------------------------------------
    // Panneau de saisie (input_panel.rs)
    // -------------------------------------------------------------------------

    /// Placeholder du champ de saisie.
    pub const fn input_placeholder(self) -> &'static str {
        match self {
            Self::Fr => "Tapez votre commande ici...",
            Self::En => "Type your command here...",
        }
    }

    // -------------------------------------------------------------------------
    // Panneau terminal (terminal_panel/display.rs)
    // -------------------------------------------------------------------------

    /// Placeholder de la barre de recherche dans le terminal.
    pub const fn search_terminal_placeholder(self) -> &'static str {
        match self {
            Self::Fr => "Rechercher dans le terminal…",
            Self::En => "Search in terminal…",
        }
    }

    /// Préfixe des messages d'erreur dans le terminal.
    pub const fn error_prefix(self) -> &'static str {
        match self {
            Self::Fr => "ERREUR:",
            Self::En => "ERROR:",
        }
    }
}
