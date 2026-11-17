use std::fmt::Write;
use std::rc::Rc;
use std::time::Duration;

use gtk4::glib;
use gtk4::prelude::*;

use super::shell::{set_bottom_notice, MainWindow};
use crate::core::connection::{
    spawn_connection_actor, ConnectionCommand, ConnectionEvent, ConnectionType,
};

#[derive(Clone, Copy, Debug)]
struct SoakModeConfig {
    interval_ms: u64,
    lines_per_tick: u32,
    progress_width: usize,
    diagnostics_secs: u64,
    duration_secs: Option<u64>,
}

impl SoakModeConfig {
    fn from_env() -> Option<Self> {
        if !soak_env_flag("SERIAL_TERM_SOAK_GENERATOR") {
            return None;
        }

        Some(Self {
            interval_ms: soak_env_u64("SERIAL_TERM_SOAK_INTERVAL_MS", 16, 1, 5_000),
            lines_per_tick: u32::try_from(soak_env_u64(
                "SERIAL_TERM_SOAK_LINES_PER_TICK",
                6,
                1,
                128,
            ))
            .unwrap_or(6),
            progress_width: usize::try_from(soak_env_u64(
                "SERIAL_TERM_SOAK_PROGRESS_WIDTH",
                40,
                8,
                200,
            ))
            .unwrap_or(40),
            diagnostics_secs: soak_env_u64("SERIAL_TERM_SOAK_DIAGNOSTICS_SECS", 60, 1, 3_600),
            duration_secs: match soak_env_u64("SERIAL_TERM_SOAK_DURATION_SECS", 0, 0, 86_400) {
                0 => None,
                value => Some(value),
            },
        })
    }
}

fn soak_env_flag(name: &str) -> bool {
    crate::app::soak_env_flag(name)
}

fn soak_env_u64(name: &str, default: u64, min: u64, max: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .map_or(default, |value| value.clamp(min, max))
}

fn soak_progress_bar(width: usize, tick: u64, line_index: u32) -> String {
    let width = width.max(1);
    let width_u64 = u64::try_from(width).unwrap_or(1);
    let offset = tick.wrapping_add(u64::from(line_index));
    let position_u64 = offset.checked_rem(width_u64).unwrap_or(0);
    let position = usize::try_from(position_u64).unwrap_or(0);
    let mut bar = String::with_capacity(width);

    for idx in 0..width {
        if idx == position {
            bar.push('#');
        } else {
            bar.push('-');
        }
    }

    bar
}

fn soak_payload(tick: u64, config: SoakModeConfig) -> Vec<u8> {
    let mut payload = String::new();

    for line_index in 0..config.lines_per_tick {
        let phase = tick.wrapping_add(u64::from(line_index)) % 6;
        let color = match phase {
            0 => 31,
            1 => 32,
            2 => 33,
            3 => 34,
            4 => 35,
            _ => 36,
        };
        let bar = soak_progress_bar(config.progress_width, tick, line_index);
        let seq = tick
            .saturating_mul(u64::from(config.lines_per_tick))
            .saturating_add(u64::from(line_index));

        let _ = writeln!(
            payload,
            "\x1b[1;{color}m[SOAK {seq:06}]\x1b[0m line={line_index:02} bar=[{bar}] ansi=\x1b[38;5;45mOK\x1b[0m rgb=\x1b[38;2;255;180;0mACTIVE\x1b[0m"
        );
    }

    payload.into_bytes()
}

pub(super) fn queue_toast(
    window: &libadwaita::ApplicationWindow,
    overlay: &libadwaita::ToastOverlay,
    message: &str,
    timeout: u32,
) {
    let window = window.downgrade();
    let overlay = overlay.downgrade();
    let message = message.to_string();

    glib::idle_add_local_once(move || {
        let (Some(window), Some(overlay)) = (window.upgrade(), overlay.upgrade()) else {
            return;
        };
        if !window.is_visible() || !window.is_mapped() {
            log::debug!("Toast ignoré car la fenêtre n'est pas encore allouée: {message}");
            return;
        }

        let toast = libadwaita::Toast::new(&message);
        toast.set_timeout(timeout);
        overlay.add_toast(toast);
    });
}

