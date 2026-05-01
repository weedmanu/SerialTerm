//! Panneau série (`SerialPanel`).

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CheckButton, DropDown, Label, Orientation, SpinButton, StringList,
};

use crate::core::serial_manager::{list_serial_ports, BAUDRATE_FALLBACK};
use crate::ui::i18n::UiLang;

use super::common::{
    append_row, parse_baudrate, serial_flow_control_labels, serial_parity_labels,
    set_dropdown_by_value, value_for_selected_index, SERIAL_FLOW_CONTROL_VALUES,
    SERIAL_PARITY_VALUES,
};

// =============================================================================
// Panneau de connexion série
// =============================================================================

/// Information interne d'un port pour retrouver le nom device à partir de l'index.
struct PortEntry {
    device: String,
    stable_path: String,
    tooltip: String,
}

impl PortEntry {
    fn preferred_connection_path(&self) -> &str {
        if self.stable_path.is_empty() {
            &self.device
        } else {
            &self.stable_path
        }
    }

    fn matches_saved_path(&self, path: &str) -> bool {
        self.device == path || (!self.stable_path.is_empty() && self.stable_path == path)
    }
}

fn build_port_dropdown_label(port: &crate::core::serial_manager::SerialPortInfo) -> String {
    format!("{} — {}", port.device, port.friendly_name)
}

fn build_port_dropdown_tooltip(
    lang: UiLang,
    port: &crate::core::serial_manager::SerialPortInfo,
) -> String {
    let mut lines = Vec::new();

    match lang {
        UiLang::Fr => {
            lines.push(format!("Port : {}", port.device));
            lines.push(format!("Périphérique : {}", port.friendly_name));
            if !port.description.is_empty() {
                lines.push(format!(
                    "Type USB : {}",
                    port.description.replace("STLink", "ST-LINK")
                ));
            }
            if !port.manufacturer.is_empty() {
                lines.push(format!("Fabricant : {}", port.manufacturer));
            }
            if !port.stable_path.is_empty() {
                lines.push(format!("Alias stable : {}", port.stable_path));
            }
        }
        UiLang::En => {
            lines.push(format!("Port: {}", port.device));
            lines.push(format!("Device: {}", port.friendly_name));
            if !port.description.is_empty() {
                lines.push(format!(
                    "USB type: {}",
                    port.description.replace("STLink", "ST-LINK")
                ));
            }
            if !port.manufacturer.is_empty() {
                lines.push(format!("Manufacturer: {}", port.manufacturer));
            }
            if !port.stable_path.is_empty() {
                lines.push(format!("Stable alias: {}", port.stable_path));
            }
        }
    }

    lines.join("\n")
}

/// Panneau de configuration de la connexion série.
pub struct SerialPanel {
    pub container: GtkBox,
    pub toolbar_box: GtkBox,
    pub port_dropdown: DropDown,
    pub auto_select_single_check: CheckButton,
    pub baud_dropdown: DropDown,
    pub databits_dropdown: DropDown,
    pub parity_dropdown: DropDown,
    pub stopbits_dropdown: DropDown,
    pub flowcontrol_dropdown: DropDown,
    pub timeout_spin: SpinButton,
    pub refresh_button: Button,
    pub auto_reconnect_check: CheckButton,
    pub reconnect_delay_spin: SpinButton,
    port_model: StringList,
    port_entries: std::rc::Rc<std::cell::RefCell<Vec<PortEntry>>>,
    lang: UiLang,
}

