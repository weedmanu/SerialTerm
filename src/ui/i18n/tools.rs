//! Traductions FR/EN — tools.
//!
//! Sous-module de `crate::ui::i18n`. Étend [`UiLang`] avec un
//! ensemble cohérent de méthodes thématiques.

use super::UiLang;

impl UiLang {
    /// Retourne le titre de la fenêtre d'outils.
    pub const fn tools_title(self) -> &'static str {
        match self {
            Self::Fr => "Outils",
            Self::En => "Tools",
        }
    }

    /// Libellé outil calculatrice et convertisseur.
    pub const fn calculator_converter_label(self) -> &'static str {
        match self {
            Self::Fr => "Calculatrice et convertisseur",
            Self::En => "Calculator and converter",
        }
    }

    /// Titre section calculatrice.
    pub const fn calculator_title(self) -> &'static str {
        match self {
            Self::Fr => "Calculatrice",
            Self::En => "Calculator",
        }
    }

    /// Placeholder exemple calculatrice.
    pub const fn calculator_placeholder(self) -> &'static str {
        match self {
            Self::Fr | Self::En => "Ex: (12+5)*3/2",
        }
    }

    /// Libellé bouton calculer.
    pub const fn calculate_label(self) -> &'static str {
        match self {
            Self::Fr => "Calculer",
            Self::En => "Compute",
        }
    }

    /// Libellé initial du résultat calculatrice.
    pub const fn calculator_result_placeholder(self) -> &'static str {
        match self {
            Self::Fr => "Résultat: -",
            Self::En => "Result: -",
        }
    }

    /// Titre convertisseur DEC/HEX/BIN.
    pub const fn converter_title(self) -> &'static str {
        match self {
            Self::Fr => "Convertisseur DEC / HEX / BIN",
            Self::En => "DEC / HEX / BIN Converter",
        }
    }

    /// Placeholder valeur à convertir.
    pub const fn value_to_convert_placeholder(self) -> &'static str {
        match self {
            Self::Fr => "Valeur à convertir",
            Self::En => "Value to convert",
        }
    }

    /// Libellé bouton convertir.
    pub const fn convert_label(self) -> &'static str {
        match self {
            Self::Fr => "Convertir",
            Self::En => "Convert",
        }
    }

    /// Message calculatrice expression vide.
    pub const fn calculator_empty_expression(self) -> &'static str {
        match self {
            Self::Fr => "Résultat: expression vide",
            Self::En => "Result: empty expression",
        }
    }

    /// Message calculatrice résultat.
    pub fn calculator_result_value(self, value: f64) -> String {
        match self {
            Self::Fr => format!("Résultat: {value}"),
            Self::En => format!("Result: {value}"),
        }
    }

    /// Message calculatrice erreur.
    pub fn calculator_result_error(self, error: &str) -> String {
        match self {
            Self::Fr => format!("Résultat: erreur ({error})"),
            Self::En => format!("Result: error ({error})"),
        }
    }

    /// Erreur convertisseur entrée vide.
    pub const fn converter_empty_value_error(self) -> &'static str {
        match self {
            Self::Fr => "Erreur: valeur vide",
            Self::En => "Error: empty value",
        }
    }

    /// Erreur convertisseur formatée.
    pub fn converter_error(self, error: &str) -> String {
        match self {
            Self::Fr => format!("Erreur: {error}"),
            Self::En => format!("Error: {error}"),
        }
    }
}
