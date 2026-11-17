// =============================================================================
// Fichier : settings.rs
// Rôle    : Gestion de la configuration persistante (JSON)
// =============================================================================

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[cfg(test)]
thread_local! {
    static TEST_CONFIG_PATH_OVERRIDE: std::cell::RefCell<Option<PathBuf>> =
        const { std::cell::RefCell::new(None) };
}

static NEXT_SAVE: AtomicU64 = AtomicU64::new(0);

pub const MIN_WINDOW_WIDTH: i32 = 800;
pub const MIN_WINDOW_HEIGHT: i32 = 600;

const fn clamp_window_width(width: i32) -> i32 {
    if width < MIN_WINDOW_WIDTH {
        MIN_WINDOW_WIDTH
    } else {
        width
    }
}

const fn clamp_window_height(height: i32) -> i32 {
    if height < MIN_WINDOW_HEIGHT {
        MIN_WINDOW_HEIGHT
    } else {
        height
    }
}

// =============================================================================
// Structures de configuration
// =============================================================================

/// Configuration complète de l'application.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub serial: SerialSettings,
    pub ui: UiSettings,
    pub log: LogSettings,
}

/// Paramètres de connexion série.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SerialSettings {
    pub port: String,
    pub auto_select_single_port: bool,
    pub baudrate: u32,
    pub data_bits: u8,
    pub parity: String,
    pub stop_bits: u8,
    pub flow_control: String,
    pub timeout_ms: u64,
    /// Reconnexion automatique si le port se déconnecte de façon inattendue
    /// (ex. arrachage USB).  Défaut : `false`.
    #[serde(default)]
    pub auto_reconnect: bool,
    /// Délai d'attente avant tentative de reconnexion automatique (millisecondes).
    /// Plage utile : 500 – 30 000.  Défaut : 2 000 ms.
    #[serde(default = "default_reconnect_delay_ms")]
    pub reconnect_delay_ms: u64,
}

/// Paramètres d'interface utilisateur.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSettings {
    pub language: String, // "auto" | "fr" | "en"
    pub theme: String,    // "light" | "dark" | "hacker"
    pub font_family: String,
    pub font_size: u32,
    pub window_width: i32,
    pub window_height: i32,
    pub show_sidebar: bool,
    pub stop_scroll: bool,
    pub show_line_numbers: bool,
    pub max_scrollback_lines: u32,
    pub line_ending: String, // "LF" | "CR" | "CRLF" | "None"
}

/// Paramètres de logging.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LogSettings {
    pub enabled: bool,
    pub level: String,
    pub log_to_file: bool,
    pub log_directory: String,
    #[serde(default)]
    pub timestamp_saved_lines: bool,
}

const fn default_reconnect_delay_ms() -> u64 {
    2_000
}

// =============================================================================
// Implémentations par défaut
// =============================================================================

impl Default for SerialSettings {
    fn default() -> Self {
        Self {
            port: String::new(),
            auto_select_single_port: true,
            baudrate: 115_200,
            data_bits: 8,
            parity: "None".to_string(),
            stop_bits: 1,
            flow_control: "None".to_string(),
            timeout_ms: 1000,
            auto_reconnect: false,
            reconnect_delay_ms: default_reconnect_delay_ms(),
        }
    }
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            language: "auto".to_string(),
            theme: "light".to_string(),
            font_family: "Monospace".to_string(),
            font_size: 11,
            window_width: MIN_WINDOW_WIDTH,
            window_height: MIN_WINDOW_HEIGHT,
            show_sidebar: false,
            stop_scroll: false,
            show_line_numbers: false,
            max_scrollback_lines: 10000,
            line_ending: "LF".to_string(),
        }
    }
}

impl Default for LogSettings {
    fn default() -> Self {
        // Utilise dirs::data_dir() pour un chemin absolu stable indépendant
        // du répertoire de travail courant.
        let log_directory = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("serial-term")
            .join("logs")
            .to_string_lossy()
            .into_owned();
        Self {
            enabled: true,
            level: "INFO".to_string(),
            log_to_file: false,
            log_directory,
            timestamp_saved_lines: true,
        }
    }
}

// =============================================================================
// Gestionnaire de configuration
// =============================================================================

/// Gestionnaire de configuration avec chargement/sauvegarde JSON.
#[derive(Debug, Clone)]
pub struct SettingsManager {
    settings: AppSettings,
    config_path: PathBuf,
    load_warning: Option<String>,
}