impl SerialPanel {
    // GTK4 impose de créer et d'assembler chaque widget dans le même bloc pour
    // que les références partagées (DropDown, Entry…) restent accessibles après
    // construction. Extraire des sous-fonctions nécessiterait de retourner des
    // tuples de widgets ou d'utiliser Rc supplémentaires — plus de complexité
    // pour aucun gain fonctionnel. La taille est donc justifiée par l'API GTK.
    #[allow(clippy::too_many_lines)]
    pub fn new(lang: UiLang) -> Self {
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .build();
        container.add_css_class("connection-panel");

        // Port série
        let port_model = StringList::new(&[]);
        let port_dropdown = DropDown::builder()
            .model(&port_model)
            .tooltip_text(lang.serial_port_tooltip())
            .build();

        let auto_select_single_check = CheckButton::builder()
            .label(lang.serial_auto_select_single_port())
            .tooltip_text(lang.serial_auto_select_single_port_tooltip())
            .active(true)
            .build();

        // Rafraîchir
        let refresh_button = Button::builder()
            .icon_name("view-refresh-symbolic")
            .tooltip_text(lang.serial_refresh_tooltip())
            .build();

        // Vitesse
        let baud_model = StringList::new(&[
            "9600", "19200", "38400", "57600", "115200", "230400", "460800", "921600",
        ]);
        let baud_dropdown = DropDown::builder()
            .model(&baud_model)
            .selected(4) // 115200
            .build();

        // Bits de données
        let databits_model = StringList::new(&["5", "6", "7", "8"]);
        let databits_dropdown = DropDown::builder()
            .model(&databits_model)
            .selected(3) // 8
            .build();

        // Parité
        let parity_labels = serial_parity_labels(lang);
        let parity_model = StringList::new(&parity_labels);
        let parity_dropdown = DropDown::builder().model(&parity_model).selected(0).build();

        // Stop bits
        let stopbits_model = StringList::new(&["1", "2"]);
        let stopbits_dropdown = DropDown::builder()
            .model(&stopbits_model)
            .selected(0)
            .build();

        // Flow control
        let flowcontrol_labels = serial_flow_control_labels(lang);
        let flowcontrol_model = StringList::new(&flowcontrol_labels);
        let flowcontrol_dropdown = DropDown::builder()
            .model(&flowcontrol_model)
            .selected(0)
            .build();

        let timeout_spin = SpinButton::with_range(1.0, 60_000.0, 10.0);
        timeout_spin.set_value(1_000.0);
        timeout_spin.set_digits(0);
        timeout_spin.set_width_chars(7);
        timeout_spin.set_tooltip_text(Some(lang.serial_timeout_tooltip()));

        // Reconnexion automatique
        let auto_reconnect_check = CheckButton::builder()
            .label(lang.serial_auto_reconnect_label())
            .tooltip_text(lang.serial_auto_reconnect_tooltip())
            .active(false)
            .build();

        let reconnect_delay_spin = SpinButton::with_range(500.0, 30_000.0, 500.0);
        reconnect_delay_spin.set_value(2_000.0);
        reconnect_delay_spin.set_digits(0);
        reconnect_delay_spin.set_width_chars(7);
        reconnect_delay_spin.set_tooltip_text(Some(lang.serial_reconnect_delay_tooltip()));
        reconnect_delay_spin.set_sensitive(false); // Désactivé tant que auto_reconnect est off

        // Activer/désactiver le spinner selon la case
        {
            let spin = reconnect_delay_spin.clone();
            auto_reconnect_check.connect_toggled(move |check| {
                spin.set_sensitive(check.is_active());
            });
        }

        // Layout
        let port_row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .build();
        port_dropdown.set_hexpand(true);
        // port_row.append(&port_label);
        port_row.append(&port_dropdown);
        port_row.append(&refresh_button);

        let toolbar_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .build();

        let port_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .build();
        port_box.append(&Label::new(Some(lang.serial_port_label())));
        port_box.append(&port_row);

        let baud_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .build();
        baud_box.append(&Label::new(Some(lang.serial_speed_label())));
        baud_box.append(&baud_dropdown);

        toolbar_box.append(&port_box);
        toolbar_box.append(&baud_box);

        // Paramètres avancés
        let adv_label = Label::builder()
            .label(lang.serial_advanced_label())
            .use_markup(true)
            .halign(gtk4::Align::Start)
            .build();
        container.append(&adv_label);

        container.append(&auto_select_single_check);

        let db_label = Label::new(Some(lang.serial_data_bits_label()));
        append_row(&container, &db_label, &databits_dropdown);

        let p_label = Label::new(Some(lang.serial_parity_label()));
        append_row(&container, &p_label, &parity_dropdown);

        let sb_label = Label::new(Some(lang.serial_stop_bits_label()));
        append_row(&container, &sb_label, &stopbits_dropdown);

        let fc_label = Label::new(Some(lang.serial_flow_label()));
        append_row(&container, &fc_label, &flowcontrol_dropdown);

        let timeout_label = Label::new(Some(lang.serial_timeout_label()));
        append_row(&container, &timeout_label, &timeout_spin);

        container.append(&auto_reconnect_check);

        let reconnect_delay_label = Label::new(Some(lang.serial_reconnect_delay_label()));
        append_row(&container, &reconnect_delay_label, &reconnect_delay_spin);

        let panel = Self {
            container,
            toolbar_box,
            port_dropdown,
            auto_select_single_check,
            baud_dropdown,
            databits_dropdown,
            parity_dropdown,
            stopbits_dropdown,
            flowcontrol_dropdown,
            timeout_spin,
            refresh_button,
            auto_reconnect_check,
            reconnect_delay_spin,
            port_model,
            port_entries: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
            lang,
        };

        {
            let lang = panel.lang;
            let entries = panel.port_entries.clone();
            panel
                .port_dropdown
                .connect_selected_notify(move |dropdown| {
                    let idx = usize::try_from(dropdown.selected()).unwrap_or(0);
                    let tooltip = entries
                        .borrow()
                        .get(idx)
                        .map(|entry| entry.tooltip.clone())
                        .filter(|tooltip| !tooltip.is_empty())
                        .unwrap_or_else(|| lang.serial_port_tooltip().to_string());
                    dropdown.set_tooltip_text(Some(&tooltip));
                });
            panel
                .port_dropdown
                .set_tooltip_text(Some(lang.serial_port_tooltip()));
        }

        panel.refresh_ports();
        panel
    }

