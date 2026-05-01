//! Traductions FR/EN — shortcuts.
//!
//! Sous-module de `crate::ui::i18n`. Étend [`UiLang`] avec un
//! ensemble cohérent de méthodes thématiques.

use super::UiLang;

impl UiLang {
    /// Retourne le titre de la fenêtre des raccourcis clavier.
    pub const fn shortcuts_title(self) -> &'static str {
        match self {
            Self::Fr => "Raccourcis clavier",
            Self::En => "Keyboard shortcuts",
        }
    }

    /// Section «Barre d'outils» des raccourcis.
    pub const fn shortcuts_toolbar_section(self) -> &'static str {
        match self {
            Self::Fr => "Barre d'outils",
            Self::En => "Toolbar",
        }
    }

    /// Section «Terminal» des raccourcis.
    pub const fn shortcuts_terminal_section(self) -> &'static str {
        match self {
            Self::Fr | Self::En => "Terminal",
        }
    }

    /// Libellé raccourci effacement terminal.
    pub const fn shortcuts_clear_terminal(self) -> &'static str {
        match self {
            Self::Fr => "Effacer le terminal",
            Self::En => "Clear terminal",
        }
    }

    /// Libellé occurrence suivante.
    pub const fn shortcuts_next_match(self) -> &'static str {
        match self {
            Self::Fr => "Occurrence suivante",
            Self::En => "Next match",
        }
    }

    /// Libellé occurrence précédente.
    pub const fn shortcuts_previous_match(self) -> &'static str {
        match self {
            Self::Fr => "Occurrence précédente",
            Self::En => "Previous match",
        }
    }

    /// Libellé raccourci copie terminal.
    pub const fn shortcuts_copy_terminal_selection(self) -> &'static str {
        match self {
            Self::Fr => "Copier la sélection du terminal",
            Self::En => "Copy terminal selection",
        }
    }

    /// Libellé raccourci collage dans la saisie.
    pub const fn shortcuts_paste_input(self) -> &'static str {
        match self {
            Self::Fr => "Coller dans la zone de saisie",
            Self::En => "Paste into input field",
        }
    }
}
