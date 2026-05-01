//! @file       runtime.rs
//! @author     M@nu
//! @brief      Runtime UI partagé: assainissement desktop et support de tests GTK.
//! @version    1.0.0
//! @date       2025-03-24
//! @copyright  MIT License

use gtk4::gio;
use gtk4::prelude::SettingsExt;

pub fn sanitized_gtk_theme(theme_name: &str) -> Option<&'static str> {
    let normalized = theme_name.trim().to_ascii_lowercase();
    if !normalized.starts_with("yaru-mate") {
        return None;
    }

    if normalized.contains("dark") {
        Some("Yaru-dark")
    } else {
        Some("Yaru")
    }
}

pub fn sanitize_problematic_desktop_theme() {
    if std::env::var_os("GTK_THEME").is_some() {
        return;
    }

    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("XDG_SESSION_DESKTOP"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !desktop.contains("mate") {
        return;
    }

    let interface_settings = gio::Settings::new("org.gnome.desktop.interface");
    let current_theme = interface_settings.string("gtk-theme");
    let Some(fallback_theme) = sanitized_gtk_theme(current_theme.as_str()) else {
        return;
    };

    log::info!(
        "Theme GTK systeme '{current_theme}' incompatible avec Libadwaita sous MATE, fallback vers '{fallback_theme}' pour cette session."
    );
    // SAFETY: appelé dans `app::run()` avant la création de l'objet
    // `libadwaita::Application` et donc avant tout thread GTK/GLib.
    // Aucune course possible : le processus est encore mono-thread à ce stade.
    // La valeur injectée (`"Yaru"` ou `"Yaru-dark"`) est une chaîne statique
    // ne contenant ni nul, ni caractères de contrôle.
    #[allow(clippy::undocumented_unsafe_blocks)]
    unsafe {
        std::env::set_var("GTK_THEME", fallback_theme);
    }
}

#[cfg(test)]
mod tests {
    use super::sanitized_gtk_theme;

    #[test]
    fn maps_yaru_mate_light_to_yaru() {
        assert_eq!(sanitized_gtk_theme("Yaru-MATE-light"), Some("Yaru"));
    }

    #[test]
    fn maps_yaru_mate_dark_to_yaru_dark() {
        assert_eq!(sanitized_gtk_theme("Yaru-MATE-dark"), Some("Yaru-dark"));
    }

    #[test]
    fn keeps_unrelated_themes_untouched() {
        assert_eq!(sanitized_gtk_theme("Adwaita-dark"), None);
    }
}
