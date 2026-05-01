//! Traductions FR/EN — serial.
//!
//! Sous-module de `crate::ui::i18n`. Étend [`UiLang`] avec un
//! ensemble cohérent de méthodes thématiques.

use super::UiLang;

impl UiLang {
    /// Retourne le message affiché après un rafraîchissement des ports série.
    pub const fn serial_ports_refreshed(self) -> &'static str {
        match self {
            Self::Fr => "Ports série rafraîchis.",
            Self::En => "Serial ports refreshed.",
        }
    }

    /// Retourne le message affiché pendant l'actualisation des ports série.
    pub const fn serial_ports_refreshing(self) -> &'static str {
        match self {
            Self::Fr => "Actualisation des ports série...",
            Self::En => "Refreshing serial ports...",
        }
    }

    /// Libellé type de connexion série.
    pub const fn serial_label(self) -> &'static str {
        match self {
            Self::Fr => "Série",
            Self::En => "Serial",
        }
    }

    // -------------------------------------------------------------------------
    // Panneau série
    // -------------------------------------------------------------------------

    /// Tooltip du sélecteur de port série.
    pub const fn serial_port_tooltip(self) -> &'static str {
        match self {
            Self::Fr => "Sélectionner le port série",
            Self::En => "Select serial port",
        }
    }

    /// Tooltip du bouton de rafraîchissement des ports.
    pub const fn serial_refresh_tooltip(self) -> &'static str {
        match self {
            Self::Fr => "Rafraîchir les ports",
            Self::En => "Refresh ports",
        }
    }

    /// Libellé du port série dans la barre compacte.
    pub const fn serial_port_label(self) -> &'static str {
        match self {
            Self::Fr => "Port :",
            Self::En => "Port:",
        }
    }

    /// Libellé de la case d'auto-sélection du port unique.
    pub const fn serial_auto_select_single_port(self) -> &'static str {
        match self {
            Self::Fr => "Auto-sélectionner s'il n'y a qu'un seul port",
            Self::En => "Auto-select when only one port is available",
        }
    }

    /// Tooltip de la case d'auto-sélection du port unique.
    pub const fn serial_auto_select_single_port_tooltip(self) -> &'static str {
        match self {
            Self::Fr => {
                "Sélectionne automatiquement le port détecté quand une seule carte est présente"
            }
            Self::En => "Automatically selects the detected port when only one device is present",
        }
    }

    /// Libellé de la vitesse (baudrate).
    pub const fn serial_speed_label(self) -> &'static str {
        match self {
            Self::Fr => "Vitesse :",
            Self::En => "Speed:",
        }
    }

    /// Libellé de la section avancée (markup HTML).
    pub const fn serial_advanced_label(self) -> &'static str {
        match self {
            Self::Fr => "<b>Réglages supplémentaires</b>",
            Self::En => "<b>More settings</b>",
        }
    }

    /// Libellé des bits de données.
    pub const fn serial_data_bits_label(self) -> &'static str {
        match self {
            Self::Fr => "Bits de données :",
            Self::En => "Data bits:",
        }
    }

    /// Libellé des bits d'arrêt.
    pub const fn serial_stop_bits_label(self) -> &'static str {
        match self {
            Self::Fr => "Bits d'arrêt :",
            Self::En => "Stop bits:",
        }
    }

    /// Libellé de la parité.
    pub const fn serial_parity_label(self) -> &'static str {
        match self {
            Self::Fr => "Parité:",
            Self::En => "Parity:",
        }
    }

    /// Libellé du contrôle de flux.
    pub const fn serial_flow_label(self) -> &'static str {
        match self {
            Self::Fr => "Flux:",
            Self::En => "Flow:",
        }
    }

    /// Libellé du timeout d'I/O série en millisecondes.
    pub const fn serial_timeout_label(self) -> &'static str {
        match self {
            Self::Fr => "Timeout I/O (ms) :",
            Self::En => "Serial I/O timeout (ms):",
        }
    }

    /// Tooltip du timeout d'I/O série.
    pub const fn serial_timeout_tooltip(self) -> &'static str {
        match self {
            Self::Fr => {
                "Délai maximal d'attente sur lecture/écriture série avant retour de contrôle."
            }
            Self::En => "Maximum time to wait on serial read/write before returning control.",
        }
    }

    /// Libellé de la case "Connexion automatique".
    pub const fn serial_auto_reconnect_label(self) -> &'static str {
        match self {
            Self::Fr => "Connexion automatique",
            Self::En => "Auto-connect",
        }
    }

    /// Tooltip de la case "Connexion automatique".
    pub const fn serial_auto_reconnect_tooltip(self) -> &'static str {
        match self {
            Self::Fr => {
                "Se connecte automatiquement au branchement si un seul port est détecté \
                 (avec « sélection auto »), et reconnecte si le port se déconnecte de façon \
                 inattendue (ex. arrachage USB). Le délai est configurable ci-dessous."
            }
            Self::En => {
                "Automatically connects when a single port is detected (with « auto-select »), \
                 and reconnects when the port disconnects unexpectedly \
                 (e.g. USB unplug). The delay is configurable below."
            }
        }
    }

    /// Libellé du délai de reconnexion automatique.
    pub const fn serial_reconnect_delay_label(self) -> &'static str {
        match self {
            Self::Fr => "Délai reconnexion (ms) :",
            Self::En => "Reconnect delay (ms):",
        }
    }

    /// Tooltip du délai de reconnexion automatique.
    pub const fn serial_reconnect_delay_tooltip(self) -> &'static str {
        match self {
            Self::Fr => {
                "Délai d'attente en millisecondes avant de tenter la reconnexion automatique \
                 (500 – 30 000 ms)."
            }
            Self::En => {
                "Wait time in milliseconds before attempting automatic reconnection \
                 (500 – 30 000 ms)."
            }
        }
    }

    /// Libellé de l'option sans parité.
    pub const fn serial_parity_none_label(self) -> &'static str {
        match self {
            Self::Fr => "Aucune",
            Self::En => "None",
        }
    }

    /// Libellé de l'option parité impaire.
    pub const fn serial_parity_odd_label(self) -> &'static str {
        match self {
            Self::Fr => "Impaire",
            Self::En => "Odd",
        }
    }

    /// Libellé de l'option parité paire.
    pub const fn serial_parity_even_label(self) -> &'static str {
        match self {
            Self::Fr => "Paire",
            Self::En => "Even",
        }
    }

    /// Libellé de l'option sans contrôle de flux.
    pub const fn serial_flow_none_label(self) -> &'static str {
        match self {
            Self::Fr => "Aucun",
            Self::En => "None",
        }
    }

    /// Libellé du contrôle de flux matériel.
    pub const fn serial_flow_hardware_label(self) -> &'static str {
        match self {
            Self::Fr => "Matériel (RTS/CTS)",
            Self::En => "Hardware (RTS/CTS)",
        }
    }

    /// Libellé du contrôle de flux logiciel.
    pub const fn serial_flow_software_label(self) -> &'static str {
        match self {
            Self::Fr => "Logiciel (XON/XOFF)",
            Self::En => "Software (XON/XOFF)",
        }
    }

    /// Texte affiché quand aucun port série n'est disponible.
    pub const fn serial_no_port(self) -> &'static str {
        match self {
            Self::Fr => "— Brancher un périphérique série —",
            Self::En => "— Connect a serial device —",
        }
    }

    /// Tooltip affiché sur le dropdown quand aucun port n'est détecté.
    pub const fn serial_no_port_tooltip(self) -> &'static str {
        match self {
            Self::Fr => "Aucun port série détecté.\nBranchez un adaptateur USB-série ou une carte de développement,\npuis cliquez sur ↺ pour rafraîchir la liste.",
            Self::En => "No serial port detected.\nPlug in a USB-to-serial adapter or a development board,\nthen click ↺ to refresh the list.",
        }
    }

    /// Préfixe erreur d'envoi.
    pub const fn send_error(self) -> &'static str {
        match self {
            Self::Fr => "Erreur d'envoi :",
            Self::En => "Send error:",
        }
    }

    /// Libellé de fin de ligne.
    pub const fn line_ending_label(self) -> &'static str {
        match self {
            Self::Fr => "Fin :",
            Self::En => "End:",
        }
    }

    /// Entrée «aucun» du sélecteur de fin de ligne.
    pub const fn line_ending_none(self) -> &'static str {
        match self {
            Self::Fr => "Aucun",
            Self::En => "None",
        }
    }

    /// Libellé du bouton Envoyer.
    pub const fn send_label(self) -> &'static str {
        match self {
            Self::Fr => "Envoyer",
            Self::En => "Send",
        }
    }

    /// Libellé de la case Arrêt défilement.
    pub const fn stop_scroll_label(self) -> &'static str {
        match self {
            Self::Fr => "Arrêt défilement",
            Self::En => "Stop scroll",
        }
    }

    /// Tooltip de la case Arrêt défilement.
    pub const fn stop_scroll_tooltip(self) -> &'static str {
        match self {
            Self::Fr => "Bloque le défilement automatique du terminal",
            Self::En => "Block automatic terminal scrolling",
        }
    }
}
