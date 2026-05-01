use gtk4::prelude::*;
use gtk4::{gio, FileDialog};
use libadwaita::prelude::*;

use crate::application::use_cases::{
    apply_serial_settings, build_terminal_payload, create_serial_config, SerialConfigError,
    SerialConnectionInput,
};
use crate::core::connection::{Connection, ConnectionCommand};
use crate::core::serial_manager::SerialManager;
use crate::ui::i18n::UiLang;
use crate::ui::terminal_panel::{LogExportMode, TerminalPanel};

use super::lifecycle::queue_toast;
use super::shell::{set_bottom_notice, MainWindow};

fn save_logs_with_mode(
    window: &libadwaita::ApplicationWindow,
    toast_overlay: &libadwaita::ToastOverlay,
    terminal: &TerminalPanel,
    notice_label: &gtk4::Label,
    notice_generation: &std::rc::Rc<std::cell::Cell<u64>>,
    lang: UiLang,
    mode: LogExportMode,
) {
    let export_text = terminal.export_text(mode);
    let dialog = FileDialog::builder()
        .title(lang.save_logs_dialog_title(mode))
        .initial_name(format!(
            "serial_log_{}.txt",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        ))
        .build();

    let toast_overlay = toast_overlay.clone();
    let parent_window = window.clone();
    let window = window.clone();
    let notice_label = notice_label.clone();
    let notice_generation = notice_generation.clone();
    let lang_saved = lang.logs_saved_toast(mode);
    let lang_saved_term = lang.logs_saved_term_prefix(mode);

    dialog.save(
        Some(&parent_window),
        gio::Cancellable::NONE,
        move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    match std::fs::write(&path, &export_text) {
                        Ok(()) => {
                            log::info!("Logs sauvegardés dans {}", path.display());
                            queue_toast(
                                &window,
                                &toast_overlay,
                                &format!("{} {}", lang_saved, path.display()),
                                4,
                            );
                            set_bottom_notice(
                                &notice_label,
                                &notice_generation,
                                &format!("{} {}", lang_saved_term, path.display()),
                                Some(4),
                            );
                        }
                        Err(e) => {
                            log::error!("Erreur de sauvegarde : {e}");
                        }
                    }
                }
            }
        },
    );
}

impl MainWindow {
    fn localize_serial_config_error(&self, error: SerialConfigError) -> String {
        match error {
            SerialConfigError::NoPortSelected => self.lang.no_port_selected().to_string(),
        }
    }

    /// Construit le manager série + persiste les réglages UI.
    pub(super) fn build_serial_manager(&self) -> Result<Box<dyn Connection>, String> {
        let sp = &self.connection_panel.serial_panel;
        let input = SerialConnectionInput {
            port: sp
                .effective_port()
                .ok_or_else(|| self.lang.no_port_selected().to_string())?,
            baudrate: sp.selected_baudrate(),
            data_bits: sp.selected_data_bits(),
            parity: sp.selected_parity(),
            stop_bits: sp.selected_stop_bits(),
            flow_control: sp.selected_flow_control(),
            timeout_ms: sp.selected_timeout_ms(),
        };

        let config =
            create_serial_config(&input).map_err(|e| self.localize_serial_config_error(e))?;

        {
            let mut sm = self.settings.borrow_mut();
            let serial = &mut sm.settings_mut().serial;
            apply_serial_settings(serial, &input);
            serial.auto_select_single_port = sp.auto_select_single_port_enabled();
            serial.auto_reconnect = sp.auto_reconnect_enabled();
            serial.reconnect_delay_ms = sp.selected_reconnect_delay_ms();
            if let Err(e) = sm.save() {
                log::warn!("Impossible de sauvegarder les paramètres série : {e}");
            }
        }

        Ok(Box::new(SerialManager::new(config)))
    }

    /// Envoie la ligne saisie vers la connexion active.
    pub(super) fn send_data(&self) {
        let text = self.input.get_text();
        let line_ending = self.input.selected_line_ending();
        let Some(data) = build_terminal_payload(&text, line_ending) else {
            return;
        };

        let data_len = u64::try_from(data.len()).unwrap_or(0);

        if let Some(tx) = self.connection_tx.borrow().as_ref() {
            if let Err(e) = tx.try_send(ConnectionCommand::SendData(data)) {
                self.terminal
                    .append_error(&format!("{} {e}", self.lang.send_error()));
            } else {
                self.tx_bytes
                    .set(self.tx_bytes.get().saturating_add(data_len));
                self.update_stats();
                self.terminal.append_sent(&format!("→ {text}\n"));
                self.input.clear();
                self.input.grab_focus();
            }
        } else {
            self.terminal.append_error(self.lang.not_connected_error());
        }
    }

    /// Sauvegarde le terminal dans un fichier texte.
    pub(super) fn save_logs(&self) {
        let text = self.terminal.get_text();
        if text.is_empty() {
            self.show_bottom_notice(self.lang.nothing_to_save());
            return;
        }

        let prefer_timestamped = self.settings.borrow().settings().log.timestamp_saved_lines;
        let default_response = if prefer_timestamped {
            "timestamped"
        } else {
            "raw"
        };

        let format_dialog = libadwaita::AlertDialog::new(
            Some(self.lang.save_logs_mode_heading()),
            Some(self.lang.save_logs_mode_body()),
        );
        format_dialog.add_response("cancel", self.lang.save_logs_mode_cancel_label());
        format_dialog.add_response("raw", self.lang.save_logs_mode_raw_label());
        format_dialog.add_response("timestamped", self.lang.save_logs_mode_timestamped_label());
        format_dialog.add_response("split", self.lang.save_logs_mode_split_label());
        format_dialog.set_close_response("cancel");
        format_dialog.set_default_response(Some(default_response));
        format_dialog
            .set_response_appearance(default_response, libadwaita::ResponseAppearance::Suggested);

        let window = self.window.clone();
        let toast_overlay = self.toast_overlay.clone();
        let terminal = self.terminal.clone();
        let notice_label = self.bottom_notice_label.clone();
        let notice_generation = self.bottom_notice_generation.clone();
        let lang = self.lang;
        let settings = self.settings.clone();
        format_dialog.connect_response(None, move |_, response| {
            let mode = match response {
                "raw" => LogExportMode::Raw,
                "timestamped" => LogExportMode::Timestamped,
                "split" => LogExportMode::Split,
                _ => return,
            };

            {
                let mut sm = settings.borrow_mut();
                sm.settings_mut().log.timestamp_saved_lines = !matches!(mode, LogExportMode::Raw);
                if let Err(e) = sm.save() {
                    log::warn!("Impossible de sauvegarder la préférence d'horodatage : {e}");
                }
            }

            save_logs_with_mode(
                &window,
                &toast_overlay,
                &terminal,
                &notice_label,
                &notice_generation,
                lang,
                mode,
            );
        });

        format_dialog.present(Some(&self.window));
    }
}
