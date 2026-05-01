//! Helpers partagés par `SerialPanel` et `ConnectionPanel`.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, DropDown, Label, Orientation};

use crate::core::serial_manager::BAUDRATE_MAX;
use crate::ui::i18n::UiLang;

pub(super) const SERIAL_PARITY_VALUES: [&str; 3] = ["None", "Odd", "Even"];
pub(super) const SERIAL_FLOW_CONTROL_VALUES: [&str; 3] = ["None", "Hardware", "Software"];

pub(super) const fn serial_parity_labels(lang: UiLang) -> [&'static str; 3] {
    [
        lang.serial_parity_none_label(),
        lang.serial_parity_odd_label(),
        lang.serial_parity_even_label(),
    ]
}

pub(super) const fn serial_flow_control_labels(lang: UiLang) -> [&'static str; 3] {
    [
        lang.serial_flow_none_label(),
        lang.serial_flow_hardware_label(),
        lang.serial_flow_software_label(),
    ]
}

pub(super) fn value_for_selected_index(dropdown: &DropDown, values: &[&str]) -> Option<String> {
    let idx = usize::try_from(dropdown.selected()).ok()?;
    values.get(idx).map(|value| (*value).to_string())
}

pub(super) fn set_dropdown_by_value(dropdown: &DropDown, values: &[&str], value: &str) {
    if let Some(idx) = values.iter().position(|candidate| *candidate == value) {
        dropdown.set_selected(u32::try_from(idx).unwrap_or(0));
    }
}

pub(super) fn parse_baudrate(value: &str) -> Option<u32> {
    let baudrate = value.trim().parse::<u32>().ok()?;
    if (1..=BAUDRATE_MAX).contains(&baudrate) {
        Some(baudrate)
    } else {
        None
    }
}

pub(super) fn append_row<W: IsA<gtk4::Widget>>(container: &GtkBox, label: &Label, widget: &W) {
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    widget.set_hexpand(true);
    row.append(label);
    row.append(widget);
    container.append(&row);
}
