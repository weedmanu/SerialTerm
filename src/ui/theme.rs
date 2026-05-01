// =============================================================================
// Fichier : theme.rs
// Rôle    : Gestionnaire de thèmes (Clair, Sombre, Hacker)
// =============================================================================

use gtk4::CssProvider;

use crate::ui::i18n::UiLang;

/// Thèmes disponibles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
    Hacker,
}

impl Theme {
    /// Convertit depuis une chaîne.
    pub fn from_str_name(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "light" | "clair" => Self::Light,
            "hacker" | "matrix" => Self::Hacker,
            _ => Self::Dark,
        }
    }

    /// Normalise une préférence texte vers un identifiant de thème canonique.
    pub fn normalized_id(s: &str) -> &'static str {
        match s.to_lowercase().as_str() {
            "light" | "clair" => "light",
            "hacker" | "matrix" => "hacker",
            _ => "dark",
        }
    }

    /// Nom d'affichage.
    pub const fn display_name(&self) -> &str {
        match self {
            Self::Light => "Clair",
            Self::Dark => "Sombre",
            Self::Hacker => "Hacker",
        }
    }

    pub const fn display_name_localized(&self, lang: UiLang) -> &str {
        match lang {
            UiLang::Fr => self.display_name(),
            UiLang::En => match self {
                Self::Light => "Light",
                Self::Dark => "Dark",
                Self::Hacker => "Hacker",
            },
        }
    }

    /// Nom technique.
    pub const fn id(&self) -> &str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::Hacker => "hacker",
        }
    }

    /// Liste de tous les thèmes.
    pub const fn all() -> &'static [Self] {
        &[Self::Light, Self::Dark, Self::Hacker]
    }
}

/// Gestionnaire de thèmes pour l'application.
pub struct ThemeManager;

impl ThemeManager {
    /// Applique le thème sélectionné à l'application.
    pub fn apply(theme: Theme) {
        // Configurer le color scheme Adwaita
        let style_manager = libadwaita::StyleManager::default();
        match theme {
            Theme::Light => {
                style_manager.set_color_scheme(libadwaita::ColorScheme::ForceLight);
            }
            Theme::Dark | Theme::Hacker => {
                style_manager.set_color_scheme(libadwaita::ColorScheme::ForceDark);
            }
        }

        // CSS personnalisé par thème
        let css = Self::css_for_theme(theme);
        let provider = CssProvider::new();
        provider.load_from_string(&css);

        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        log::info!("Thème appliqué : {}", theme.display_name());
    }

