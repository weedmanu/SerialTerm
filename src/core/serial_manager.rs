// =============================================================================
// Fichier : serial_manager.rs
// Rôle    : Gestionnaire de connexion série basé sur le trait Connection
// =============================================================================

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::{
    available_ports, ClearBuffer, DataBits, FlowControl, Parity, SerialPort, SerialPortBuilderExt,
    SerialPortType, SerialStream, StopBits,
};

use super::connection::{Connection, ConnectionState, ConnectionType};

const fn classify_serial_io_kind(kind: std::io::ErrorKind) -> ConnectionState {
    match kind {
        std::io::ErrorKind::BrokenPipe
        | std::io::ErrorKind::ConnectionAborted
        | std::io::ErrorKind::ConnectionReset
        | std::io::ErrorKind::NotConnected
        | std::io::ErrorKind::UnexpectedEof => ConnectionState::Disconnected,
        _ => ConnectionState::Error,
    }
}

#[cfg(unix)]
const fn classify_serial_unix_errno(raw_os_error: i32) -> ConnectionState {
    match raw_os_error {
        // Linux et BSD remontent fréquemment EIO/ENODEV/ENXIO lors d'un retrait à chaud.
        5 | 6 | 19 => ConnectionState::Disconnected,
        _ => ConnectionState::Error,
    }
}

fn classify_serial_io_error(error: &std::io::Error) -> ConnectionState {
    let kind_state = classify_serial_io_kind(error.kind());
    if kind_state == ConnectionState::Disconnected {
        return kind_state;
    }

    #[cfg(unix)]
    if let Some(raw_os_error) = error.raw_os_error() {
        let unix_state = classify_serial_unix_errno(raw_os_error);
        if unix_state == ConnectionState::Disconnected {
            return unix_state;
        }
    }

    kind_state
}

fn serial_io_context(action: &str, error: &std::io::Error) -> String {
    match classify_serial_io_error(error) {
        ConnectionState::Disconnected => {
            format!("Port série déconnecté pendant {action} : {error}")
        }
        ConnectionState::Error => format!("Erreur {action} série : {error}"),
        ConnectionState::Connecting | ConnectionState::Connected => {
            format!("Erreur {action} série : {error}")
        }
    }
}

/// Enrichit le message d'erreur d'ouverture de port avec une action corrective concrète.
///
/// Traduit les codes d'erreur `tokio_serial::ErrorKind` en messages compréhensibles :
/// - `PermissionDenied` → conseil groupe `dialout`
/// - `NotFound`         → alias `by-id` obsolète ou câble débranché
/// - `NoDevice`         → port verrouillé par un processus concurrent (EBUSY/TIOCEXCL)
/// - `InvalidInput`     → paramètre de configuration invalide
/// - _                  → erreur matérielle ou périphérique non initialisé
fn enrich_serial_open_error(port: &str, error: &tokio_serial::Error) -> String {
    use tokio_serial::ErrorKind;
    let hint: &str = match error.kind() {
        ErrorKind::Io(std::io::ErrorKind::PermissionDenied) => concat!(
            "\n  → Permission refusée : vérifiez que votre compte est membre du groupe 'dialout'",
            "\n    sudo usermod -aG dialout $USER  (puis fermer et rouvrir la session)"
        ),
        ErrorKind::Io(std::io::ErrorKind::NotFound) => concat!(
            "\n  → Périphérique introuvable : câble débranché ou alias by-id obsolète",
            "\n    Rebranchez la carte puis rafraîchissez la liste des ports (bouton ↺)"
        ),
        // EBUSY (POSIX) : TIOCEXCL déjà positionné par un autre processus
        ErrorKind::NoDevice => {
            "\n  → Port verrouillé : fermez les outils concurrents (minicom, screen, picocom, OpenOCD, GDB, st-link…)"
        }
        ErrorKind::InvalidInput => {
            "\n  → Paramètre invalide : vérifiez le baudrate et les options avancées"
        }
        _ => "\n  → Erreur matérielle ou périphérique non initialisé",
    };
    format!("Impossible d'ouvrir le port {port} : {error}{hint}")
}

// =============================================================================
// Information sur un port série
// =============================================================================

/// Décrit un port série détecté sur le système.
#[derive(Debug, Clone)]
pub struct SerialPortInfo {
    pub device: String,
    pub manufacturer: String,
    pub description: String,
    pub friendly_name: String,
    pub stable_path: String,
}