impl MainWindow {
    /// L'énumération peut attendre le système : toujours hors du thread GTK.
    #[allow(clippy::future_not_send)] // Future locale GTK, exécutée par GLib.
    async fn scan_serial_ports(&self) -> Option<Vec<crate::core::serial_manager::SerialPortInfo>> {
        match self
            .runtime
            .spawn_blocking(crate::core::serial_manager::list_serial_ports)
            .await
        {
            Ok(ports) => Some(ports),
            Err(error) => {
                log::warn!("Énumération des ports interrompue : {error}");
                None
            }
        }
    }

    #[allow(clippy::future_not_send)] // Future locale GTK, exécutée par GLib.
    pub(super) async fn refresh_serial_ports(&self) {
        let Some(ports) = self.scan_serial_ports().await else {
            return;
        };
        let panel = &self.connection_panel.serial_panel;
        let settings = self.settings.borrow();
        let saved_port = &settings.settings().serial.port;
        if panel.effective_port().is_none() && !saved_port.is_empty() {
            panel.update_ports_for_reconnect(&ports, saved_port);
        } else {
            panel.update_ports_from_list(&ports);
        }
    }

    fn handle_connection_event(
        self: &Rc<Self>,
        event: Result<ConnectionEvent, async_channel::RecvError>,
        automatic: bool,
    ) -> bool {
        match event {
            Ok(ConnectionEvent::Connected {
                conn_type,
                description,
            }) => {
                let type_label = match conn_type {
                    ConnectionType::Serial => self.lang.serial_label(),
                };
                log::debug!("Connexion établie : {description}");
                if automatic {
                    self.terminal.reset_ansi_state();
                    self.terminal.append_system(self.lang.connected_system());
                } else {
                    self.terminal.clear();
                }
                self.rx_bytes.set(0);
                self.tx_bytes.set(0);
                self.update_stats();
                let baudrate = self.connection_panel.serial_panel.selected_baudrate();
                let baud_unit = self.lang.serial_baud_unit();
                let connected_status = self
                    .connection_panel
                    .serial_panel
                    .selected_port_friendly_name()
                    .map_or_else(
                        || {
                            format!(
                                "{} {type_label} : {baudrate} {baud_unit}",
                                self.lang.connected_system()
                            )
                        },
                        |device| {
                            format!(
                                "{} {type_label} : {device}, {baudrate} {baud_unit}",
                                self.lang.connected_system()
                            )
                        },
                    );
                self.set_connected(true);
                self.set_status(&connected_status, true);
                self.show_bottom_notice(&connected_status);
                self.input.grab_focus();
                self.sync_terminal_size();
                true
            }
            Ok(ConnectionEvent::DataSent(data)) => {
                self.tx_bytes.set(
                    self.tx_bytes
                        .get()
                        .saturating_add(u64::try_from(data.len()).unwrap_or(0)),
                );
                self.update_stats();
                let payload = String::from_utf8_lossy(&data);
                self.terminal.append_sent(&format!(
                    "TX : {}\n",
                    payload.trim_end_matches(['\r', '\n'])
                ));
                true
            }
            Ok(ConnectionEvent::DataReceived(data)) => {
                let n = u64::try_from(data.len()).unwrap_or(0);
                self.rx_bytes.set(self.rx_bytes.get().saturating_add(n));
                self.update_stats();
                self.terminal.append_ansi(&data);
                true
            }
            Ok(ConnectionEvent::Error(e)) => {
                self.terminal.append_error(&e);
                self.handle_disconnect();
                false
            }
            Ok(ConnectionEvent::Disconnected) | Err(_) => {
                self.handle_disconnect();
                false
            }
        }
    }