    /// Génère le CSS personnalisé pour un thème donné.
    ///
    /// Chaque thème est un bloc CSS complet (couleurs, widgets, états hover/active).
    /// Le fractionnement par sous-fonction ne ferait que déplacer la verbosité
    /// sans apporter de testabilité supplémentaire.
    #[allow(clippy::too_many_lines)]
    fn css_for_theme(theme: Theme) -> String {
        match theme {
            Theme::Light => r#"
                .terminal-view {
                    background-color: #fafafa;
                    color: #2e2e2e;
                    font-family: "Monospace";
                    font-size: 11pt;
                    padding: 8px;
                }
                .input-entry {
                    font-family: "Monospace";
                    font-size: 11pt;
                    min-height: 36px;
                }
                .input-entry,
                .connection-panel entry,
                .send-button,
                .send-button.suggested-action {
                    background-color: transparent;
                    background-image: none;
                    border: 1px solid @borders;
                    box-shadow: none;
                }
                .input-entry placeholder,
                .connection-panel entry placeholder {
                    background-color: transparent;
                }
                .connection-panel {
                    padding: 6px 12px;
                    background-color: transparent;
                }
                .tools-card {
                    background-color: @card_bg_color;
                    border: 1px solid @borders;
                    border-radius: 12px;
                    padding: 10px;
                }
                .tools-title {
                    font-weight: 700;
                }
                .status-connected {
                    color: #26a269;
                    font-weight: bold;
                }
                .status-disconnected {
                    color: #888888;
                }
                .status-bar {
                    background-color: @headerbar_bg_color;
                    border-top: 1px solid @borders;
                    font-size: 9pt;
                }
                .status-dot-connected {
                    color: #26a269;
                    font-size: 11pt;
                }
                .status-dot-disconnected {
                    color: #c0c0c0;
                    font-size: 11pt;
                }
            "#
            .to_string(),

            Theme::Dark => r#"
                .terminal-view {
                    background-color: #1e1e2e;
                    color: #cdd6f4;
                    font-family: "Monospace";
                    font-size: 11pt;
                    padding: 8px;
                }
                .input-entry {
                    font-family: "Monospace";
                    font-size: 11pt;
                    min-height: 36px;
                }
                .input-entry,
                .connection-panel entry,
                .send-button,
                .send-button.suggested-action {
                    background-color: transparent;
                    background-image: none;
                    border: 1px solid @borders;
                    box-shadow: none;
                }
                .input-entry placeholder,
                .connection-panel entry placeholder {
                    background-color: transparent;
                }
                .connection-panel {
                    padding: 6px 12px;
                    background-color: transparent;
                }
                .tools-card {
                    background-color: @card_bg_color;
                    border: 1px solid @borders;
                    border-radius: 12px;
                    padding: 10px;
                }
                .tools-title {
                    font-weight: 700;
                }
                .status-connected {
                    color: #a6e3a1;
                    font-weight: bold;
                }
                .status-disconnected {
                    color: #888888;
                }
                .status-bar {
                    background-color: @headerbar_bg_color;
                    border-top: 1px solid @borders;
                    font-size: 9pt;
                }
                .status-dot-connected {
                    color: #a6e3a1;
                    font-size: 11pt;
                }
                .status-dot-disconnected {
                    color: #555555;
                    font-size: 11pt;
                }
            "#
            .to_string(),

            Theme::Hacker => r#"
                .terminal-view {
                    background-color: #0a0a0a;
                    color: #00ff41;
                    font-family: "Monospace";
                    font-size: 11pt;
                    padding: 8px;
                    text-shadow: 0 0 3px rgba(0, 255, 65, 0.3);
                }
                .input-entry {
                    font-family: "Monospace";
                    font-size: 11pt;
                    min-height: 36px;
                    color: #00ff41;
                }
                .input-entry,
                .connection-panel entry,
                .send-button,
                .send-button.suggested-action {
                    background-color: transparent;
                    background-image: none;
                    border: 1px solid @borders;
                    box-shadow: none;
                }
                .input-entry placeholder,
                .connection-panel entry placeholder {
                    background-color: transparent;
                }
                .connection-panel {
                    padding: 6px 12px;
                    background-color: transparent;
                }
                .tools-card {
                    background-color: shade(#0f0f0f, 1.04);
                    border: 1px solid #00ff41;
                    border-radius: 12px;
                    padding: 10px;
                }
                .tools-title {
                    font-weight: 700;
                }
                .status-connected {
                    color: #00ff41;
                    font-weight: bold;
                }
                .status-disconnected {
                    color: #444444;
                }
                .status-bar {
                    background-color: #111111;
                    border-top: 1px solid #00ff41;
                    font-size: 9pt;
                }
                .status-dot-connected {
                    color: #00ff41;
                    font-size: 11pt;
                }
                .status-dot-disconnected {
                    color: #333333;
                    font-size: 11pt;
                }
                .hacker-title {
                    color: #00ff41;
                    font-weight: bold;
                }
            "#
            .to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Theme;

    #[test]
    fn normalized_id_maps_aliases_to_canonical_theme_ids() {
        assert_eq!(Theme::normalized_id("clair"), "light");
        assert_eq!(Theme::normalized_id("matrix"), "hacker");
    }

    #[test]
    fn normalized_id_falls_back_to_dark_for_unknown_values() {
        assert_eq!(Theme::normalized_id("unknown-theme"), "dark");
    }
}