fn prettify_usb_text(text: &str) -> String {
    let prettified = text
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    prettified.replace("STLink", "ST-LINK")
}

fn stable_path_for_device(device: &str) -> String {
    let Some(device_name) = Path::new(device).file_name() else {
        return String::new();
    };
    let by_id_dir = Path::new("/dev/serial/by-id");
    let Ok(entries) = std::fs::read_dir(by_id_dir) else {
        return String::new();
    };

    let device_name = PathBuf::from(device_name);

    for entry in entries.flatten() {
        let alias_path = entry.path();
        let Ok(target_path) = std::fs::read_link(&alias_path) else {
            continue;
        };

        let resolved_name = alias_path
            .parent()
            .unwrap_or(by_id_dir)
            .join(target_path)
            .file_name()
            .map(PathBuf::from);

        if resolved_name.as_ref() == Some(&device_name) {
            return alias_path.to_string_lossy().to_string();
        }
    }

    String::new()
}

fn infer_friendly_name(
    device: &str,
    manufacturer: &str,
    description: &str,
    stable_path: &str,
) -> String {
    let description = prettify_usb_text(description);
    let manufacturer = prettify_usb_text(manufacturer);
    let stable_hint = prettify_usb_text(stable_path);

    if !description.is_empty() {
        return description;
    }

    if stable_hint.contains("ST-LINK") || manufacturer.contains("STMicroelectronics") {
        return "ST-LINK".to_string();
    }

    if !manufacturer.is_empty() {
        return manufacturer;
    }

    Path::new(device).file_name().map_or_else(
        || device.to_string(),
        |name| name.to_string_lossy().to_string(),
    )
}

fn is_linux_hotplug_device_name(device: &str) -> bool {
    let Some(device_name) = Path::new(device).file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    device_name.starts_with("ttyUSB")
        || device_name.starts_with("ttyACM")
        || device_name.starts_with("rfcomm")
}

fn should_list_serial_port(device: &str, port_type: &SerialPortType, stable_path: &str) -> bool {
    match port_type {
        SerialPortType::UsbPort(_) | SerialPortType::BluetoothPort => true,
        SerialPortType::PciPort | SerialPortType::Unknown => {
            #[cfg(target_os = "linux")]
            {
                !stable_path.is_empty() || is_linux_hotplug_device_name(device)
            }

            #[cfg(not(target_os = "linux"))]
            {
                let _ = (device, stable_path);
                true
            }
        }
    }
}

/// Liste les ports série disponibles sur le système.
pub fn list_serial_ports() -> Vec<SerialPortInfo> {
    match available_ports() {
        Ok(ports) => ports
            .into_iter()
            .filter_map(|p| {
                let (manufacturer, description) = match &p.port_type {
                    SerialPortType::UsbPort(info) => (
                        info.manufacturer.clone().unwrap_or_default(),
                        info.product.clone().unwrap_or_default(),
                    ),
                    _ => (String::new(), String::new()),
                };
                let stable_path = stable_path_for_device(&p.port_name);
                if !should_list_serial_port(&p.port_name, &p.port_type, &stable_path) {
                    log::debug!(
                        "Port série ignoré dans la liste auto : {} ({:?})",
                        p.port_name,
                        p.port_type
                    );
                    return None;
                }
                let friendly_name =
                    infer_friendly_name(&p.port_name, &manufacturer, &description, &stable_path);
                Some(SerialPortInfo {
                    device: p.port_name,
                    manufacturer,
                    description,
                    friendly_name,
                    stable_path,
                })
            })
            .collect(),
        Err(e) => {
            log::warn!("Impossible d'énumérer les ports série : {e}");
            Vec::new()
        }
    }
}

#[cfg(test)]
mod serial_port_info_tests {
    use super::{infer_friendly_name, should_list_serial_port};
    use tokio_serial::SerialPortType;

    #[test]
    fn infer_friendly_name_prefers_description() {
        assert_eq!(
            infer_friendly_name(
                "/dev/ttyACM0",
                "STMicroelectronics",
                "STM32 STLink",
                "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_123-if02"
            ),
            "STM32 ST-LINK"
        );
    }