impl SettingsManager {
    /// Crée un nouveau gestionnaire en chargeant depuis le chemin par défaut.
    pub fn new() -> Self {
        let config_path = Self::default_config_path();
        let (settings, load_warning) = match Self::load_from_path(&config_path) {
            Ok(settings) => (settings, None),
            Err(error) if !config_path.exists() => {
                log::debug!("Configuration absente : {error}");
                (AppSettings::default(), None)
            }
            Err(error) => {
                let warning = format!("Configuration illisible : {error:#}. Valeurs par défaut utilisées ; sauvegarde .bak avant remplacement.");
                log::warn!("{warning}");
                (AppSettings::default(), Some(warning))
            }
        };
        Self {
            settings,
            config_path,
            load_warning,
        }
    }

    /// Chemin par défaut du fichier de configuration.
    fn default_config_path() -> PathBuf {
        #[cfg(test)]
        if let Some(path) = TEST_CONFIG_PATH_OVERRIDE.with(|slot| slot.borrow().clone()) {
            return path;
        }

        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("serial-term")
            .join("settings.json")
    }

    #[cfg(test)]
    pub fn set_test_config_path(path: PathBuf) {
        TEST_CONFIG_PATH_OVERRIDE.with(|slot| {
            *slot.borrow_mut() = Some(path);
        });
    }

    #[cfg(test)]
    pub fn clear_test_config_path() {
        TEST_CONFIG_PATH_OVERRIDE.with(|slot| {
            *slot.borrow_mut() = None;
        });
    }

    /// Charge la configuration depuis un fichier JSON.
    fn load_from_path(path: &PathBuf) -> Result<AppSettings> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Impossible de lire {}", path.display()))?;
        let settings: AppSettings =
            serde_json::from_str(&content).context("Format JSON invalide")?;
        log::info!("Configuration chargée depuis {}", path.display());
        Ok(settings)
    }

    /// Sauvegarde la configuration dans le fichier JSON.
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Impossible de créer {}", parent.display()))?;
        }
        let json =
            serde_json::to_string_pretty(&self.settings).context("Erreur de sérialisation JSON")?;
        // Préserver le JSON invalide avant toute tentative de remplacement.
        if let Ok(previous) = fs::read(&self.config_path) {
            if serde_json::from_slice::<AppSettings>(&previous).is_err() {
                fs::write(self.config_path.with_extension("json.bak"), previous)
                    .context("Impossible de sauvegarder la configuration invalide")?;
            }
        }
        let temp_path = self.config_path.with_extension(format!(
            "json.{}.{}.tmp",
            std::process::id(),
            NEXT_SAVE.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .context("Impossible de créer le fichier temporaire de configuration")?;
        let result = (|| -> Result<()> {
            file.write_all(json.as_bytes())?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temp_path, &self.config_path)?;
            #[cfg(unix)]
            if let Some(parent) = self.config_path.parent() {
                fs::File::open(parent)?.sync_all()?;
            }
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp_path);
        }
        result.with_context(|| format!("Impossible d'écrire {}", self.config_path.display()))?;
        log::info!(
            "Configuration sauvegardée dans {}",
            self.config_path.display()
        );
        Ok(())
    }

    /// Avertissement à présenter après la construction de l'interface.
    pub fn load_warning(&self) -> Option<&str> {
        self.load_warning.as_deref()
    }

    /// Accès en lecture aux paramètres.
    pub const fn settings(&self) -> &AppSettings {
        &self.settings
    }

    /// Accès en écriture aux paramètres.
    pub const fn settings_mut(&mut self) -> &mut AppSettings {
        &mut self.settings
    }

    /// Met à jour le thème et sauvegarde.
    pub fn set_theme(&mut self, theme: &str) {
        self.settings.ui.theme = theme.to_string();
        if let Err(e) = self.save() {
            log::warn!("Impossible de sauvegarder le thème : {e}");
        }
    }

    /// Met à jour la langue et sauvegarde.
    pub fn set_language(&mut self, lang: &str) {
        self.settings.ui.language = lang.to_string();
        if let Err(e) = self.save() {
            log::warn!("Impossible de sauvegarder la langue : {e}");
        }
    }

    /// Met à jour la taille de fenêtre.
    pub const fn set_window_size(&mut self, width: i32, height: i32) {
        self.settings.ui.window_width = clamp_window_width(width);
        self.settings.ui.window_height = clamp_window_height(height);
    }

    /// Met à jour la terminaison de ligne.
    pub fn set_line_ending(&mut self, ending: &str) {
        self.settings.ui.line_ending = ending.to_string();
        if let Err(e) = self.save() {
            log::warn!("Impossible de sauvegarder la terminaison de ligne : {e}");
        }
    }
}

