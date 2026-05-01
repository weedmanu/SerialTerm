// =============================================================================
// Fichier : tools_dialog.rs
// Rôle    : Fenêtre d'outils (calculatrice + convertisseur de base)
// =============================================================================

use anyhow::Context;
use fasteval::EmptyNamespace;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, DropDown, Entry, Label, Orientation, StringList};

use crate::ui::i18n::UiLang;

// Constructeur de dialogue GTK4 : widgets, signaux de conversion et mise en page
// dans un seul bloc pour que toutes les références de widgets restent valides.
#[allow(clippy::too_many_lines)]
pub fn open_tools_dialog(parent: &impl IsA<gtk4::Window>, lang: UiLang) {
    let dialog = gtk4::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(lang.tools_title())
        .default_width(520)
        .default_height(320)
        .build();

    let content = GtkBox::builder().orientation(Orientation::Vertical).build();
    content.set_spacing(12);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let title = Label::builder()
        .label(lang.tools_title())
        .xalign(0.0)
        .build();
    title.add_css_class("title-3");
    title.add_css_class("tools-title");

    // ---------------------------------------------------------------------
    // Calculatrice
    // ---------------------------------------------------------------------
    let calc_title = Label::builder()
        .label(lang.calculator_title())
        .xalign(0.0)
        .build();
    calc_title.add_css_class("heading");

    let calc_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();
    calc_card.add_css_class("tools-card");

    let calc_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let calc_entry = Entry::builder()
        .placeholder_text(lang.calculator_placeholder())
        .hexpand(true)
        .build();
    let calc_button = Button::builder().label(lang.calculate_label()).build();
    calc_button.add_css_class("suggested-action");
    calc_box.append(&calc_entry);
    calc_box.append(&calc_button);
    let calc_result = Label::builder()
        .label(lang.calculator_result_placeholder())
        .xalign(0.0)
        .build();
    calc_result.add_css_class("monospace");

    // ---------------------------------------------------------------------
    // Convertisseur DEC/HEX/BIN
    // ---------------------------------------------------------------------
    let conv_title = Label::builder()
        .label(lang.converter_title())
        .xalign(0.0)
        .build();
    conv_title.add_css_class("heading");

    let conv_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();
    conv_card.add_css_class("tools-card");

    let conv_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let base_model = StringList::new(&["DEC", "HEX", "BIN"]);
    let base_dropdown = DropDown::builder().model(&base_model).selected(0).build();

    let value_entry = Entry::builder()
        .placeholder_text(lang.value_to_convert_placeholder())
        .hexpand(true)
        .build();
    let convert_button = Button::builder().label(lang.convert_label()).build();
    convert_button.add_css_class("suggested-action");

    conv_row.append(&base_dropdown);
    conv_row.append(&value_entry);
    conv_row.append(&convert_button);

    let conv_dec = Label::builder().label("DEC: -").xalign(0.0).build();
    let conv_hex = Label::builder().label("HEX: -").xalign(0.0).build();
    let conv_bin = Label::builder().label("BIN: -").xalign(0.0).build();
    let conv_error = Label::builder().label("").xalign(0.0).build();
    conv_dec.add_css_class("monospace");
    conv_hex.add_css_class("monospace");
    conv_bin.add_css_class("monospace");
    conv_error.add_css_class("error");

    calc_card.append(&calc_title);
    calc_card.append(&calc_box);
    calc_card.append(&calc_result);

    conv_card.append(&conv_title);
    conv_card.append(&conv_row);
    conv_card.append(&conv_dec);
    conv_card.append(&conv_hex);
    conv_card.append(&conv_bin);
    conv_card.append(&conv_error);

    content.append(&title);
    content.append(&calc_card);
    content.append(&gtk4::Separator::new(Orientation::Horizontal));
    content.append(&conv_card);

    let actions = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .halign(gtk4::Align::End)
        .build();
    let close_button = Button::builder().label(lang.close_label()).build();
    actions.append(&close_button);
    content.append(&actions);

    {
        let calc_entry = calc_entry;
        let calc_result = calc_result;
        calc_button.connect_clicked(move |_| {
            let expression = calc_entry.text().trim().to_string();
            if expression.is_empty() {
                calc_result.set_label(lang.calculator_empty_expression());
                return;
            }

            let mut namespace = EmptyNamespace;
            match fasteval::ez_eval(&expression, &mut namespace) {
                Ok(value) => calc_result.set_label(&lang.calculator_result_value(value)),
                Err(e) => calc_result.set_label(&lang.calculator_result_error(&e.to_string())),
            }
        });
    }

    {
        let value_entry = value_entry;
        let base_dropdown = base_dropdown;
        let conv_dec = conv_dec;
        let conv_hex = conv_hex;
        let conv_bin = conv_bin;
        let conv_error = conv_error;

        convert_button.connect_clicked(move |_| {
            let input = value_entry.text().trim().to_string();
            if input.is_empty() {
                conv_error.set_label(lang.converter_empty_value_error());
                return;
            }

            let base = match base_dropdown.selected() {
                1 => 16,
                2 => 2,
                _ => 10,
            };

            match parse_signed_radix(&input, base) {
                Ok(value) => {
                    conv_dec.set_label(&format!("DEC: {value}"));
                    conv_hex.set_label(&format!("HEX: {}", format_hex(value)));
                    conv_bin.set_label(&format!("BIN: {}", format_bin(value)));
                    conv_error.set_label("");
                }
                Err(e) => conv_error.set_label(&lang.converter_error(&e.to_string())),
            }
        });
    }

    {
        let dialog = dialog.clone();
        close_button.connect_clicked(move |_| {
            dialog.close();
        });
    }

    dialog.set_child(Some(&content));
    dialog.present();
}