    #[allow(clippy::clone_on_ref_ptr)]
    pub(crate) fn start_soak_mode_from_env(self: &Rc<Self>) {
        let Some(config) = SoakModeConfig::from_env() else {
            return;
        };

        self.show_bottom_notice(&format!(
            "Mode soak actif: interval={} ms, lignes/tick={}, largeur={}, diagnostics={} s{}",
            config.interval_ms,
            config.lines_per_tick,
            config.progress_width,
            config.diagnostics_secs,
            config
                .duration_secs
                .map(|secs| format!(", durée={secs} s"))
                .unwrap_or_default()
        ));
        log::info!(
            "Soak mode actif: interval={} ms, lines_per_tick={}, progress_width={}, diagnostics_secs={}, duration_secs={:?}",
            config.interval_ms,
            config.lines_per_tick,
            config.progress_width,
            config.diagnostics_secs,
            config.duration_secs
        );

        let tick = Rc::new(std::cell::Cell::new(0_u64));

        {
            let w = Rc::downgrade(self);
            glib::timeout_add_local(Duration::from_millis(config.interval_ms), move || {
                let Some(window) = w.upgrade() else {
                    return glib::ControlFlow::Break;
                };

                let next_tick = tick.get().saturating_add(1);
                tick.set(next_tick);
                let payload = soak_payload(next_tick, config);
                window.terminal.append_ansi(&payload);

                glib::ControlFlow::Continue
            });
        }

        {
            let w = Rc::downgrade(self);
            glib::timeout_add_local(Duration::from_secs(config.diagnostics_secs), move || {
                let Some(window) = w.upgrade() else {
                    return glib::ControlFlow::Break;
                };

                log::info!(
                    "Soak diagnostics: lines={}, chars={}, rx={}, tx={}, connected={}",
                    window.terminal.buffer.line_count(),
                    window.terminal.buffer.char_count(),
                    window.rx_bytes.get(),
                    window.tx_bytes.get(),
                    window.connection_tx.borrow().is_some()
                );

                glib::ControlFlow::Continue
            });
        }

        if let Some(duration_secs) = config.duration_secs {
            let w = Rc::downgrade(self);
            glib::timeout_add_local(Duration::from_secs(duration_secs), move || {
                if let Some(window) = w.upgrade() {
                    window.show_bottom_notice("Durée soak atteinte, fermeture automatique.");
                    log::info!("Soak duration atteinte, fermeture automatique de l'application.");
                    window.window.close();
                }

                glib::ControlFlow::Break
            });
        }
    }

    /// Bascule connexion / déconnexion.
    #[allow(clippy::clone_on_ref_ptr)]
    pub(super) fn toggle_connection(self: &Rc<Self>) {
        if self.connection_tx.borrow().is_some() {
            self.disconnect();
        } else {
            self.connect();
        }
    }

    /// Établit la connexion série et démarre la pompe d'événements async.
    pub(super) fn connect(self: &Rc<Self>) {
        self.reconnect_generation
            .set(self.reconnect_generation.get().wrapping_add(1));
        self.start_connection(false);
    }

    #[allow(clippy::clone_on_ref_ptr)]
    fn start_connection(self: &Rc<Self>, automatic: bool) {
        let (manager, port) = match self.build_serial_manager() {
            Ok(connection) => connection,
            Err(e) => {
                self.set_status(self.lang.configuration_error(), false);
                self.terminal.append_error(&e);
                self.show_toast(&e);
                log::error!("Erreur de configuration : {e}");
                return;
            }
        };

        self.set_status(self.lang.status_connecting(), false);
        self.show_bottom_notice(self.lang.status_connecting());

        let guard = self.runtime.enter();
        let (cmd_tx, event_rx) = spawn_connection_actor(manager);
        drop(guard);

        *self.connection_tx.borrow_mut() = Some(cmd_tx);

        let my_generation = self.connection_generation.get().wrapping_add(1);
        self.connection_generation.set(my_generation);

        let this = self.clone();
        glib::spawn_future_local(async move {
            loop {
                let event = event_rx.recv().await;
                if this.connection_generation.get() != my_generation {
                    break;
                }
                if !this.handle_connection_event(event, automatic) {
                    this.schedule_auto_reconnect(port);
                    break;
                }
            }
        });
    }

