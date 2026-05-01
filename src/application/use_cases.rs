use crate::core::serial_manager::SerialConfig;
use crate::core::settings::SerialSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerialConfigError {
    NoPortSelected,
}

#[derive(Debug, Clone)]
pub struct SerialConnectionInput {
    pub port: String,
    pub baudrate: u32,
    pub data_bits: u8,
    pub parity: String,
    pub stop_bits: u8,
    pub flow_control: String,
    pub timeout_ms: u64,
}

pub fn create_serial_config(
    input: &SerialConnectionInput,
) -> Result<SerialConfig, SerialConfigError> {
    if input.port.is_empty() {
        return Err(SerialConfigError::NoPortSelected);
    }

    Ok(SerialConfig::from_params(
        &input.port,
        input.baudrate,
        input.data_bits,
        &input.parity,
        input.stop_bits,
        &input.flow_control,
        input.timeout_ms,
    ))
}

pub fn apply_serial_settings(settings: &mut SerialSettings, input: &SerialConnectionInput) {
    settings.port.clone_from(&input.port);
    settings.baudrate = input.baudrate;
    settings.data_bits = input.data_bits;
    settings.parity.clone_from(&input.parity);
    settings.stop_bits = input.stop_bits;
    settings.flow_control.clone_from(&input.flow_control);
    settings.timeout_ms = input.timeout_ms;
}

pub fn build_terminal_payload(text: &str, line_ending: &str) -> Option<Vec<u8>> {
    if text.is_empty() {
        return None;
    }

    let mut data = Vec::with_capacity(text.len().saturating_add(line_ending.len()));
    data.extend_from_slice(text.as_bytes());
    data.extend_from_slice(line_ending.as_bytes());
    Some(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serial_config_successfully_maps_fields() {
        let input = SerialConnectionInput {
            port: "/dev/ttyUSB9".to_string(),
            baudrate: 57_600,
            data_bits: 7,
            parity: "Even".to_string(),
            stop_bits: 2,
            flow_control: "Hardware".to_string(),
            timeout_ms: 250,
        };

        let config = create_serial_config(&input).expect("serial config should be created");

        assert_eq!(config.port, "/dev/ttyUSB9");
        assert_eq!(config.baudrate, 57_600);
        assert_eq!(config.data_bits, tokio_serial::DataBits::Seven);
        assert_eq!(config.parity, tokio_serial::Parity::Even);
        assert_eq!(config.stop_bits, tokio_serial::StopBits::Two);
        assert_eq!(config.flow_control, tokio_serial::FlowControl::Hardware);
        assert_eq!(config.timeout, std::time::Duration::from_millis(250));
    }

    #[test]
    fn create_serial_config_rejects_empty_port() {
        let input = SerialConnectionInput {
            port: String::new(),
            baudrate: 9_600,
            data_bits: 8,
            parity: "None".to_string(),
            stop_bits: 1,
            flow_control: "None".to_string(),
            timeout_ms: 100,
        };

        assert!(matches!(
            create_serial_config(&input),
            Err(SerialConfigError::NoPortSelected)
        ));
    }

    #[test]
    fn apply_serial_settings_updates_all_fields() {
        let mut settings = SerialSettings::default();
        let input = SerialConnectionInput {
            port: "/dev/ttyS1".to_string(),
            baudrate: 9_600,
            data_bits: 5,
            parity: "Odd".to_string(),
            stop_bits: 2,
            flow_control: "Software".to_string(),
            timeout_ms: 700,
        };

        apply_serial_settings(&mut settings, &input);

        assert_eq!(settings.port, "/dev/ttyS1");
        assert_eq!(settings.baudrate, 9_600);
        assert_eq!(settings.data_bits, 5);
        assert_eq!(settings.parity, "Odd");
        assert_eq!(settings.stop_bits, 2);
        assert_eq!(settings.flow_control, "Software");
        assert_eq!(settings.timeout_ms, 700);
    }

    #[test]
    fn payload_none_when_empty_text() {
        assert!(build_terminal_payload("", "\n").is_none());
    }

    #[test]
    fn payload_concatenates_text_and_line_ending() {
        let payload = build_terminal_payload("show version", "\r\n").unwrap_or_default();
        assert_eq!(payload, b"show version\r\n");
    }
}
