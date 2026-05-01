//! Tests d'intégration GTK pour `MainWindow` (widgets, settings, lifecycle).

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use gtk4::gio::prelude::*;

use super::*;
use crate::core::settings::{AppSettings, SettingsManager};

static TEST_APP_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn drain_main_loop() {
    while gtk4::glib::MainContext::default().iteration(false) {}
}

fn close_window_for_tests(window: &Rc<MainWindow>) {
    if window.window.is_visible() {
        window.window.close();
    }
    drain_main_loop();
}

fn isolated_test_config_path(id: usize) -> PathBuf {
    std::env::temp_dir().join(format!("serial-term-tests/shell-{id}-settings.json"))
}

fn build_test_app() -> libadwaita::Application {
    let id = TEST_APP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let app = libadwaita::Application::builder()
        .application_id(format!(
            "io.github.tutoelectroweb.serialterm.shelltests{id}"
        ))
        .build();
    app.register(gtk4::gio::Cancellable::NONE)
        .expect("l'application GTK de test doit pouvoir s'enregistrer");
    app
}

fn build_window_for_tests() -> Rc<MainWindow> {
    let id = TEST_APP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = isolated_test_config_path(id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("le dossier temporaire de test doit être créé");
    }

    let mut seeded = AppSettings::default();
    seeded.ui.language = "fr".to_string();
    fs::write(
        &path,
        serde_json::to_string_pretty(&seeded).expect("la config de test doit se sérialiser"),
    )
    .expect("la config de test doit être écrite");

    SettingsManager::set_test_config_path(path);
    let app = build_test_app();
    let window = MainWindow::new(&app).expect("la fenêtre principale doit être construite");
    SettingsManager::clear_test_config_path();
    window
}

#[gtk4::test]
fn deferred_welcome_waits_for_window_mapping_and_runs_once() {
    crate::ui::runtime::sanitize_problematic_desktop_theme();

    let app = build_test_app();
    let window = libadwaita::ApplicationWindow::builder()
        .application(&app)
        .title("serial-term test")
        .default_width(640)
        .default_height(480)
        .build();
    let terminal = TerminalPanel::new(256, UiLang::Fr);
    let notice_label = Label::new(None);
    let notice_generation = Rc::new(Cell::new(0));

    window.set_content(Some(&terminal.container));
    defer_terminal_welcome_until_mapped(&window, &notice_label, &notice_generation, UiLang::Fr);

    for _ in 0..4 {
        drain_main_loop();
    }
    assert_eq!(terminal.get_text(), "");

    window.present();

    for _ in 0..8 {
        drain_main_loop();
    }

    assert_eq!(terminal.get_text(), "");

    let first_pass = notice_label.label();
    assert!(first_pass.contains(UiLang::Fr.welcome_line_1()));
    assert!(first_pass.contains(UiLang::Fr.welcome_line_2()));

    for _ in 0..4 {
        drain_main_loop();
    }

    assert_eq!(notice_label.label(), first_pass);
    window.close();
    drain_main_loop();
}

#[gtk4::test]
fn restore_ui_from_settings_matches_visible_widgets() {
    crate::ui::runtime::sanitize_problematic_desktop_theme();

    let window = build_window_for_tests();

    {
        let mut sm = window.settings.borrow_mut();
        let settings = sm.settings_mut();
        settings.serial.port = "/dev/ttyUSB42".to_string();
        settings.serial.auto_select_single_port = false;
        settings.serial.baudrate = 57_600;
        settings.serial.data_bits = 7;
        settings.serial.parity = "Even".to_string();
        settings.serial.stop_bits = 2;
        settings.serial.flow_control = "Software".to_string();
        settings.serial.timeout_ms = 250;
        settings.ui.theme = "hacker".to_string();
        settings.ui.language = "en".to_string();
        settings.ui.line_ending = "CRLF".to_string();
        settings.ui.stop_scroll = true;
        settings.ui.show_sidebar = true;
    }

    window.restore_ui_from_settings();
    while gtk4::glib::MainContext::default().iteration(false) {}

    assert_eq!(
        window.connection_panel.serial_panel.selected_baudrate(),
        57_600
    );
    assert_eq!(window.connection_panel.serial_panel.selected_data_bits(), 7);
    assert_eq!(
        window.connection_panel.serial_panel.selected_parity(),
        "Even"
    );
    assert_eq!(window.connection_panel.serial_panel.selected_stop_bits(), 2);
    assert_eq!(
        window.connection_panel.serial_panel.selected_flow_control(),
        "Software"
    );
    assert_eq!(
        window.connection_panel.serial_panel.selected_timeout_ms(),
        250
    );
    assert!(!window
        .connection_panel
        .serial_panel
        .auto_select_single_port_enabled());

    assert_eq!(window.input.line_ending_dropdown.selected(), 2);
    assert!(window.input.stop_scroll_enabled());
    assert!(!window.terminal.auto_scroll_handle().get());
    assert!(window.header.toggle_sidebar_button.is_active());

    let theme_action = window
        .window
        .lookup_action("set-theme")
        .expect("l'action theme doit exister");
    let theme_state = theme_action
        .state()
        .expect("l'action theme doit être stateful")
        .get::<String>()
        .expect("l'état theme doit être une chaîne");
    assert_eq!(theme_state, "hacker");

    let lang_action = window
        .window
        .lookup_action("set-language")
        .expect("l'action langue doit exister");
    let lang_state = lang_action
        .state()
        .expect("l'action langue doit être stateful")
        .get::<String>()
        .expect("l'état langue doit être une chaîne");
    assert_eq!(lang_state, "en");

    close_window_for_tests(&window);
}

#[gtk4::test]
fn sync_ui_state_to_settings_captures_current_widgets() {
    crate::ui::runtime::sanitize_problematic_desktop_theme();

    let window = build_window_for_tests();

    window
        .connection_panel
        .serial_panel
        .set_auto_select_single_port(false);
    window
        .connection_panel
        .serial_panel
        .apply_settings(230_400, 7, "Odd", 2, "Hardware", 350, true, 3_000);

    window.input.line_ending_dropdown.set_selected(3);
    window.input.set_stop_scroll_enabled(true);
    window.terminal.set_auto_scroll_enabled(false);
    window.header.toggle_sidebar_button.set_active(true);

    window.sync_ui_state_to_settings();

    let settings = window.settings.borrow();
    assert!(!settings.settings().serial.auto_select_single_port);
    assert_eq!(settings.settings().serial.baudrate, 230_400);
    assert_eq!(settings.settings().serial.data_bits, 7);
    assert_eq!(settings.settings().serial.parity, "Odd");
    assert_eq!(settings.settings().serial.stop_bits, 2);
    assert_eq!(settings.settings().serial.flow_control, "Hardware");
    assert_eq!(settings.settings().serial.timeout_ms, 350);

    assert_eq!(settings.settings().ui.line_ending, "None");
    assert!(settings.settings().ui.stop_scroll);
    assert!(settings.settings().ui.show_sidebar);

    drop(settings);
    close_window_for_tests(&window);
}