    /// Déconnexion idempotente + synchronisation état UI.
    pub(super) fn handle_disconnect(&self) {
        let had_connection = self.connection_tx.borrow().is_some();
        if let Some(tx) = self.connection_tx.borrow_mut().take() {
            if tx.try_send(ConnectionCommand::Disconnect).is_err() {
                log::debug!("Acteur déjà fermé lors de handle_disconnect");
            }
        }

        if had_connection {
            self.set_connected(false);
            self.set_status(self.lang.status_disconnected(), false);
            self.show_bottom_notice(self.lang.status_disconnected());
            self.show_toast(self.lang.connection_closed_toast());
            self.rx_bytes.set(0);
            self.tx_bytes.set(0);
            self.last_terminal_grid.set((0, 0, 0, 0));
            self.io_bytes_label.set_label("");
        }
    }

    /// Synchronise la taille visible du terminal avec le PTY distant.
    pub(super) fn sync_terminal_size(&self) {
        let Some((columns, rows, pixel_width, pixel_height)) = self.terminal.visible_grid_size()
        else {
            return;
        };

        let next_grid = (columns, rows, pixel_width, pixel_height);
        if self.last_terminal_grid.get() == next_grid {
            return;
        }

        let Some(tx) = self.connection_tx.borrow().as_ref().cloned() else {
            return;
        };

        match tx.try_send(ConnectionCommand::ResizeTerminal {
            columns,
            rows,
            pixel_width,
            pixel_height,
        }) {
            Ok(()) => self.last_terminal_grid.set(next_grid),
            Err(e) => {
                log::debug!("Redimensionnement PTY ignoré : {e}");
            }
        }
    }

    /// Met à jour l'apparence du bouton Connecter/Déconnecter.
    pub fn set_connected(&self, connected: bool) {
        if connected {
            self.connect_button
                .set_tooltip_text(Some(self.lang.disconnect_tooltip()));
            self.connect_button
                .set_icon_name("network-offline-symbolic");
            self.connect_button.remove_css_class("suggested-action");
            self.connect_button.add_css_class("destructive-action");
        } else {
            self.connect_button
                .set_tooltip_text(Some(self.lang.connect_tooltip()));
            self.connect_button.set_icon_name("network-wired-symbolic");
            self.connect_button.remove_css_class("destructive-action");
            self.connect_button.add_css_class("suggested-action");
        }
    }

    /// Affiche un toast Adwaita non-bloquant.
    pub fn show_toast(&self, message: &str) {
        queue_toast(&self.window, &self.toast_overlay, message, 3);
    }

    pub(super) fn show_bottom_notice(&self, message: &str) {
        set_bottom_notice(
            &self.bottom_notice_label,
            &self.bottom_notice_generation,
            message,
            Some(4),
        );
    }

    /// Met à jour le texte et la couleur de l'état en barre basse.
    pub fn set_status(&self, text: &str, connected: bool) {
        self.status_label.set_label(text);
        if connected {
            self.status_label.remove_css_class("status-disconnected");
            self.status_label.add_css_class("status-connected");
            self.status_dot.remove_css_class("status-dot-disconnected");
            self.status_dot.add_css_class("status-dot-connected");
        } else {
            self.status_label.remove_css_class("status-connected");
            self.status_label.add_css_class("status-disconnected");
            self.status_dot.remove_css_class("status-dot-connected");
            self.status_dot.add_css_class("status-dot-disconnected");
        }
    }

    /// Rafraîchit les stats RX/TX avec format lisible.
    pub(super) fn update_stats(&self) {
        let rx = self.rx_bytes.get();
        let tx = self.tx_bytes.get();
        self.io_bytes_label.set_label(&format!(
            "RX : {} ; TX : {}",
            format_bytes(rx),
            format_bytes(tx)
        ));
    }