    /// Rafraîchit la liste des ports série disponibles (appel bloquant dans le thread courant).
    ///
    /// ⚠️  Préférer `refresh_ports_async` depuis un handler UI pour ne pas bloquer le thread GTK.
    #[allow(clippy::as_conversions)]
    pub fn refresh_ports(&self) {
        let ports = list_serial_ports();
        self.update_ports_from_list(&ports);
    }

    /// Met à jour l'info-bulle du port actuellement sélectionné.
    fn update_selected_port_tooltip(&self) {
        let idx = usize::try_from(self.port_dropdown.selected()).unwrap_or(0);
        let tooltip = self
            .port_entries
            .borrow()
            .get(idx)
            .map(|entry| entry.tooltip.clone())
            .filter(|tooltip| !tooltip.is_empty())
            .unwrap_or_else(|| self.lang.serial_port_tooltip().to_string());
        self.port_dropdown.set_tooltip_text(Some(&tooltip));
    }

    /// Met à jour l'UI avec une liste de ports déjà collectée (thread GTK uniquement).
    pub fn update_ports_from_list(&self, ports: &[crate::core::serial_manager::SerialPortInfo]) {
        let previous_device = self.selected_port();

        // Vider le modèle existant
        self.port_model.splice(0, self.port_model.n_items(), &[]);

        let mut entries = Vec::new();

        if ports.is_empty() {
            self.port_model.append(self.lang.serial_no_port());
            entries.push(PortEntry {
                device: String::new(),
                stable_path: String::new(),
                tooltip: self.lang.serial_no_port_tooltip().to_string(),
            });
        } else {
            for port in ports {
                let label = build_port_dropdown_label(port);
                self.port_model.append(&label);
                entries.push(PortEntry {
                    device: port.device.clone(),
                    stable_path: port.stable_path.clone(),
                    tooltip: build_port_dropdown_tooltip(self.lang, port),
                });
            }
        }

        *self.port_entries.borrow_mut() = entries;

        if let Some(device) = previous_device {
            if self.select_port_by_device(&device) {
                self.update_selected_port_tooltip();
                log::info!("Port série conservé après rafraîchissement : {device}");
                log::info!("Ports série rafraîchis : {} trouvé(s)", ports.len());
                return;
            }
        }

        if ports.len() == 1 && self.auto_select_single_port_enabled() {
            self.port_dropdown.set_selected(0);
            self.update_selected_port_tooltip();
            if let Some(port) = ports.first() {
                log::info!("Port série auto-sélectionné : {}", port.device);
            }
            log::info!("Ports série rafraîchis : {} trouvé(s)", ports.len());
            return;
        }

        self.port_dropdown.set_selected(0);
        self.update_selected_port_tooltip();
        log::info!("Ports série rafraîchis : {} trouvé(s)", ports.len());
    }