    #[test]
    fn infer_friendly_name_falls_back_to_st_link_for_st_devices_without_product() {
        assert_eq!(
            infer_friendly_name(
                "/dev/ttyACM0",
                "STMicroelectronics",
                "",
                "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_123-if02"
            ),
            "ST-LINK"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn hides_linux_legacy_ttys_without_hotplug_metadata() {
        assert!(!should_list_serial_port(
            "/dev/ttyS15",
            &SerialPortType::PciPort,
            ""
        ));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn keeps_linux_usb_serial_ports_visible() {
        assert!(should_list_serial_port(
            "/dev/ttyACM0",
            &SerialPortType::Unknown,
            ""
        ));
        assert!(should_list_serial_port(
            "/dev/ttyUSB0",
            &SerialPortType::PciPort,
            ""
        ));
    }

    #[test]
    fn keeps_usb_typed_ports_visible() {
        let port_type = SerialPortType::UsbPort(tokio_serial::UsbPortInfo {
            vid: 0x0483,
            pid: 0x5740,
            serial_number: None,
            manufacturer: Some("STMicroelectronics".to_string()),
            product: Some("STM32 STLink".to_string()),
        });

        assert!(should_list_serial_port("/dev/ttyACM0", &port_type, ""));
    }
}

// =============================================================================
// Gestionnaire de connexion série
// =============================================================================

/// Configuration d'une connexion série.
#[derive(Debug, Clone)]
pub struct SerialConfig {
    pub port: String,
    pub baudrate: u32,
    pub data_bits: DataBits,
    pub parity: Parity,
    pub stop_bits: StopBits,
    pub flow_control: FlowControl,
    pub timeout: Duration,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            port: String::new(),
            baudrate: 115_200,
            data_bits: DataBits::Eight,
            parity: Parity::None,
            stop_bits: StopBits::One,
            flow_control: FlowControl::None,
            timeout: Duration::from_millis(10),
        }
    }
}

/// Baudrate de repli utilisé quand la valeur fournie est hors plage.
pub const BAUDRATE_FALLBACK: u32 = 115_200;

/// Plage acceptable pour un baudrate série (1 baud – 4 Mbaud).
pub const BAUDRATE_MAX: u32 = 4_000_000;

impl SerialConfig {
    /// Construit la configuration à partir des paramètres utilisateur.
    ///
    /// Si `baudrate` est 0 ou dépasse `BAUDRATE_MAX` (4 Mbaud), la valeur est
    /// remplacée par `BAUDRATE_FALLBACK` (115200) et un warning est loggué.
    pub fn from_params(
        port: &str,
        baudrate: u32,
        data_bits: u8,
        parity: &str,
        stop_bits: u8,
        flow_control: &str,
        timeout_ms: u64,
    ) -> Self {
        let baudrate = if baudrate == 0 || baudrate > BAUDRATE_MAX {
            log::warn!("Baudrate invalide ({baudrate}), repli sur {BAUDRATE_FALLBACK} bauds");
            BAUDRATE_FALLBACK
        } else {
            baudrate
        };
        Self {
            port: port.to_string(),
            baudrate,
            data_bits: match data_bits {
                5 => DataBits::Five,
                6 => DataBits::Six,
                7 => DataBits::Seven,
                _ => DataBits::Eight,
            },
            parity: match parity {
                "Odd" => Parity::Odd,
                "Even" => Parity::Even,
                _ => Parity::None,
            },
            stop_bits: match stop_bits {
                2 => StopBits::Two,
                _ => StopBits::One,
            },
            flow_control: match flow_control {
                "Hardware" => FlowControl::Hardware,
                "Software" => FlowControl::Software,
                _ => FlowControl::None,
            },
            timeout: Duration::from_millis(timeout_ms),
        }
    }
}

/// Gestionnaire de connexion série implémentant le trait `Connection`.
pub struct SerialManager {
    config: SerialConfig,
    port: Option<SerialStream>,
    state: ConnectionState,
    bytes_sent: u64,
    bytes_received: u64,
    /// Buffer de lecture réutilisable — évite une allocation + memset par trame reçue.
    read_buf: Vec<u8>,
}

impl SerialManager {
    /// Crée un nouveau gestionnaire avec la configuration donnée.
    pub fn new(config: SerialConfig) -> Self {
        Self {
            config,
            port: None,
            state: ConnectionState::Disconnected,
            bytes_sent: 0,
            bytes_received: 0,
            // Capacité initiale de 4096 octets : évite les réallocations pour les trames courantes.
            read_buf: Vec::with_capacity(4096),
        }
    }
}

#[async_trait]
impl Connection for SerialManager {
    async fn connect(&mut self) -> Result<()> {
        if self.state == ConnectionState::Connected {
            bail!("Déjà connecté à {}", self.config.port);
        }

        self.state = ConnectionState::Connecting;
        log::info!(
            "Connexion série vers {} @ {}...",
            self.config.port,
            self.config.baudrate
        );

        let port = tokio_serial::new(&self.config.port, self.config.baudrate)
            .data_bits(self.config.data_bits)
            .parity(self.config.parity)
            .stop_bits(self.config.stop_bits)
            .flow_control(self.config.flow_control)
            .timeout(self.config.timeout)
            .open_native_async()
            .map_err(|e| {
                // Transition état avant de propager l'erreur : sinon state resterait
                // Connecting, rendant la machine d'état incohérente pour les appels suivants.
                self.state = ConnectionState::Error;
                anyhow::anyhow!("{}", enrich_serial_open_error(&self.config.port, &e))
            })?;

        // Vider le buffer RX pour éviter de lire des données stale d'une session précédente
        // (reconnexion rapide sans arrachage physique, ou données en attente à l'ouverture).
        if let Err(e) = port.clear(ClearBuffer::Input) {
            log::warn!("Impossible de vider le buffer RX série à l'ouverture : {e}");
        }

        self.port = Some(port);
        self.state = ConnectionState::Connected;
        self.bytes_sent = 0;
        self.bytes_received = 0;
        log::info!("Connecté à {} @ {}", self.config.port, self.config.baudrate);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if self.state == ConnectionState::Disconnected {
            return Ok(());
        }

        log::info!("Déconnexion série de {}...", self.config.port);

        // Flush le buffer TX avant de fermer le port pour vider les données en attente.
        // On ignore les erreurs ici : si le port est déjà arraché (EIO), le flush
        // échoue de toute façon et on veut quand même fermer proprement.
        if let Some(port) = self.port.as_mut() {
            if let Err(e) = port.flush().await {
                log::debug!("Flush série ignoré à la déconnexion : {e}");
            }
        }

        self.port = None; // Drop ferme le port (l'OS libère les ressources)
        self.state = ConnectionState::Disconnected;
        log::info!(
            "Déconnecté de {} (envoyés: {} octets, reçus: {} octets)",
            self.config.port,
            self.bytes_sent,
            self.bytes_received
        );
        Ok(())
    }