    /// Déconnexion demandée explicitement par l'utilisateur.
    pub(super) fn disconnect(&self) {
        // Invalide les boucles d'événements et les timers de reconnexion en attente.
        self.connection_generation
            .set(self.connection_generation.get().wrapping_add(1));
        self.reconnect_generation
            .set(self.reconnect_generation.get().wrapping_add(1));
        self.handle_disconnect();
    }

    /// Planifie une reconnexion automatique série si l'option est activée.
    #[allow(clippy::clone_on_ref_ptr)]
    fn schedule_auto_reconnect(self: &Rc<Self>, port: String) {
        let auto_reconnect = self.connection_panel.serial_panel.auto_reconnect_enabled();
        let delay_ms = self
            .connection_panel
            .serial_panel
            .selected_reconnect_delay_ms();

        if !auto_reconnect || port.is_empty() {
            return;
        }
        #[cfg(target_os = "linux")]
        if crate::core::serial_manager::is_linux_hotplug_device_name(&port) {
            self.show_bottom_notice(self.lang.auto_reconnect_unstable_port_notice());
            return;
        }

        let gen = self.reconnect_generation.get().wrapping_add(1);
        self.reconnect_generation.set(gen);

        let notice = if delay_ms >= 1_000 {
            format!(
                "{} {} s...",
                self.lang.auto_reconnect_scheduled_notice(),
                delay_ms / 1_000
            )
        } else {
            format!(
                "{} {} ms...",
                self.lang.auto_reconnect_scheduled_notice(),
                delay_ms
            )
        };
        self.show_bottom_notice(&notice);
        log::info!("Reconnexion automatique planifiée dans {delay_ms} ms (génération {gen}).");

        let w = Rc::downgrade(self);
        glib::timeout_add_local_once(Duration::from_millis(delay_ms), move || {
            let Some(w) = w.upgrade() else {
                return;
            };
            if w.reconnect_generation.get() != gen {
                log::debug!("Timer de connexion automatique annulé (génération obsolète).");
                return;
            }
            if w.connection_tx.borrow().is_some() {
                log::debug!("Timer de connexion automatique ignoré (déjà connecté).");
                return;
            }

            glib::spawn_future_local(async move {
                let ports = w.scan_serial_ports().await;
                let panel = &w.connection_panel.serial_panel;
                if w.reconnect_generation.get() != gen
                    || w.connection_tx.borrow().is_some()
                    || !panel.auto_reconnect_enabled()
                {
                    return;
                }
                if ports.is_some_and(|ports| panel.update_ports_for_reconnect(&ports, &port)) {
                    w.show_bottom_notice(w.lang.auto_reconnect_attempt_notice());
                    w.start_connection(true);
                } else {
                    w.schedule_auto_reconnect(port);
                    w.show_bottom_notice(w.lang.auto_reconnect_port_unavailable_notice());
                }
            });
        });
    }

    /// Lance la connexion automatique au démarrage si la case est cochée.
    #[allow(clippy::clone_on_ref_ptr)]
    pub(super) fn maybe_start_auto_connect(self: &Rc<Self>) {
        if !self.connection_panel.serial_panel.auto_reconnect_enabled() {
            return;
        }
        let saved_port = self.settings.borrow().settings().serial.port.clone();
        let port = if saved_port.is_empty() {
            self.connection_panel
                .serial_panel
                .effective_port()
                .unwrap_or_default()
        } else {
            saved_port
        };
        self.schedule_auto_reconnect(port);
    }
}