// `digits` est dérivé de l'entrée sans le `-` initial → toujours >= 0 : la négation est sûre.
#[allow(clippy::arithmetic_side_effects)]
fn parse_signed_radix(input: &str, base: u32) -> anyhow::Result<i128> {
    let raw = input.trim();
    let is_negative = raw.starts_with('-');
    let mut digits = if is_negative { &raw[1..] } else { raw };

    if base == 16 {
        digits = digits.trim_start_matches("0x").trim_start_matches("0X");
    } else if base == 2 {
        digits = digits.trim_start_matches("0b").trim_start_matches("0B");
    }

    let unsigned = i128::from_str_radix(digits, base)
        .with_context(|| format!("valeur invalide pour la base {base}"))?;

    if is_negative {
        Ok(-unsigned)
    } else {
        Ok(unsigned)
    }
}

fn format_hex(value: i128) -> String {
    if value < 0 {
        // `unsigned_abs()` retourne u128 : évite l'overflow à i128::MIN (arithmetic_side_effects).
        format!("-0x{:X}", value.unsigned_abs())
    } else {
        format!("0x{value:X}")
    }
}

fn format_bin(value: i128) -> String {
    if value < 0 {
        format!("-0b{:b}", value.unsigned_abs())
    } else {
        format!("0b{value:b}")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    // ── parse_signed_radix ────────────────────────────────────────────────────

    #[test]
    fn parse_dec_positive() {
        assert_eq!(parse_signed_radix("42", 10).unwrap(), 42);
        assert_eq!(parse_signed_radix("0", 10).unwrap(), 0);
        assert_eq!(parse_signed_radix("  255  ", 10).unwrap(), 255);
    }

    #[test]
    fn parse_dec_negative() {
        assert_eq!(parse_signed_radix("-1", 10).unwrap(), -1);
        assert_eq!(parse_signed_radix("-128", 10).unwrap(), -128);
    }

    #[test]
    fn parse_hex_with_prefix() {
        assert_eq!(parse_signed_radix("0xFF", 16).unwrap(), 255);
        assert_eq!(parse_signed_radix("0XFF", 16).unwrap(), 255);
        assert_eq!(parse_signed_radix("FF", 16).unwrap(), 255);
    }

    #[test]
    fn parse_hex_negative() {
        assert_eq!(parse_signed_radix("-0xFF", 16).unwrap(), -255);
    }

    #[test]
    fn parse_bin_with_prefix() {
        assert_eq!(parse_signed_radix("0b1010", 2).unwrap(), 10);
        assert_eq!(parse_signed_radix("0B1111", 2).unwrap(), 15);
        assert_eq!(parse_signed_radix("1111", 2).unwrap(), 15);
    }

    #[test]
    fn parse_bin_negative() {
        assert_eq!(parse_signed_radix("-0b10", 2).unwrap(), -2);
    }

    #[test]
    fn parse_invalid_returns_error() {
        assert!(parse_signed_radix("xyz", 10).is_err());
        assert!(parse_signed_radix("0xGG", 16).is_err());
        assert!(parse_signed_radix("0b2", 2).is_err());
        assert!(parse_signed_radix("", 10).is_err());
    }

    // ── format_hex ────────────────────────────────────────────────────────────

    #[test]
    fn format_hex_positive() {
        assert_eq!(format_hex(0), "0x0");
        assert_eq!(format_hex(255), "0xFF");
        assert_eq!(format_hex(256), "0x100");
    }

    #[test]
    fn format_hex_negative() {
        assert_eq!(format_hex(-1), "-0x1");
        assert_eq!(format_hex(-255), "-0xFF");
    }

    // ── format_bin ────────────────────────────────────────────────────────────

    #[test]
    fn format_bin_positive() {
        assert_eq!(format_bin(0), "0b0");
        assert_eq!(format_bin(1), "0b1");
        assert_eq!(format_bin(10), "0b1010");
    }

    #[test]
    fn format_bin_negative() {
        assert_eq!(format_bin(-1), "-0b1");
        assert_eq!(format_bin(-10), "-0b1010");
    }

    // ── round-trip DEC → HEX → parse ─────────────────────────────────────────

    #[test]
    fn round_trip_dec_to_hex_and_back() {
        let original = 1234_i128;
        let hex_str = format_hex(original);
        // format_hex donne "0x4D2" → parseable via base 16
        let trimmed = hex_str.trim_start_matches("0x");
        assert_eq!(parse_signed_radix(trimmed, 16).unwrap(), original);
    }
}