    async fn send(&mut self, data: &[u8]) -> Result<usize> {
        let port = self.port.as_mut().context("Port série non connecté")?;

        let written = match port.write(data).await {
            Ok(written) => written,
            Err(error) => {
                self.state = classify_serial_io_error(&error);
                bail!(serial_io_context("l'écriture", &error));
            }
        };

        if let Err(error) = port.flush().await {
            self.state = classify_serial_io_error(&error);
            bail!(serial_io_context("le flush", &error));
        }

        self.bytes_sent = self
            .bytes_sent
            .saturating_add(u64::try_from(written).unwrap_or(0));
        Ok(written)
    }

    async fn read(&mut self) -> Result<Vec<u8>> {
        let port = self.port.as_mut().context("Port série non connecté")?;

        // Réutilise le buffer pré-alloué : pas d'alloc ni de memset sur le chemin chaud.
        // resize() est un no-op quand len == 4096 (cas nominal après la première lecture) ;
        // il n'initialise que les octets manquants si len < 4096 (premier appel ou readapt.).
        // Évite la combinaison clear()+resize() qui zéro-initialisait systématiquement
        // les 4096 octets à chaque itération de lecture.
        self.read_buf.resize(4096, 0);

        match port.read(&mut self.read_buf).await {
            Ok(0) => {
                // EOF
                self.state = ConnectionState::Disconnected;
                Ok(Vec::new())
            }
            Ok(n) => {
                self.bytes_received = self
                    .bytes_received
                    .saturating_add(u64::try_from(n).unwrap_or(0));
                Ok(self.read_buf[..n].to_vec())
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(Vec::new()),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(Vec::new()),
            Err(e) => {
                self.state = classify_serial_io_error(&e);
                bail!(serial_io_context("la lecture", &e))
            }
        }
    }

    fn state(&self) -> ConnectionState {
        self.state
    }

    fn connection_type(&self) -> ConnectionType {
        ConnectionType::Serial
    }

    fn description(&self) -> String {
        format!("{} @ {}", self.config.port, self.config.baudrate)
    }

    fn bytes_sent(&self) -> u64 {
        self.bytes_sent
    }

    fn bytes_received(&self) -> u64 {
        self.bytes_received
    }
}

// =============================================================================
// Tests unitaires
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── SerialConfig::default() ───────────────────────────────────────────────

