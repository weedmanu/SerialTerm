use gtk4::{gio, Box as GtkBox, Label, Orientation};
use libadwaita::prelude::*;

use crate::ui::i18n::UiLang;

/// Fenêtre des raccourcis clavier (FR/EN).
pub(super) fn open_shortcuts_dialog(parent: &impl IsA<gtk4::Window>, lang: UiLang) {
    let dialog = libadwaita::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(lang.shortcuts_title())
        .default_width(560)
        .default_height(520)
        .build();

    let header = libadwaita::HeaderBar::new();

    let inner = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_start(18)
        .margin_end(18)
        .margin_top(12)
        .margin_bottom(18)
        .build();

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .min_content_height(320)
        .vexpand(true)
        .child(&inner)
        .build();

    let sections: &[(&str, &[(&str, &str)])] = &[
        (
            lang.app_menu_file(),
            &[
                (lang.new_terminal_label(), "Ctrl + N"),
                (lang.save_logs_label(), "Ctrl + S"),
                (lang.quit_label(), "Ctrl + Q"),
            ],
        ),
        (
            lang.app_menu_edit(),
            &[
                (lang.view_logs_label(), "Ctrl + Shift + L"),
                (lang.search_terminal_label(), "Ctrl + F"),
            ],
        ),
        (
            lang.shortcuts_toolbar_section(),
            &[
                (lang.shortcuts_clear_terminal(), "Ctrl + L"),
                (lang.shortcuts_next_match(), "F3"),
                (lang.shortcuts_previous_match(), "Shift + F3"),
            ],
        ),
        (
            lang.shortcuts_terminal_section(),
            &[
                (lang.shortcuts_copy_terminal_selection(), "Ctrl + Shift + C"),
                (lang.shortcuts_paste_input(), "Ctrl + Shift + V"),
            ],
        ),
        (
            lang.app_menu_tools(),
            &[(lang.calculator_converter_label(), "Ctrl + T")],
        ),
        (lang.app_menu_help(), &[(lang.shortcuts_title(), "F1")]),
    ];

    for (section_title, shortcuts) in sections {
        let group = libadwaita::PreferencesGroup::builder()
            .title(*section_title)
            .build();

        for (action, keys) in *shortcuts {
            let row = libadwaita::ActionRow::builder().title(*action).build();
            let kbd = Label::builder()
                .label(*keys)
                .valign(gtk4::Align::Center)
                .build();
            kbd.add_css_class("dim-label");
            kbd.add_css_class("monospace");
            row.add_suffix(&kbd);
            group.add(&row);
        }

        inner.append(&group);
    }

    let toolbar_view = libadwaita::ToolbarView::new();
    toolbar_view.add_top_bar(&header);
    toolbar_view.set_content(Some(&scrolled));
    dialog.set_content(Some(&toolbar_view));

    dialog.present();
}

/// Dialogue de contact (FR/EN) — affiche le mail et permet de l'ouvrir.
pub(super) fn show_contact_dialog(parent: &libadwaita::ApplicationWindow, lang: UiLang) {
    let dialog =
        libadwaita::AlertDialog::new(Some(lang.contact_label()), Some(lang.contact_dialog_body()));
    dialog.add_response("close", lang.close_label());
    dialog.add_response("mailto", lang.open_mail_client_label());
    dialog.set_default_response(Some("close"));
    dialog.set_response_appearance("mailto", libadwaita::ResponseAppearance::Suggested);

    dialog.connect_response(None, move |_, response| {
        if response == "mailto" {
            if let Err(e) = gio::AppInfo::launch_default_for_uri(
                "mailto:tutoelectroweb@gmail.com",
                None::<&gio::AppLaunchContext>,
            ) {
                log::warn!("Impossible d'ouvrir le client mail : {e}");
            }
        }
    });

    dialog.present(Some(parent));
}