    /// Retourne le port sélectionné (nom device).
    pub fn selected_port(&self) -> Option<String> {
        let idx = usize::try_from(self.port_dropdown.selected()).unwrap_or(0);
        let entries = self.port_entries.borrow();
        entries.get(idx).and_then(|e| {
            let path = e.preferred_connection_path();
            if path.is_empty() {
                None
            } else {
                Some(path.to_string())
            }
        })
    }

    /// Retourne le port à utiliser effectivement pour la connexion.
    pub fn effective_port(&self) -> Option<String> {
        self.selected_port()
    }

    /// Retourne si l'auto-sélection du port unique est active.
    pub fn auto_select_single_port_enabled(&self) -> bool {
        self.auto_select_single_check.is_active()
    }

    /// Définit l'état de l'auto-sélection du port unique.
    pub fn set_auto_select_single_port(&self, enabled: bool) {
        self.auto_select_single_check.set_active(enabled);
    }

    /// Helper pour lire la valeur textuelle d'un `DropDown` `StringList`.
    pub(super) fn dropdown_text(dropdown: &DropDown) -> Option<String> {
        let model = dropdown.model()?;
        let idx = dropdown.selected();
        let item = model.item(idx)?;
        let string_obj = item.downcast::<gtk4::StringObject>().ok()?;
        Some(string_obj.string().to_string())
    }

    /// Positionne un `DropDown` `StringList` sur une valeur textuelle donnée.
    fn set_dropdown_by_text(dropdown: &DropDown, value: &str) {
        let Some(model) = dropdown.model() else {
            return;
        };

        for idx in 0..model.n_items() {
            let Some(item) = model.item(idx) else {
                continue;
            };
            let Ok(string_obj) = item.downcast::<gtk4::StringObject>() else {
                continue;
            };
            if string_obj.string() == value {
                dropdown.set_selected(idx);
                return;
            }
        }
    }

    /// Retourne le baudrate sélectionné.
    pub fn selected_baudrate(&self) -> u32 {
        Self::dropdown_text(&self.baud_dropdown)
            .and_then(|s| parse_baudrate(&s))
            .unwrap_or(BAUDRATE_FALLBACK)
    }

    /// Retourne les data bits sélectionnés.
    pub fn selected_data_bits(&self) -> u8 {
        Self::dropdown_text(&self.databits_dropdown)
            .and_then(|s| s.parse().ok())
            .unwrap_or(8)
    }

    /// Retourne la parité sélectionnée.
    pub fn selected_parity(&self) -> String {
        value_for_selected_index(&self.parity_dropdown, &SERIAL_PARITY_VALUES)
            .unwrap_or_else(|| "None".to_string())
    }

    /// Retourne les stop bits sélectionnés.
    pub fn selected_stop_bits(&self) -> u8 {
        Self::dropdown_text(&self.stopbits_dropdown)
            .and_then(|s| s.parse().ok())
            .unwrap_or(1)
    }