    #[test]
    fn default_baudrate_is_115200() {
        let c = SerialConfig::default();
        assert_eq!(c.baudrate, 115_200);
    }

    #[test]
    fn default_data_bits_is_eight() {
        let c = SerialConfig::default();
        assert_eq!(c.data_bits, DataBits::Eight);
    }

    #[test]
    fn default_parity_is_none() {
        let c = SerialConfig::default();
        assert_eq!(c.parity, Parity::None);
    }

    #[test]
    fn default_stop_bits_is_one() {
        let c = SerialConfig::default();
        assert_eq!(c.stop_bits, StopBits::One);
    }

    #[test]
    fn default_flow_control_is_none() {
        let c = SerialConfig::default();
        assert_eq!(c.flow_control, FlowControl::None);
    }

    #[test]
    fn classify_disconnect_error_kinds_as_disconnected() {
        for kind in [
            std::io::ErrorKind::BrokenPipe,
            std::io::ErrorKind::ConnectionAborted,
            std::io::ErrorKind::ConnectionReset,
            std::io::ErrorKind::NotConnected,
            std::io::ErrorKind::UnexpectedEof,
        ] {
            assert_eq!(classify_serial_io_kind(kind), ConnectionState::Disconnected);
        }
    }

    #[test]
    fn classify_non_disconnect_error_kinds_as_error() {
        for kind in [
            std::io::ErrorKind::Other,
            std::io::ErrorKind::InvalidInput,
            std::io::ErrorKind::PermissionDenied,
        ] {
            assert_eq!(classify_serial_io_kind(kind), ConnectionState::Error);
        }
    }