/// Formate les octets en unité lisible (B/KiB/MiB), sans flottants.
pub(super) fn format_bytes(n: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB.saturating_mul(1024);
    if n >= MIB {
        let whole = n / MIB;
        let frac = (n % MIB).saturating_mul(10) / MIB;
        format!("{whole}.{frac} MiB")
    } else if n >= KIB {
        let whole = n / KIB;
        let frac = (n % KIB).saturating_mul(10) / KIB;
        format!("{whole}.{frac} KiB")
    } else {
        format!("{n} B")
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::core::settings::{AppSettings, SettingsManager};

    static TEST_APP_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn close_window_for_tests(window: &Rc<MainWindow>) {
        if window.window.is_visible() {
            window.window.close();
        }
        while gtk4::glib::MainContext::default().iteration(false) {}
    }

    fn isolated_test_config_path(id: usize) -> PathBuf {
        std::env::temp_dir().join(format!("serial-term-tests/lifecycle-{id}-settings.json"))
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
    fn tx_is_only_displayed_after_confirmation_and_full_queue_keeps_input() {
        let window = build_window_for_tests();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        *window.connection_tx.borrow_mut() = Some(tx);
        window.input.entry.set_text("AT");
        window.send_data();
        assert_eq!(window.tx_bytes.get(), 0);
        assert_eq!(window.terminal.get_text(), "");
        window.input.entry.set_text("second");
        window.send_data();
        assert_eq!(window.input.get_text(), "second");
        assert_eq!(window.tx_bytes.get(), 0);
        assert!(matches!(rx.try_recv(), Ok(ConnectionCommand::SendData(_))));
        window.handle_connection_event(Ok(ConnectionEvent::DataSent(b"AT\n".to_vec())), false);
        assert_eq!(window.tx_bytes.get(), 3);
        assert!(window.terminal.get_text().contains("TX : AT\n"));
        window.disconnect();
        close_window_for_tests(&window);
    }

    #[gtk4::test]
    fn previous_export_survives_failed_open_and_automatic_reconnect() {
        use crate::ui::terminal_panel::LogExportMode;

        let window = build_window_for_tests();
        window.terminal.append_ansi(b"ancienne session\n");
        assert!(!window.handle_connection_event(
            Ok(ConnectionEvent::Error("Port indisponible".into())),
            false,
        ));
        assert_eq!(
            window.terminal.export_text(LogExportMode::Raw),
            "ancienne session\n"
        );

        for automatic in [true, false] {
            window.rx_bytes.set(123);
            window.tx_bytes.set(456);
            assert!(window.handle_connection_event(
                Ok(ConnectionEvent::Connected {
                    conn_type: ConnectionType::Serial,
                    description: "test".into(),
                }),
                automatic,
            ));
            let expected = if automatic { "ancienne session\n" } else { "" };
            assert_eq!(window.terminal.export_text(LogExportMode::Raw), expected);
            assert_eq!(window.rx_bytes.get(), 0);
            assert_eq!(window.tx_bytes.get(), 0);
        }
        close_window_for_tests(&window);
    }

    #[gtk4::test]
    fn handle_disconnect_resets_ui_once_after_active_connection() {
        crate::ui::runtime::sanitize_problematic_desktop_theme();
        let window = build_window_for_tests();
        let (tx, _rx) = tokio::sync::mpsc::channel(1);

        *window.connection_tx.borrow_mut() = Some(tx);
        window.rx_bytes.set(1_536);
        window.tx_bytes.set(2_048);
        window.last_terminal_grid.set((120, 40, 960, 720));
        window.update_stats();
        window.set_connected(true);
        window.set_status(window.lang.connected_system(), true);

        window.handle_disconnect();

        assert!(window.connection_tx.borrow().is_none());
        assert_eq!(window.rx_bytes.get(), 0);
        assert_eq!(window.tx_bytes.get(), 0);
        assert_eq!(window.last_terminal_grid.get(), (0, 0, 0, 0));
        assert_eq!(window.io_bytes_label.label(), "");
        assert_eq!(
            window.status_label.label(),
            window.lang.status_disconnected()
        );
        assert!(window.status_label.has_css_class("status-disconnected"));
        assert!(window.status_dot.has_css_class("status-dot-disconnected"));
        assert_eq!(window.terminal.get_text(), "");
        assert_eq!(
            window.bottom_notice_label.label(),
            window.lang.status_disconnected()
        );

        window.handle_disconnect();

        assert_eq!(window.terminal.get_text(), "");

        close_window_for_tests(&window);
    }

    #[gtk4::test]
    fn connected_status_shows_device_name_without_port_path() {
        crate::ui::runtime::sanitize_problematic_desktop_theme();
        let window = build_window_for_tests();
        let ports = vec![crate::core::serial_manager::SerialPortInfo {
            device: "/dev/ttyACM0".to_string(),
            manufacturer: "STMicroelectronics".to_string(),
            description: "STM32 STLink".to_string(),
            friendly_name: "STM32 ST-LINK".to_string(),
            stable_path: "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_123-if02"
                .to_string(),
        }];
        window
            .connection_panel
            .serial_panel
            .update_ports_from_list(&ports);

        let should_continue = window.handle_connection_event(
            Ok(ConnectionEvent::Connected {
                conn_type: ConnectionType::Serial,
                description:
                    "/dev/serial/by-id/usb-STMicroelectronics_STM32_STLink_123-if02 @ 115200"
                        .to_string(),
            }),
            false,
        );

        assert!(should_continue);
        assert!(window.status_label.label().contains("STM32 ST-LINK"));
        assert!(window.status_label.label().contains("115200 bauds"));
        assert!(!window.status_label.label().contains("/dev/"));
        assert!(window.status_dot.has_css_class("status-dot-connected"));

        close_window_for_tests(&window);
    }

    #[gtk4::test]
    fn error_event_is_rendered_then_releases_ui_control() {
        crate::ui::runtime::sanitize_problematic_desktop_theme();
        let window = build_window_for_tests();
        let (tx, _rx) = tokio::sync::mpsc::channel(1);

        *window.connection_tx.borrow_mut() = Some(tx);
        window.set_connected(true);
        window.set_status(window.lang.connected_system(), true);

        let should_continue = window.handle_connection_event(
            Ok(ConnectionEvent::Error(
                "Erreur de communication série: trame invalide détectée".to_string(),
            )),
            false,
        );

        assert!(!should_continue);
        assert!(window.connection_tx.borrow().is_none());
        assert_eq!(
            window.status_label.label(),
            window.lang.status_disconnected()
        );

        let terminal_text = window.terminal.get_text();
        assert!(terminal_text.contains(window.lang.error_prefix()));
        assert!(terminal_text.contains("Erreur de communication série"));
        assert!(terminal_text.contains("trame invalide"));
        assert_eq!(
            window.bottom_notice_label.label(),
            window.lang.status_disconnected()
        );

        close_window_for_tests(&window);
    }

    #[gtk4::test]
    fn serial_hot_unplug_error_is_rendered_then_releases_ui_control() {
        crate::ui::runtime::sanitize_problematic_desktop_theme();
        let window = build_window_for_tests();
        let (tx, _rx) = tokio::sync::mpsc::channel(1);

        *window.connection_tx.borrow_mut() = Some(tx);
        window.set_connected(true);
        window.set_status(window.lang.connected_system(), true);

        let should_continue = window.handle_connection_event(
            Ok(ConnectionEvent::Error(
                "Port série déconnecté pendant la lecture : Input/output error".to_string(),
            )),
            false,
        );

        assert!(!should_continue);
        assert!(window.connection_tx.borrow().is_none());
        assert_eq!(
            window.status_label.label(),
            window.lang.status_disconnected()
        );

        let terminal_text = window.terminal.get_text();
        assert!(terminal_text.contains(window.lang.error_prefix()));
        assert!(terminal_text.contains("Port série déconnecté pendant la lecture"));
        assert!(terminal_text.contains("Input/output error"));
        assert_eq!(
            window.bottom_notice_label.label(),
            window.lang.status_disconnected()
        );

        close_window_for_tests(&window);
    }
}
