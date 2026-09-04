use std::rc::Rc;
use std::time::Duration;

use gtk4::glib;
use gtk4::prelude::*;

use crate::core::connection::ConnectionCommand;

use super::shell::MainWindow;

impl MainWindow {
    /// Connecte tous les signaux UI (boutons, entrées, close-request).
    #[allow(clippy::clone_on_ref_ptr, clippy::too_many_lines)]
    pub(super) fn setup_signals(win: &Rc<Self>) {
        {
            let w = Rc::downgrade(win);
            win.connect_button.connect_clicked(move |_| {
                let Some(w) = w.upgrade() else {
                    return;
                };
                w.toggle_connection();
            });
        }

        {
            let w = Rc::downgrade(win);
            win.clear_button.connect_clicked(move |_| {
                let Some(w) = w.upgrade() else {
                    return;
                };
                w.terminal.clear();
                w.show_bottom_notice(w.lang.terminal_cleared());
            });
        }

        {
            let w = Rc::downgrade(win);
            win.connection_panel
                .serial_panel
                .refresh_button
                .connect_clicked(move |_| {
                    let Some(w) = w.upgrade() else {
                        return;
                    };

                    let refresh_button = w.connection_panel.serial_panel.refresh_button.clone();
                    refresh_button.set_sensitive(false);
                    refresh_button.set_icon_name("process-working-symbolic");
                    w.show_toast(w.lang.serial_ports_refreshing());

                    let (sender, receiver) = async_channel::bounded::<
                        Vec<crate::core::serial_manager::SerialPortInfo>,
                    >(1);

                    glib::spawn_future_local(async move {
                        if let Ok(ports) = receiver.recv().await {
                            w.connection_panel
                                .serial_panel
                                .refresh_button
                                .set_sensitive(true);
                            w.connection_panel
                                .serial_panel
                                .refresh_button
                                .set_icon_name("view-refresh-symbolic");
                            w.connection_panel
                                .serial_panel
                                .update_ports_from_list(&ports);
                            w.show_bottom_notice(w.lang.serial_ports_refreshed());
                            w.show_toast(w.lang.serial_ports_refreshed());
                        }
                    });

                    std::thread::spawn(move || {
                        let ports = crate::core::serial_manager::list_serial_ports();
                        let _ = sender.try_send(ports);
                    });
                });
        }

        {
            let w = Rc::downgrade(win);
            win.input.send_button.connect_clicked(move |_| {
                let Some(w) = w.upgrade() else {
                    return;
                };
                w.send_data();
            });
        }

        {
            let w = Rc::downgrade(win);
            win.input.entry.connect_activate(move |_| {
                let Some(w) = w.upgrade() else {
                    return;
                };
                w.send_data();
            });
        }

        {
            let w = Rc::downgrade(win);
            win.header.save_log_button.connect_clicked(move |_| {
                let Some(w) = w.upgrade() else {
                    return;
                };
                w.save_logs();
            });
        }

        {
            let w = Rc::downgrade(win);
            win.input
                .line_ending_dropdown
                .connect_selected_notify(move |dropdown| {
                    let Some(w) = w.upgrade() else {
                        return;
                    };

                    let le_str = match dropdown.selected() {
                        1 => "CR",
                        2 => "CRLF",
                        3 => "None",
                        _ => "LF",
                    };
                    w.settings.borrow_mut().set_line_ending(le_str);
                });
        }

        {
            let terminal = win.terminal.text_view.clone();
            let w = Rc::downgrade(win);
            win.input
                .stop_scroll_checkbox
                .connect_toggled(move |checkbox| {
                    let Some(w) = w.upgrade() else {
                        return;
                    };

                    let auto_scroll = !checkbox.is_active();
                    w.terminal.set_auto_scroll_enabled(auto_scroll);
                    if auto_scroll {
                        w.terminal
                            .buffer
                            .move_mark(&w.terminal.scroll_mark, &w.terminal.buffer.end_iter());
                        terminal.scroll_to_mark(&w.terminal.scroll_mark, 0.0, false, 0.0, 1.0);
                    }
                });
        }

        {
            let w = Rc::downgrade(win);
            win.window.connect_close_request(move |window| {
                let Some(w) = w.upgrade() else {
                    return gtk4::glib::Propagation::Proceed;
                };

                let (width, height) = (window.width(), window.height());
                w.sync_ui_state_to_settings();
                w.settings.borrow_mut().set_window_size(width, height);
                let _ = w.settings.borrow().save();

                if let Some(tx) = w.connection_tx.borrow_mut().take() {
                    let _ = tx.try_send(ConnectionCommand::Disconnect);
                }

                log::info!("Application fermée proprement.");
                glib::Propagation::Proceed
            });
        }

        {
            let w = Rc::downgrade(win);
            glib::timeout_add_local(Duration::from_millis(150), move || {
                let Some(window) = w.upgrade() else {
                    return glib::ControlFlow::Break;
                };

                window.sync_terminal_size();
                glib::ControlFlow::Continue
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::core::settings::{AppSettings, SettingsManager};

    static TEST_APP_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn close_window_for_tests(window: &Rc<MainWindow>) {
        if window.window.is_visible() {
            window.window.close();
        }
        while glib::MainContext::default().iteration(false) {}
    }

    fn isolated_test_config_path(id: usize) -> PathBuf {
        std::env::temp_dir().join(format!("serial-term-tests/signals-{id}-settings.json"))
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
        let app = crate::ui::window::shell::test_application();

        let window = MainWindow::new_for_test(&app)
            .expect("la fenêtre principale doit pouvoir être construite");
        SettingsManager::clear_test_config_path();
        window
    }

    #[gtk4::test]
    fn close_request_disconnects_active_connection_cleanly() {
        crate::ui::runtime::sanitize_problematic_desktop_theme();
        let window = build_window_for_tests();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        *window.connection_tx.borrow_mut() = Some(tx);

        let _ = window.window.emit_by_name::<bool>("close-request", &[]);
        while glib::MainContext::default().iteration(false) {}

        assert!(window.connection_tx.borrow().is_none());
        assert!(matches!(rx.try_recv(), Ok(ConnectionCommand::Disconnect)));

        close_window_for_tests(&window);
    }
}