    /// Retourne le flow control sélectionné.
    pub fn selected_flow_control(&self) -> String {
        value_for_selected_index(&self.flowcontrol_dropdown, &SERIAL_FLOW_CONTROL_VALUES)
            .unwrap_or_else(|| "None".to_string())
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::as_conversions
    )]
    pub fn selected_timeout_ms(&self) -> u64 {
        u64::try_from(self.timeout_spin.value_as_int()).unwrap_or(1_000)
    }

    pub fn set_timeout_ms(&self, timeout_ms: u64) {
        let timeout_ms = timeout_ms.clamp(1, 60_000);
        let timeout_ms = u32::try_from(timeout_ms).unwrap_or(1_000);
        self.timeout_spin.set_value(f64::from(timeout_ms));
    }

    /// Retourne si la reconnexion automatique est activée.
    pub fn auto_reconnect_enabled(&self) -> bool {
        self.auto_reconnect_check.is_active()
    }

    /// Active ou désactive la reconnexion automatique.
    pub fn set_auto_reconnect(&self, enabled: bool) {
        self.auto_reconnect_check.set_active(enabled);
        self.reconnect_delay_spin.set_sensitive(enabled);
    }

    /// Retourne le délai de reconnexion automatique en millisecondes.
    pub fn selected_reconnect_delay_ms(&self) -> u64 {
        u64::try_from(self.reconnect_delay_spin.value_as_int())
            .unwrap_or(2_000)
            .clamp(500, 30_000)
    }

    /// Définit le délai de reconnexion automatique en millisecondes.
    pub fn set_reconnect_delay_ms(&self, delay_ms: u64) {
        let delay_ms = delay_ms.clamp(500, 30_000);
        let delay_ms = u32::try_from(delay_ms).unwrap_or(2_000);
        self.reconnect_delay_spin.set_value(f64::from(delay_ms));
    }

    /// Sélectionne un port par son nom device s'il existe.
    pub fn select_port_by_device(&self, device: &str) -> bool {
        if device.is_empty() {
            return false;
        }

        let entries = self.port_entries.borrow();
        for (idx, entry) in entries.iter().enumerate() {
            if entry.matches_saved_path(device) {
                self.port_dropdown
                    .set_selected(u32::try_from(idx).unwrap_or(u32::MAX));
                return true;
            }
        }

        false
    }

    /// Applique les paramètres série à l'UI.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_settings(
        &self,
        baudrate: u32,
        data_bits: u8,
        parity: &str,
        stop_bits: u8,
        flow_control: &str,
        timeout_ms: u64,
        auto_reconnect: bool,
        reconnect_delay_ms: u64,
    ) {
        Self::set_dropdown_by_text(&self.baud_dropdown, &baudrate.to_string());
        Self::set_dropdown_by_text(&self.databits_dropdown, &data_bits.to_string());
        set_dropdown_by_value(&self.parity_dropdown, &SERIAL_PARITY_VALUES, parity);
        Self::set_dropdown_by_text(&self.stopbits_dropdown, &stop_bits.to_string());
        set_dropdown_by_value(
            &self.flowcontrol_dropdown,
            &SERIAL_FLOW_CONTROL_VALUES,
            flow_control,
        );
        self.set_timeout_ms(timeout_ms);
        self.set_auto_reconnect(auto_reconnect);
        self.set_reconnect_delay_ms(reconnect_delay_ms);
    }

    /// Capture l'état visible du panneau série dans un réglage persistable.
    pub fn snapshot_settings(&self) -> crate::core::settings::SerialSettings {
        crate::core::settings::SerialSettings {
            port: self.effective_port().unwrap_or_default(),
            auto_select_single_port: self.auto_select_single_port_enabled(),
            baudrate: self.selected_baudrate(),
            data_bits: self.selected_data_bits(),
            parity: self.selected_parity(),
            stop_bits: self.selected_stop_bits(),
            flow_control: self.selected_flow_control(),
            timeout_ms: self.selected_timeout_ms(),
            auto_reconnect: self.auto_reconnect_enabled(),
            reconnect_delay_ms: self.selected_reconnect_delay_ms(),
        }
    }
}
