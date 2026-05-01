// =============================================================================
// Fichier : connection_panel/mod.rs
// Rôle    : Panneau de connexion série
// =============================================================================

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation};

use crate::ui::i18n::UiLang;

mod common;
mod serial;

pub use self::serial::SerialPanel;

/// Panneau de connexion série.
pub struct ConnectionPanel {
    pub container: GtkBox,
    pub toolbar_stack: gtk4::Stack,
    pub serial_panel: SerialPanel,
}

impl ConnectionPanel {
    /// Crée le panneau de connexion série.
    pub fn new(lang: UiLang) -> Self {
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .build();

        let serial_panel = SerialPanel::new(lang);

        let toolbar_stack = gtk4::Stack::builder()
            .transition_type(gtk4::StackTransitionType::SlideUpDown)
            .interpolate_size(true)
            .build();

        toolbar_stack.add_titled(
            &serial_panel.toolbar_box,
            Some("serial"),
            lang.conn_serial_tab(),
        );

        container.append(&serial_panel.container);

        Self {
            container,
            toolbar_stack,
            serial_panel,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::common::parse_baudrate;
    use super::{ConnectionPanel, SerialPanel};
    use crate::core::serial_manager::SerialPortInfo;
    use crate::ui::i18n::UiLang;
    use gtk4::prelude::*;

    #[test]
    fn parse_baudrate_accepts_valid_non_standard_value() {
        assert_eq!(parse_baudrate("74880"), Some(74_880));
    }

    #[test]
    fn parse_baudrate_rejects_zero() {
        assert_eq!(parse_baudrate("0"), None);
    }

    #[test]
    fn parse_baudrate_rejects_out_of_range_value() {
        assert_eq!(parse_baudrate("4000001"), None);
    }

    #[gtk4::test]
    fn serial_dropdowns_show_localized_labels_but_keep_canonical_values() {
        let panel = SerialPanel::new(UiLang::Fr);

        panel.parity_dropdown.set_selected(2);
        panel.flowcontrol_dropdown.set_selected(1);

        assert_eq!(
            SerialPanel::dropdown_text(&panel.parity_dropdown).as_deref(),
            Some("Paire")
        );
        assert_eq!(panel.selected_parity(), "Even");
        assert_eq!(
            SerialPanel::dropdown_text(&panel.flowcontrol_dropdown).as_deref(),
            Some("Matériel (RTS/CTS)")
        );
        assert_eq!(panel.selected_flow_control(), "Hardware");
    }

    #[gtk4::test]
    fn serial_apply_settings_accepts_canonical_values_with_localized_ui() {
        let panel = SerialPanel::new(UiLang::Fr);

        panel.apply_settings(115_200, 8, "Odd", 1, "Software", 250, false, 2_000);

        assert_eq!(
            SerialPanel::dropdown_text(&panel.parity_dropdown).as_deref(),
            Some("Impaire")
        );
        assert_eq!(panel.selected_parity(), "Odd");
        assert_eq!(
            SerialPanel::dropdown_text(&panel.flowcontrol_dropdown).as_deref(),
            Some("Logiciel (XON/XOFF)")
        );
        assert_eq!(panel.selected_flow_control(), "Software");
        assert_eq!(panel.selected_timeout_ms(), 250);
    }

    #[gtk4::test]
    fn serial_snapshot_settings_includes_timeout_ms() {
        let panel = SerialPanel::new(UiLang::Fr);

        panel.apply_settings(57_600, 7, "Even", 2, "Hardware", 500, true, 3_000);

        let snapshot = panel.snapshot_settings();

        assert_eq!(snapshot.baudrate, 57_600);
        assert_eq!(snapshot.timeout_ms, 500);
        assert!(snapshot.auto_reconnect);
        assert_eq!(snapshot.reconnect_delay_ms, 3_000);
    }

    #[gtk4::test]
    fn serial_selected_port_prefers_stable_alias_for_connection_and_persistence() {
        let panel = SerialPanel::new(UiLang::Fr);
        let ports = vec![SerialPortInfo {
            device: "/dev/ttyACM0".to_string(),
            manufacturer: "STMicroelectronics".to_string(),
            description: "STM32 STLink".to_string(),
            friendly_name: "STM32 ST-LINK".to_string(),
            stable_path: "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_123-if02"
                .to_string(),
        }];

        panel.update_ports_from_list(&ports);

        assert_eq!(
            panel.selected_port().as_deref(),
            Some("/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_123-if02")
        );
        assert_eq!(
            panel.snapshot_settings().port,
            "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_123-if02"
        );
    }

    #[gtk4::test]
    fn serial_select_port_accepts_saved_stable_alias() {
        let panel = SerialPanel::new(UiLang::Fr);
        let ports = vec![SerialPortInfo {
            device: "/dev/ttyACM0".to_string(),
            manufacturer: "STMicroelectronics".to_string(),
            description: "STM32 STLink".to_string(),
            friendly_name: "STM32 ST-LINK".to_string(),
            stable_path: "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_123-if02"
                .to_string(),
        }];

        panel.update_ports_from_list(&ports);

        assert!(panel.select_port_by_device(
            "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_123-if02"
        ));
        assert_eq!(
            panel.selected_port().as_deref(),
            Some("/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_123-if02")
        );
    }

    #[gtk4::test]
    fn protocol_switch_stack_keeps_vertical_slide_with_size_interpolation() {
        let panel = ConnectionPanel::new(UiLang::Fr);

        assert_eq!(
            panel.toolbar_stack.transition_type(),
            gtk4::StackTransitionType::SlideUpDown
        );
        assert!(panel.toolbar_stack.property::<bool>("interpolate-size"));
    }

    #[gtk4::test]
    fn serial_ports_show_friendly_name_and_selected_tooltip() {
        let panel = SerialPanel::new(UiLang::Fr);
        let ports = vec![SerialPortInfo {
            device: "/dev/ttyACM0".to_string(),
            manufacturer: "STMicroelectronics".to_string(),
            description: "STM32 STLink".to_string(),
            friendly_name: "STM32 ST-LINK".to_string(),
            stable_path: "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_123-if02"
                .to_string(),
        }];

        panel.update_ports_from_list(&ports);

        assert_eq!(
            SerialPanel::dropdown_text(&panel.port_dropdown).as_deref(),
            Some("/dev/ttyACM0 — STM32 ST-LINK")
        );
        let tooltip = panel.port_dropdown.tooltip_text().unwrap_or_default();
        assert!(tooltip.contains("STM32 ST-LINK"));
        assert!(tooltip.contains("/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_123-if02"));
    }

    #[gtk4::test]
    fn serial_no_port_shows_explicit_placeholder_and_helpful_tooltip() {
        let panel_fr = SerialPanel::new(UiLang::Fr);
        panel_fr.update_ports_from_list(&[]);

        let label_fr = SerialPanel::dropdown_text(&panel_fr.port_dropdown).unwrap_or_default();
        assert!(
            label_fr.contains("Brancher"),
            "Le placeholder FR doit inviter à brancher un périphérique"
        );
        let tooltip_fr = panel_fr.port_dropdown.tooltip_text().unwrap_or_default();
        assert!(
            tooltip_fr.contains("rafraîchir"),
            "Le tooltip FR doit mentionner le rafraîchissement"
        );
        assert!(
            panel_fr.effective_port().is_none(),
            "effective_port doit être None sans port"
        );

        let panel_en = SerialPanel::new(UiLang::En);
        panel_en.update_ports_from_list(&[]);

        let label_en = SerialPanel::dropdown_text(&panel_en.port_dropdown).unwrap_or_default();
        assert!(
            label_en.contains("Connect"),
            "Le placeholder EN doit inviter à connecter un périphérique"
        );
        let tooltip_en = panel_en.port_dropdown.tooltip_text().unwrap_or_default();
        assert!(
            tooltip_en.contains("refresh"),
            "Le tooltip EN doit mentionner le refresh"
        );
    }
}
