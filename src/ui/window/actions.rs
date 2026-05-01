use std::process::Command;
use std::rc::Rc;

use gtk4::gio;
use libadwaita::prelude::*;

use crate::ui::theme::{Theme, ThemeManager};
use crate::ui::tools_dialog::open_tools_dialog;

use super::dialogs::{open_shortcuts_dialog, show_contact_dialog};
use super::lifecycle::queue_toast;
use super::shell::MainWindow;

impl MainWindow {
    /// Enregistre toutes les actions GIO de la fenêtre.
    ///
    /// Cette méthode centralise les commandes menu/raccourcis:
    /// thème, outils, logs, recherche, navigation et fenêtre.
    #[allow(clippy::clone_on_ref_ptr, clippy::expect_used, clippy::too_many_lines)]
    pub(super) fn setup_actions(win: &Rc<Self>) {
        let saved_theme_id = Theme::normalized_id(&win.settings.borrow().settings().ui.theme);
        let theme_action = gio::SimpleAction::new_stateful(
            "set-theme",
            Some(&String::static_variant_type()),
            &saved_theme_id.to_variant(),
        );
        {
            let w = win.clone();
            theme_action.connect_activate(move |action, param| {
                if let Some(theme_name) = param.and_then(gtk4::glib::Variant::get::<String>) {
                    let theme = Theme::from_str_name(&theme_name);
                    let normalized_theme_id = theme.id();
                    ThemeManager::apply(theme);
                    action.set_state(&normalized_theme_id.to_variant());
                    w.settings.borrow_mut().set_theme(normalized_theme_id);
                    w.show_bottom_notice(&format!(
                        "{} {}",
                        w.lang.theme_changed_prefix(),
                        theme.display_name_localized(w.lang)
                    ));
                }
            });
        }
        win.window.add_action(&theme_action);

        let lang_action = gio::SimpleAction::new_stateful(
            "set-language",
            Some(&String::static_variant_type()),
            &win.settings
                .borrow()
                .settings()
                .ui
                .language
                .clone()
                .to_variant(),
        );
        {
            let w = win.clone();
            lang_action.connect_activate(move |action, param| {
                if let Some(lang_id) = param.and_then(gtk4::glib::Variant::get::<String>) {
                    action.set_state(&lang_id.to_variant());
                    w.settings.borrow_mut().set_language(&lang_id);
                    let msg = w.lang.language_saved_restart();
                    queue_toast(&w.window, &w.toast_overlay, msg, 4);
                }
            });
        }
        win.window.add_action(&lang_action);

        let save_action = gio::SimpleAction::new("save-logs", None);
        {
            let w = win.clone();
            save_action.connect_activate(move |_, _| {
                w.save_logs();
            });
        }
        win.window.add_action(&save_action);

        let tools_action = gio::SimpleAction::new("open-tools", None);
        {
            let w = win.clone();
            tools_action.connect_activate(move |_, _| {
                open_tools_dialog(&w.window, w.lang);
            });
        }
        win.window.add_action(&tools_action);

        let view_logs_action = gio::SimpleAction::new("view-logs", None);
        {
            let w = win.clone();
            #[allow(clippy::clone_on_ref_ptr)]
            view_logs_action.connect_activate(move |_, _| {
                let wc = w.clone();
                let get_logs = std::rc::Rc::new(move || wc.terminal.get_text());
                crate::ui::log_viewer::open_log_viewer(&w.window, w.lang, get_logs);
            });
        }
        win.window.add_action(&view_logs_action);

        let clear_action = gio::SimpleAction::new("clear-terminal", None);
        {
            let w = win.clone();
            clear_action.connect_activate(move |_, _| {
                w.terminal.clear();
                w.show_bottom_notice(w.lang.terminal_cleared());
            });
        }
        win.window.add_action(&clear_action);

        let new_terminal_action = gio::SimpleAction::new("new-terminal", None);
        {
            let w = win.clone();
            #[allow(clippy::clone_on_ref_ptr)]
            new_terminal_action.connect_activate(move |_, _| match std::env::current_exe() {
                Ok(exe_path) => {
                    if let Err(e) = Command::new(exe_path).spawn() {
                        w.terminal
                            .append_error(&format!("{} {e}", w.lang.new_terminal_spawn_error()));
                    }
                }
                Err(e) => {
                    w.terminal
                        .append_error(&format!("{} {e}", w.lang.new_terminal_exe_error()));
                }
            });
        }
        win.window.add_action(&new_terminal_action);

        let shortcuts_action = gio::SimpleAction::new("show-shortcuts", None);
        {
            let w = win.clone();
            #[allow(clippy::clone_on_ref_ptr)]
            shortcuts_action.connect_activate(move |_, _| {
                open_shortcuts_dialog(&w.window, w.lang);
            });
        }
        win.window.add_action(&shortcuts_action);

        let search_action = gio::SimpleAction::new("search-terminal", None);
        {
            let w = win.clone();
            search_action.connect_activate(move |_, _| {
                w.terminal.toggle_search();
            });
        }
        win.window.add_action(&search_action);

        let copy_terminal_action = gio::SimpleAction::new("copy-terminal-selection", None);
        {
            let w = win.clone();
            copy_terminal_action.connect_activate(move |_, _| {
                if let Some((start, end)) = w.terminal.buffer.selection_bounds() {
                    let text = w.terminal.buffer.text(&start, &end, false);
                    if !text.is_empty() {
                        w.window.clipboard().set_text(text.as_str());
                    }
                }
            });
        }
        win.window.add_action(&copy_terminal_action);

        let paste_input_action = gio::SimpleAction::new("paste-input", None);
        {
            let w = win.clone();
            paste_input_action.connect_activate(move |_, _| {
                let clipboard = w.window.clipboard();
                let entry = w.input.entry.clone();

                gtk4::glib::spawn_future_local(async move {
                    if let Ok(Some(text)) = clipboard.read_text_future().await {
                        let mut current = entry.text().to_string();
                        current.push_str(text.as_str());
                        entry.set_text(&current);
                        entry.grab_focus();
                        entry.set_position(-1);
                    }
                });
            });
        }
        win.window.add_action(&paste_input_action);

        let find_next_action = gio::SimpleAction::new("find-next", None);
        {
            let w = win.clone();
            find_next_action.connect_activate(move |_, _| {
                w.terminal.find_next();
            });
        }
        win.window.add_action(&find_next_action);

        let find_prev_action = gio::SimpleAction::new("find-prev", None);
        {
            let w = win.clone();
            find_prev_action.connect_activate(move |_, _| {
                w.terminal.find_prev();
            });
        }
        win.window.add_action(&find_prev_action);

        let about_action = gio::SimpleAction::new("about", None);
        {
            let w = win.clone();
            about_action.connect_activate(move |_, _| {
                let about = libadwaita::AboutDialog::builder()
                    .application_name("SerialTerm")
                    .version(env!("CARGO_PKG_VERSION"))
                    .developer_name("M@nu")
                    .support_url("mailto:tutoelectroweb@gmail.com")
                    .issue_url("https://github.com/TutoElectroWeb/SerialTerm/issues")
                    .comments(w.lang.about_comments())
                    .license_type(gtk4::License::Gpl30)
                    .website("https://github.com/TutoElectroWeb/SerialTerm")
                    .application_icon("io.github.TutoElectroWeb.SerialTerm")
                    .build();
                about.present(Some(&w.window.clone().upcast::<gtk4::Widget>()));
            });
        }
        win.window.add_action(&about_action);

        let contact_action = gio::SimpleAction::new("show-contact", None);
        {
            let w = win.clone();
            contact_action.connect_activate(move |_, _| {
                show_contact_dialog(&w.window, w.lang);
            });
        }
        win.window.add_action(&contact_action);

        let report_issue_action = gio::SimpleAction::new("report-issue", None);
        {
            let w = win.clone();
            report_issue_action.connect_activate(move |_, _| {
                let url = "https://github.com/TutoElectroWeb/SerialTerm/issues/new/choose";
                if let Err(e) =
                    gio::AppInfo::launch_default_for_uri(url, None::<&gio::AppLaunchContext>)
                {
                    w.terminal
                        .append_error(&format!("{} {e}", w.lang.browser_open_error()));
                }
            });
        }
        win.window.add_action(&report_issue_action);

        let close_action = gio::SimpleAction::new("close", None);
        {
            let w = win.clone();
            close_action.connect_activate(move |_, _| {
                w.window.close();
            });
        }
        win.window.add_action(&close_action);

        // La fenêtre est toujours rattachée à son `Application` au moment où ce setup
        // est invoqué (cf. `MainWindow::new`). On évite tout de même `.expect()` pour
        // respecter la politique zéro-panic de la couche UI : sans application, les
        // raccourcis sont simplement ignorés (les actions `win.*` restent fonctionnelles
        // depuis le menu).
        let Some(app) = win.window.application() else {
            log::warn!(
                "MainWindow::setup_actions: aucune application GTK rattachée — \
                 raccourcis clavier non enregistrés"
            );
            return;
        };
        app.set_accels_for_action("win.new-terminal", &["<Ctrl>n"]);
        app.set_accels_for_action("win.save-logs", &["<Ctrl>s"]);
        app.set_accels_for_action("win.close", &["<Ctrl>q"]);
        app.set_accels_for_action("win.clear-terminal", &["<Ctrl>l"]);
        app.set_accels_for_action("win.open-tools", &["<Ctrl>t"]);
        app.set_accels_for_action("win.view-logs", &["<Ctrl><Shift>l"]);
        app.set_accels_for_action("win.show-shortcuts", &["F1"]);
        app.set_accels_for_action("win.search-terminal", &["<Ctrl>f"]);
        app.set_accels_for_action("win.copy-terminal-selection", &["<Ctrl><Shift>c"]);
        app.set_accels_for_action("win.paste-input", &["<Ctrl><Shift>v"]);
        app.set_accels_for_action("win.find-next", &["F3"]);
        app.set_accels_for_action("win.find-prev", &["<Shift>F3"]);
    }
}
