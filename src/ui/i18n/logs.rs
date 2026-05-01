//! Traductions FR/EN — logs.
//!
//! Sous-module de `crate::ui::i18n`. Étend [`UiLang`] avec un
//! ensemble cohérent de méthodes thématiques.

use super::UiLang;

impl UiLang {
    /// Retourne le tooltip du bouton de sauvegarde des logs.
    pub const fn save_logs_tooltip(self) -> &'static str {
        match self {
            Self::Fr => "Sauvegarder les logs",
            Self::En => "Save logs",
        }
    }

    /// Libellé menu «Sauvegarder les logs».
    pub const fn save_logs_label(self) -> &'static str {
        match self {
            Self::Fr => "Sauvegarder les logs",
            Self::En => "Save logs",
        }
    }

    /// Titre du dialogue de sauvegarde des logs.
    pub const fn save_logs_dialog_title(
        self,
        mode: crate::ui::terminal_panel::LogExportMode,
    ) -> &'static str {
        match (self, mode) {
            (Self::Fr, crate::ui::terminal_panel::LogExportMode::Raw) => "Sauvegarder le flux brut",
            (Self::Fr, crate::ui::terminal_panel::LogExportMode::Timestamped) => {
                "Sauvegarder le flux horodaté"
            }
            (Self::Fr, crate::ui::terminal_panel::LogExportMode::Split) => {
                "Sauvegarder les flux séparés"
            }
            (Self::En, crate::ui::terminal_panel::LogExportMode::Raw) => "Save raw stream",
            (Self::En, crate::ui::terminal_panel::LogExportMode::Timestamped) => {
                "Save timestamped stream"
            }
            (Self::En, crate::ui::terminal_panel::LogExportMode::Split) => "Save split streams",
        }
    }

    /// Titre du choix de format de sauvegarde.
    pub const fn save_logs_mode_heading(self) -> &'static str {
        match self {
            Self::Fr => "Choisir le format de sauvegarde",
            Self::En => "Choose the save format",
        }
    }

    /// Description du choix de format de sauvegarde.
    pub const fn save_logs_mode_body(self) -> &'static str {
        match self {
            Self::Fr => {
                "Brut : conserve seulement le flux de session utile.\nHorodaté : ajoute l'heure locale de capture à chaque ligne utile.\nSéparé : écrit trois sections distinctes pour RX, TX et Système."
            }
            Self::En => {
                "Raw: keeps only the useful session stream.\nTimestamped: adds the local capture time to each useful line.\nSplit: writes three distinct sections for RX, TX and System."
            }
        }
    }

    /// Libellé du choix de sauvegarde brute.
    pub const fn save_logs_mode_raw_label(self) -> &'static str {
        match self {
            Self::Fr => "Brut",
            Self::En => "Raw",
        }
    }

    /// Libellé du choix de sauvegarde horodatée.
    pub const fn save_logs_mode_timestamped_label(self) -> &'static str {
        match self {
            Self::Fr => "Horodaté",
            Self::En => "Timestamped",
        }
    }

    /// Libellé du choix de sauvegarde séparée.
    pub const fn save_logs_mode_split_label(self) -> &'static str {
        match self {
            Self::Fr => "Séparé RX/TX/Système",
            Self::En => "Split RX/TX/System",
        }
    }

    /// Libellé d'annulation du choix de sauvegarde.
    pub const fn save_logs_mode_cancel_label(self) -> &'static str {
        match self {
            Self::Fr => "Annuler",
            Self::En => "Cancel",
        }
    }

    /// Toast logs sauvegardés (préfixe, chemin ajouté par l'appelant).
    pub const fn logs_saved_toast(
        self,
        mode: crate::ui::terminal_panel::LogExportMode,
    ) -> &'static str {
        match (self, mode) {
            (Self::Fr, crate::ui::terminal_panel::LogExportMode::Raw) => "✓ Flux brut sauvegardé :",
            (Self::Fr, crate::ui::terminal_panel::LogExportMode::Timestamped) => {
                "✓ Flux horodaté sauvegardé :"
            }
            (Self::Fr, crate::ui::terminal_panel::LogExportMode::Split) => {
                "✓ Flux séparés sauvegardés :"
            }
            (Self::En, crate::ui::terminal_panel::LogExportMode::Raw) => "✓ Raw stream saved:",
            (Self::En, crate::ui::terminal_panel::LogExportMode::Timestamped) => {
                "✓ Timestamped stream saved:"
            }
            (Self::En, crate::ui::terminal_panel::LogExportMode::Split) => "✓ Split streams saved:",
        }
    }

    /// Préfixe message terminal logs sauvegardés.
    pub const fn logs_saved_term_prefix(
        self,
        mode: crate::ui::terminal_panel::LogExportMode,
    ) -> &'static str {
        match (self, mode) {
            (Self::Fr, crate::ui::terminal_panel::LogExportMode::Raw) => {
                "Flux brut sauvegardé dans"
            }
            (Self::Fr, crate::ui::terminal_panel::LogExportMode::Timestamped) => {
                "Flux horodaté sauvegardé dans"
            }
            (Self::Fr, crate::ui::terminal_panel::LogExportMode::Split) => {
                "Flux séparés sauvegardés dans"
            }
            (Self::En, crate::ui::terminal_panel::LogExportMode::Raw) => "Raw stream saved to",
            (Self::En, crate::ui::terminal_panel::LogExportMode::Timestamped) => {
                "Timestamped stream saved to"
            }
            (Self::En, crate::ui::terminal_panel::LogExportMode::Split) => "Split streams saved to",
        }
    }

    // -------------------------------------------------------------------------
    // Visualiseur de logs (log_viewer/window.rs)
    // -------------------------------------------------------------------------

    /// Titre de la fenêtre du visualiseur de logs.
    pub const fn log_viewer_title(self) -> &'static str {
        match self {
            Self::Fr => "Visualiseur de logs",
            Self::En => "Log viewer",
        }
    }

    /// Placeholder de recherche dans les logs.
    pub const fn log_search_placeholder(self) -> &'static str {
        match self {
            Self::Fr => "Rechercher dans les logs…",
            Self::En => "Search in logs…",
        }
    }

    /// Tooltip bouton rafraîchir depuis le terminal.
    pub const fn log_refresh_tooltip(self) -> &'static str {
        match self {
            Self::Fr => "Rafraîchir depuis le terminal",
            Self::En => "Refresh from terminal",
        }
    }

    /// Tooltip bouton exporter les logs.
    pub const fn log_export_tooltip(self) -> &'static str {
        match self {
            Self::Fr => "Exporter les logs filtrés…",
            Self::En => "Export filtered logs…",
        }
    }

    /// Tooltip bouton copier la ligne.
    pub const fn log_copy_tooltip(self) -> &'static str {
        match self {
            Self::Fr => "Copier la ligne sélectionnée (Ctrl+C)",
            Self::En => "Copy selected line (Ctrl+C)",
        }
    }

    /// Libellé «Niveau :» dans la barre de filtres.
    pub const fn log_level_label(self) -> &'static str {
        match self {
            Self::Fr => "Niveau :",
            Self::En => "Level:",
        }
    }

    /// Tooltips des boutons de niveau dans le visualiseur de logs.
    ///
    /// Ordre : ERR, WARN, INFO, DBG, SYS, normal.
    pub const fn log_level_tooltips(self) -> [&'static str; 6] {
        match self {
            Self::Fr => [
                "Erreurs",
                "Avertissements",
                "Informations",
                "Debug / Trace",
                "Messages système",
                "Lignes normales",
            ],
            Self::En => [
                "Errors",
                "Warnings",
                "Information",
                "Debug / Trace",
                "System messages",
                "Normal lines",
            ],
        }
    }

    /// Libellé du bouton «Tout».
    pub const fn log_all_label(self) -> &'static str {
        match self {
            Self::Fr => "Tout",
            Self::En => "All",
        }
    }

    /// Tooltip du bouton «Tout».
    pub const fn log_all_tooltip(self) -> &'static str {
        match self {
            Self::Fr => "Activer tous les niveaux",
            Self::En => "Enable all levels",
        }
    }

    /// Libellé du bouton «Aucun».
    pub const fn log_none_label(self) -> &'static str {
        match self {
            Self::Fr => "Aucun",
            Self::En => "None",
        }
    }

    /// Tooltip du bouton «Aucun».
    pub const fn log_none_tooltip(self) -> &'static str {
        match self {
            Self::Fr => "Désactiver tous les niveaux",
            Self::En => "Disable all levels",
        }
    }

    /// Tooltip du bouton de tri.
    pub const fn log_sort_tooltip(self) -> &'static str {
        match self {
            Self::Fr => "Inverser l'ordre (numéro de ligne)",
            Self::En => "Reverse order (line number)",
        }
    }

    /// Texte d'aide dans la barre de statut du visualiseur.
    pub const fn log_status_hint(self) -> &'static str {
        match self {
            Self::Fr => {
                "Clic : sélectionner  ·  Triple-clic : sélectionner tout le texte  ·  Ctrl+C : copier"
            }
            Self::En => "Click: select  ·  Triple-click: select all text  ·  Ctrl+C: copy",
        }
    }

    /// Compteur de lignes dans le visualiseur (format complet, retourne un `String`).
    pub fn log_count_label(self, visible: u32, total: usize, errors: u32, warnings: u32) -> String {
        match self {
            Self::Fr => format!(
                "{visible} / {total} lignes  —  {errors} erreur(s)  ·  {warnings} avertissement(s)"
            ),
            Self::En => {
                format!("{visible} / {total} lines  —  {errors} error(s)  ·  {warnings} warning(s)")
            }
        }
    }

    /// Titre du dialogue d'export des logs.
    pub const fn log_export_dialog_title(self) -> &'static str {
        match self {
            Self::Fr => "Exporter les logs filtrés",
            Self::En => "Export filtered logs",
        }
    }
}