    #[cfg(unix)]
    #[test]
    fn classify_hot_unplug_errno_as_disconnected() {
        for raw_os_error in [5, 6, 19] {
            let error = std::io::Error::from_raw_os_error(raw_os_error);
            assert_eq!(
                classify_serial_io_error(&error),
                ConnectionState::Disconnected
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn serial_hot_unplug_context_mentions_disconnection() {
        let error = std::io::Error::from_raw_os_error(5);
        let context = serial_io_context("la lecture", &error);
        assert!(context.contains("Port série déconnecté"));
    }

    // ── enrich_serial_open_error() ────────────────────────────────────────────

    #[test]
    fn enrich_open_error_includes_port_name() {
        let error = tokio_serial::Error::new(tokio_serial::ErrorKind::Unknown, "test error");
        let msg = enrich_serial_open_error("/dev/ttyACM0", &error);
        assert!(msg.contains("/dev/ttyACM0"));
    }

    #[test]
    fn enrich_open_error_permission_denied_mentions_dialout() {
        let error = tokio_serial::Error::new(
            tokio_serial::ErrorKind::Io(std::io::ErrorKind::PermissionDenied),
            "permission denied",
        );
        let msg = enrich_serial_open_error("/dev/ttyACM0", &error);
        assert!(
            msg.contains("dialout"),
            "devrait mentionner le groupe dialout"
        );
        assert!(msg.contains("usermod"), "devrait suggérer usermod");
    }

    #[test]
    fn enrich_open_error_not_found_mentions_unplug() {
        let error = tokio_serial::Error::new(
            tokio_serial::ErrorKind::Io(std::io::ErrorKind::NotFound),
            "no such file",
        );
        let msg = enrich_serial_open_error("/dev/ttyACM0", &error);
        assert!(
            msg.contains("introuvable"),
            "devrait mentionner 'introuvable'"
        );
        assert!(msg.contains("↺"), "devrait suggérer de rafraîchir la liste");
    }

    #[test]
    fn enrich_open_error_no_device_mentions_lock() {
        let error = tokio_serial::Error::new(tokio_serial::ErrorKind::NoDevice, "device busy");
        let msg = enrich_serial_open_error("/dev/ttyACM0", &error);
        assert!(
            msg.contains("verrouillé"),
            "devrait mentionner le verrouillage"
        );
    }

    #[test]
    fn enrich_open_error_invalid_input_mentions_params() {
        let error = tokio_serial::Error::new(tokio_serial::ErrorKind::InvalidInput, "invalid baud");
        let msg = enrich_serial_open_error("/dev/ttyACM0", &error);
        assert!(
            msg.contains("invalide"),
            "devrait mentionner le paramètre invalide"
        );
    }

    #[test]
    fn enrich_open_error_unknown_mentions_hardware() {
        let error = tokio_serial::Error::new(tokio_serial::ErrorKind::Unknown, "unknown io error");
        let msg = enrich_serial_open_error("/dev/ttyACM0", &error);
        assert!(
            msg.contains("matériel"),
            "devrait mentionner l'erreur matérielle"
        );
    }

    // ── SerialConfig::from_params() ───────────────────────────────────────────

    #[test]
    fn from_params_five_data_bits() {
        let c = SerialConfig::from_params("/dev/ttyUSB0", 9600, 5, "None", 1, "None", 100);
        assert_eq!(c.data_bits, DataBits::Five);
    }

    #[test]
    fn from_params_six_data_bits() {
        let c = SerialConfig::from_params("/dev/ttyUSB0", 9600, 6, "None", 1, "None", 100);
        assert_eq!(c.data_bits, DataBits::Six);
    }

    #[test]
    fn from_params_seven_data_bits() {
        let c = SerialConfig::from_params("/dev/ttyUSB0", 9600, 7, "None", 1, "None", 100);
        assert_eq!(c.data_bits, DataBits::Seven);
    }

    #[test]
    fn from_params_eight_data_bits_default() {
        // Toute valeur autre que 5, 6, 7 → 8 bits
        let c = SerialConfig::from_params("/dev/ttyUSB0", 9600, 8, "None", 1, "None", 100);
        assert_eq!(c.data_bits, DataBits::Eight);
    }

    #[test]
    fn from_params_odd_parity() {
        let c = SerialConfig::from_params("/dev/ttyUSB0", 115_200, 8, "Odd", 1, "None", 100);
        assert_eq!(c.parity, Parity::Odd);
    }

    #[test]
    fn from_params_even_parity() {
        let c = SerialConfig::from_params("/dev/ttyUSB0", 115_200, 8, "Even", 1, "None", 100);
        assert_eq!(c.parity, Parity::Even);
    }

    #[test]
    fn from_params_two_stop_bits() {
        let c = SerialConfig::from_params("/dev/ttyUSB0", 115_200, 8, "None", 2, "None", 100);
        assert_eq!(c.stop_bits, StopBits::Two);
    }

    #[test]
    fn from_params_hardware_flow_control() {
        let c = SerialConfig::from_params("/dev/ttyUSB0", 115_200, 8, "None", 1, "Hardware", 100);
        assert_eq!(c.flow_control, FlowControl::Hardware);
    }

    #[test]
    fn from_params_software_flow_control() {
        let c = SerialConfig::from_params("/dev/ttyUSB0", 115_200, 8, "None", 1, "Software", 100);
        assert_eq!(c.flow_control, FlowControl::Software);
    }

    #[test]
    fn from_params_sets_port() {
        let c = SerialConfig::from_params("/dev/ttyS0", 9600, 8, "None", 1, "None", 100);
        assert_eq!(c.port, "/dev/ttyS0");
    }

    #[test]
    fn from_params_sets_timeout_ms() {
        let c = SerialConfig::from_params("/dev/ttyUSB0", 9600, 8, "None", 1, "None", 500);
        assert_eq!(c.timeout, std::time::Duration::from_millis(500));
    }

    // ── Validation baudrate ───────────────────────────────────────────────────

    #[test]
    fn from_params_zero_baudrate_falls_back() {
        let c = SerialConfig::from_params("/dev/ttyUSB0", 0, 8, "None", 1, "None", 100);
        assert_eq!(c.baudrate, BAUDRATE_FALLBACK);
    }

    #[test]
    fn from_params_over_max_baudrate_falls_back() {
        let c = SerialConfig::from_params("/dev/ttyUSB0", 5_000_000, 8, "None", 1, "None", 100);
        assert_eq!(c.baudrate, BAUDRATE_FALLBACK);
    }

    #[test]
    fn from_params_valid_baudrate_kept() {
        let c = SerialConfig::from_params("/dev/ttyUSB0", 500_000, 8, "None", 1, "None", 100);
        assert_eq!(c.baudrate, 500_000);
    }

    #[test]
    fn from_params_max_baudrate_kept() {
        let c = SerialConfig::from_params("/dev/ttyUSB0", BAUDRATE_MAX, 8, "None", 1, "None", 100);
        assert_eq!(c.baudrate, BAUDRATE_MAX);
    }
}