// =============================================================================
// Tests unitaires
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_preserves_invalid_json_and_replaces_valid_config_atomically() {
        let root =
            std::env::temp_dir().join(format!("serial-term-settings-{}", std::process::id()));
        fs::create_dir_all(&root).expect("test directory");
        let path = root.join("settings.json");
        fs::write(&path, b"{invalid").expect("invalid fixture");
        SettingsManager::set_test_config_path(path.clone());
        let mut manager = SettingsManager::new();
        SettingsManager::clear_test_config_path();
        assert!(manager.load_warning().is_some());
        manager.settings_mut().serial.baudrate = 9600;
        manager.save().expect("atomic save");
        assert_eq!(
            fs::read(path.with_extension("json.bak")).expect("backup"),
            b"{invalid"
        );
        assert_eq!(
            SettingsManager::load_from_path(&path)
                .expect("valid config")
                .serial
                .baudrate,
            9600
        );
        manager.settings_mut().serial.baudrate = 115_200;
        manager.save().expect("replace existing file");
        assert_eq!(
            SettingsManager::load_from_path(&path)
                .expect("valid replacement")
                .serial
                .baudrate,
            115_200
        );
        assert_eq!(fs::read_dir(&root).expect("directory").count(), 2);
        fs::remove_dir_all(root).expect("cleanup");
    }

    // ── AppSettings defaults ──────────────────────────────────────────────────

    #[test]
    fn serial_settings_default_baudrate() {
        let s = SerialSettings::default();
        assert_eq!(s.baudrate, 115_200);
    }

    #[test]
    fn serial_settings_default_auto_select_single_port_true() {
        let s = SerialSettings::default();
        assert!(s.auto_select_single_port);
    }

    #[test]
    fn serial_settings_default_data_bits() {
        let s = SerialSettings::default();
        assert_eq!(s.data_bits, 8);
    }

    #[test]
    fn serial_settings_default_parity_none() {
        let s = SerialSettings::default();
        assert_eq!(s.parity, "None");
    }

    #[test]
    fn serial_settings_default_stop_bits_one() {
        let s = SerialSettings::default();
        assert_eq!(s.stop_bits, 1);
    }

    #[test]
    fn ui_settings_default_theme_light() {
        let s = UiSettings::default();
        assert_eq!(s.theme, "light");
    }

    #[test]
    fn ui_settings_default_sidebar_hidden() {
        let s = UiSettings::default();
        assert!(!s.show_sidebar);
    }

    #[test]
    fn ui_settings_default_stop_scroll_false() {
        let s = UiSettings::default();
        assert!(!s.stop_scroll);
    }

    #[test]
    fn ui_settings_default_max_scrollback_ten_thousand() {
        let s = UiSettings::default();
        assert_eq!(s.max_scrollback_lines, 10_000);
    }

    #[test]
    fn ui_settings_default_line_ending_lf() {
        let s = UiSettings::default();
        assert_eq!(s.line_ending, "LF");
    }

    #[test]
    fn log_settings_default_timestamp_true() {
        let s = LogSettings::default();
        assert!(s.timestamp_saved_lines);
    }

    // ── JSON round-trip ───────────────────────────────────────────────────────

    #[test]
    fn app_settings_json_round_trip() {
        let original = AppSettings::default();
        let json = serde_json::to_string(&original).expect("sérialisation échoue");
        let loaded: AppSettings = serde_json::from_str(&json).expect("désérialisation échoue");
        assert_eq!(
            loaded.serial.auto_select_single_port,
            original.serial.auto_select_single_port
        );
        assert_eq!(loaded.serial.baudrate, original.serial.baudrate);
        assert_eq!(loaded.ui.theme, original.ui.theme);
    }

    #[test]
    fn app_settings_json_round_trip_modified() {
        let mut original = AppSettings::default();
        original.serial.auto_select_single_port = false;
        original.serial.baudrate = 9600;
        original.ui.theme = "hacker".to_string();

        let json = serde_json::to_string_pretty(&original).expect("sérialisation échoue");
        let loaded: AppSettings = serde_json::from_str(&json).expect("désérialisation échoue");
        assert!(!loaded.serial.auto_select_single_port);
        assert_eq!(loaded.serial.baudrate, 9600);
        assert_eq!(loaded.ui.theme, "hacker");
    }

    // ── set_window_size ───────────────────────────────────────────────────────

    #[test]
    fn settings_manager_set_window_size() {
        let mut mgr = SettingsManager {
            settings: AppSettings::default(),
            config_path: std::path::PathBuf::new(),
            load_warning: None,
        };
        mgr.set_window_size(1920, 1080);
        assert_eq!(mgr.settings().ui.window_width, 1920);
        assert_eq!(mgr.settings().ui.window_height, 1080);
    }

    #[test]
    fn settings_manager_set_window_size_clamps_to_minimum() {
        let mut mgr = SettingsManager {
            settings: AppSettings::default(),
            config_path: std::path::PathBuf::new(),
            load_warning: None,
        };
        mgr.set_window_size(320, 240);
        assert_eq!(mgr.settings().ui.window_width, MIN_WINDOW_WIDTH);
        assert_eq!(mgr.settings().ui.window_height, MIN_WINDOW_HEIGHT);
    }
}
