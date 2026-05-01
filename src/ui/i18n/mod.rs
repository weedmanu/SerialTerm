//! @file       ui/i18n.rs
//! @author     M@nu
//! @brief      Gestion de la langue et des traductions FR/EN.
//! @version    1.0.0
//! @date       2025-03-05
//! @copyright  MIT License
//!
//! Ce module définit [`UiLang`], une énumération simple à deux variantes
//! (`Fr` / `En`) avec des méthodes `const fn` retournant des chaînes
//! statiques pour chaque élément d'interface.
//!
//! ## Choix de conception
//!
//! - Les traductions sont codées en dur (pas de fichiers `.po` / gettext).
//! - `const fn` garantit zéro allocation à l'exécution.
//! - La détection automatique lit la variable d'environnement `LANG`.
//! - Si `LANG` ne commence pas par `"fr"`, l'anglais est utilisé.

/// Langue de l'interface utilisateur.
///
/// Utilisée par tous les composants UI pour retourner les chaînes
/// dans la bonne langue sans dépendre d'une bibliothèque i18n externe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLang {
    /// Français — sélectionné si `LANG` commence par `"fr"` ou si
    /// la préférence est `"fr"`, `"fr-fr"`, `"french"` ou `"français"`.
    Fr,
    /// Anglais — sélectionné par défaut si aucune correspondance FR.
    En,
}

mod logs;
mod menus;
mod serial;
mod shortcuts;
mod status;
mod tools;

impl UiLang {
    /// Détermine la langue à partir d'une préférence textuelle.
    ///
    /// Accepte : `"fr"`, `"fr-fr"`, `"french"`, `"français"`,
    /// `"en"`, `"en-us"`, `"en-gb"`, `"english"`.
    /// Toute autre valeur déclenche la détection système.
    ///
    /// # Arguments
    ///
    /// * `pref` — Chaîne de préférence (ex : `"auto"`, `"fr"`, `"en"`).
    pub fn from_preference(pref: &str) -> Self {
        let lower = pref.trim().to_lowercase(); // Normaliser en minuscules sans espaces.
        match lower.as_str() {
            "fr" | "fr-fr" | "french" | "français" => Self::Fr, // Préférences françaises explicites.
            "en" | "en-us" | "en-gb" | "english" => Self::En,   // Préférences anglaises explicites.
            _ => Self::from_system(),                           // Détection automatique.
        }
    }

    /// Détecte la langue depuis la variable d'environnement `LANG`.
    ///
    /// Retourne [`UiLang::Fr`] si `LANG` commence par `"fr"`, sinon [`UiLang::En`].
    fn from_system() -> Self {
        let lang = std::env::var("LANG") // Lire la variable LANG (ex: "fr_FR.UTF-8").
            .unwrap_or_default() // Défaut : chaîne vide si non définie.
            .to_lowercase(); // Normaliser pour la comparaison.
        if lang.starts_with("fr") {
            // Toute locale française (fr_FR, fr_BE…).
            Self::Fr
        } else {
            Self::En // Anglais par défaut pour tout autre locale.
        }
    }
}
