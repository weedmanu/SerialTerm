use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use gtk4::{gio, Box as GtkBox, Button, Label, Orientation, Separator};
use libadwaita::prelude::*;
use tokio::runtime::Runtime;

use crate::core::connection::ConnectionCommand;
use crate::core::settings::{SettingsManager, MIN_WINDOW_HEIGHT, MIN_WINDOW_WIDTH};
use crate::ui::connection_panel::ConnectionPanel;
use crate::ui::header_bar::AppHeaderBar;
use crate::ui::i18n::UiLang;
use crate::ui::input_panel::InputPanel;
use crate::ui::terminal_panel::TerminalPanel;
use crate::ui::theme::{Theme, ThemeManager};

fn defer_terminal_welcome_until_mapped(
    window: &libadwaita::ApplicationWindow,
    notice_label: &Label,
    notice_generation: &Rc<Cell<u64>>,
    lang: UiLang,
) {
    let welcome_message = format!("{} {}", lang.welcome_line_1(), lang.welcome_line_2());

    if window.is_visible() && window.is_mapped() {
        set_bottom_notice(notice_label, notice_generation, &welcome_message, Some(6));
        return;
    }

    let notice_label = notice_label.clone();
    let notice_generation = notice_generation.clone();
    let appended = Rc::new(Cell::new(false));

    window.connect_map(move |_| {
        if appended.replace(true) {
            return;
        }

        set_bottom_notice(&notice_label, &notice_generation, &welcome_message, Some(6));
    });
}

pub(super) fn set_bottom_notice(
    notice_label: &Label,
    notice_generation: &Rc<Cell<u64>>,
    message: &str,
    clear_after_secs: Option<u32>,
) {
    let next_generation = notice_generation.get().wrapping_add(1);
    notice_generation.set(next_generation);
    notice_label.set_label(message);

    let Some(delay_secs) = clear_after_secs else {
        return;
    };

    let notice_label = notice_label.clone();
    let notice_generation = notice_generation.clone();
    gtk4::glib::timeout_add_local(
        std::time::Duration::from_secs(u64::from(delay_secs)),
        move || {
            if notice_generation.get() == next_generation {
                notice_label.set_label("");
            }
            gtk4::glib::ControlFlow::Break
        },
    );
}

/// Fenêtre principale de l'application `SerialTerm`.
///
/// Cette structure centralise les widgets et l'état runtime:
/// - Widgets GTK (fenêtre, barre de statut, panneaux),
/// - État de connexion async (canal de commandes vers l'acteur Tokio),
/// - Compteurs TX/RX et configuration persistante.
pub struct MainWindow {
    pub window: libadwaita::ApplicationWindow,
    pub header: AppHeaderBar,
    pub connection_panel: ConnectionPanel,
    pub terminal: TerminalPanel,
    pub input: InputPanel,
    pub status_label: Label,
    pub(super) bottom_notice_label: Label,
    pub connect_button: Button,
    pub clear_button: Button,

    pub(super) settings: Rc<RefCell<SettingsManager>>,
    pub(super) connection_tx: RefCell<Option<tokio::sync::mpsc::Sender<ConnectionCommand>>>,
    pub(super) runtime: Arc<Runtime>,
    pub(super) toast_overlay: libadwaita::ToastOverlay,
    pub(super) status_dot: Label,
    pub(super) bottom_notice_generation: Rc<Cell<u64>>,
    pub(super) connection_generation: Cell<u64>,
    /// Compteur de génération pour annuler les timers de reconnexion automatique
    /// quand l'utilisateur se reconnecte manuellement ou change de port.
    pub(super) reconnect_generation: Cell<u64>,
    pub(super) rx_bytes: Cell<u64>,
    pub(super) tx_bytes: Cell<u64>,
    pub(super) last_terminal_grid: Cell<(u32, u32, u32, u32)>,
    pub(super) io_bytes_label: Label,
    pub(super) lang: UiLang,
    /// Séparateur entre le terminal et la barre d'envoi.
    #[allow(dead_code)]
    pub(super) input_separator: Separator,
}

impl MainWindow {
    fn restore_sidebar_visibility(&self, show_sidebar: bool) {
        if !show_sidebar {
            self.header.toggle_sidebar_button.set_active(false);
            return;
        }

        let toggle_sidebar_button = self.header.toggle_sidebar_button.clone();
        let apply_sidebar_visibility = move || {
            if !toggle_sidebar_button.is_active() {
                toggle_sidebar_button.set_active(true);
            }
        };

        if self.window.is_mapped() {
            gtk4::glib::idle_add_local_once(apply_sidebar_visibility);
            return;
        }

        let toggle_sidebar_button = self.header.toggle_sidebar_button.clone();
        self.window.connect_map(move |_| {
            let toggle_sidebar_button = toggle_sidebar_button.clone();
            gtk4::glib::idle_add_local_once(move || {
                if !toggle_sidebar_button.is_active() {
                    toggle_sidebar_button.set_active(true);
                }
            });
        });
    }

    fn restore_ui_from_settings(&self) {
        let (serial, ui) = {
            let settings = self.settings.borrow();
            (
                settings.settings().serial.clone(),
                settings.settings().ui.clone(),
            )
        };

        self.connection_panel.serial_panel.apply_settings(
            serial.baudrate,
            serial.data_bits,
            &serial.parity,
            serial.stop_bits,
            &serial.flow_control,
            serial.timeout_ms,
            serial.auto_reconnect,
            serial.reconnect_delay_ms,
        );
        self.connection_panel
            .serial_panel
            .set_auto_select_single_port(serial.auto_select_single_port);

        self.connection_panel.serial_panel.refresh_ports();
        self.connection_panel
            .serial_panel
            .select_port_by_device(&serial.port);

        let idx = match ui.line_ending.as_str() {
            "CR" => 1,
            "CRLF" => 2,
            "None" => 3,
            _ => 0,
        };
        self.input.line_ending_dropdown.set_selected(idx);
        self.input.set_stop_scroll_enabled(ui.stop_scroll);
        self.terminal.set_auto_scroll_enabled(!ui.stop_scroll);
        self.restore_sidebar_visibility(ui.show_sidebar);

        if let Some(action) = self.window.lookup_action("set-theme") {
            let theme_id = Theme::normalized_id(&ui.theme);
            action.change_state(&theme_id.to_variant());
        }

        if let Some(action) = self.window.lookup_action("set-language") {
            let lang_id = match ui.language.as_str() {
                "fr" | "en" => ui.language.as_str(),
                _ => "auto",
            };
            action.change_state(&lang_id.to_variant());
        }
    }

    pub(super) fn sync_ui_state_to_settings(&self) {
        let serial_snapshot = self.connection_panel.serial_panel.snapshot_settings();
        let line_ending = match self.input.line_ending_dropdown.selected() {
            1 => "CR",
            2 => "CRLF",
            3 => "None",
            _ => "LF",
        };

        let mut settings = self.settings.borrow_mut();
        let app_settings = settings.settings_mut();
        app_settings.serial = serial_snapshot;
        app_settings.ui.show_sidebar = self.header.toggle_sidebar_button.is_active();
        app_settings.ui.stop_scroll = self.input.stop_scroll_enabled();
        app_settings.ui.line_ending = line_ending.to_string();
    }

    /// Construit et affiche la fenêtre principale.
    ///
    /// Retourne `None` si le runtime Tokio ne peut pas être créé (rare, manque de ressources OS).
    ///
    /// La longueur reflète l'assemblage inévitable de la hiérarchie GTK4 complète :
    /// menus, barres d'outils, split-view, status bar, thème, signaux initiaux.
    /// Chaque section est délimitée par des commentaires ; la décomposition en
    /// sous-fonctions briserait les borrows des widgets partagés entre sections.
    #[allow(clippy::too_many_lines)]
    pub fn new(app: &libadwaita::Application) -> Option<Rc<Self>> {
        let settings = Rc::new(RefCell::new(SettingsManager::new()));
        let s = settings.borrow();
        let lang = UiLang::from_preference(&s.settings().ui.language);
        let window_width = s.settings().ui.window_width.max(MIN_WINDOW_WIDTH);
        let window_height = s.settings().ui.window_height.max(MIN_WINDOW_HEIGHT);

        let runtime = match Runtime::new() {
            Ok(rt) => Arc::new(rt),
            Err(e) => {
                log::error!("Impossible de créer le runtime Tokio : {e}");
                let alert = libadwaita::AlertDialog::builder()
                    .heading(lang.fatal_error_title())
                    .body(format!("{}{e}", lang.fatal_error_body_prefix()))
                    .build();
                alert.add_response("close", lang.quit_label());
                alert.present(gtk4::Widget::NONE);
                app.quit();
                return None;
            }
        };

        let window = libadwaita::ApplicationWindow::builder()
            .application(app)
            .title("SerialTerm")
            .default_width(window_width)
            .default_height(window_height)
            .build();
        window.set_size_request(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT);
        drop(s);

        let header = AppHeaderBar::new(lang);
        let connection_panel = ConnectionPanel::new(lang);
        let terminal =
            TerminalPanel::new(settings.borrow().settings().ui.max_scrollback_lines, lang);
        let input = InputPanel::new(lang);

        let main_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .build();

        let menubar_model = gio::Menu::new();

        let file_menu = gio::Menu::new();
        file_menu.append(Some(lang.new_terminal_label()), Some("win.new-terminal"));
        file_menu.append(Some(lang.save_logs_label()), Some("win.save-logs"));
        file_menu.append(Some(lang.quit_label()), Some("win.close"));
        menubar_model.append_submenu(Some(lang.app_menu_file()), &file_menu);

        let edit_menu = gio::Menu::new();
        edit_menu.append(Some(lang.view_logs_label()), Some("win.view-logs"));
        edit_menu.append(
            Some(lang.search_terminal_label()),
            Some("win.search-terminal"),
        );
        edit_menu.append(
            Some(lang.copy_terminal_selection_label()),
            Some("win.copy-terminal-selection"),
        );
        edit_menu.append(Some(lang.paste_input_label()), Some("win.paste-input"));
        menubar_model.append_submenu(Some(lang.app_menu_edit()), &edit_menu);

        let tools_menu = gio::Menu::new();
        tools_menu.append(
            Some(lang.calculator_converter_label()),
            Some("win.open-tools"),
        );

        let theme_menu = gio::Menu::new();
        for theme in crate::ui::theme::Theme::all() {
            theme_menu.append(
                Some(theme.display_name_localized(lang)),
                Some(&format!("win.set-theme::{}", theme.id())),
            );
        }
        tools_menu.append_submenu(Some(lang.theme_label()), &theme_menu);

        let lang_menu = gio::Menu::new();
        let lang_entries = [
            (lang.language_auto_system_label(), "auto"),
            (lang.language_french_label(), "fr"),
            (lang.language_english_label(), "en"),
        ];
        for (label, id) in lang_entries {
            lang_menu.append(Some(label), Some(&format!("win.set-language::{id}")));
        }
        tools_menu.append_submenu(Some(lang.language_label()), &lang_menu);

        menubar_model.append_submenu(Some(lang.app_menu_tools()), &tools_menu);

        let help_menu = gio::Menu::new();
        help_menu.append(Some(lang.shortcuts_title()), Some("win.show-shortcuts"));
        help_menu.append(Some(lang.contact_label()), Some("win.show-contact"));
        help_menu.append(Some(lang.report_issue_label()), Some("win.report-issue"));
        help_menu.append(Some(lang.about_label()), Some("win.about"));
        menubar_model.append_submenu(Some(lang.app_menu_help()), &help_menu);

        let menu_bar = gtk4::PopoverMenuBar::from_model(Some(&menubar_model));
        main_box.append(&menu_bar);

        let action_toolbar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .margin_start(4)
            .margin_end(4)
            .margin_top(4)
            .margin_bottom(4)
            .build();
        action_toolbar.add_css_class("toolbar");

        header.toggle_sidebar_button.add_css_class("flat");
        action_toolbar.append(&header.toggle_sidebar_button);

        action_toolbar.append(&Separator::new(Orientation::Vertical));

        // Barre série fusionnée dans la toolbar principale.
        let stack_clone = connection_panel.toolbar_stack.clone();
        stack_clone.set_hexpand(true);
        action_toolbar.append(&stack_clone);

        action_toolbar.append(&Separator::new(Orientation::Vertical));

        header.save_log_button.add_css_class("flat");
        action_toolbar.append(&header.save_log_button);

        action_toolbar.append(&Separator::new(Orientation::Vertical));

        let connect_button = Button::builder()
            .icon_name("network-wired-symbolic")
            .tooltip_text(lang.connect_tooltip())
            .build();
        connect_button.add_css_class("suggested-action");

        let clear_button = Button::builder()
            .icon_name("edit-clear-all-symbolic")
            .tooltip_text(lang.clear_terminal_tooltip())
            .build();
        clear_button.add_css_class("flat");

        action_toolbar.append(&connect_button);
        action_toolbar.append(&clear_button);

        main_box.append(&action_toolbar);
        main_box.append(&Separator::new(Orientation::Horizontal));

        let split_view = libadwaita::OverlaySplitView::builder()
            .show_sidebar(false) // Panneau MASQUÉ au démarrage par défaut.
            .min_sidebar_width(250.0)
            .max_sidebar_width(320.0)
            .vexpand(true)
            .build();

        let sidebar_scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .child(&connection_panel.container)
            .build();

        split_view.set_sidebar(Some(&sidebar_scroll));

        header
            .toggle_sidebar_button
            .bind_property("active", &split_view, "show-sidebar")
            .sync_create()
            .bidirectional()
            .build();

        let input_separator = Separator::new(Orientation::Horizontal);
        let content_box = GtkBox::builder().orientation(Orientation::Vertical).build();
        content_box.append(&terminal.container);
        content_box.append(&input_separator);
        content_box.append(&input.container);

        split_view.set_content(Some(&content_box));
        main_box.append(&split_view);

        let toast_overlay = libadwaita::ToastOverlay::new();
        toast_overlay.set_child(Some(&main_box));

        let status_bar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(12)
            .margin_end(12)
            .margin_top(4)
            .margin_bottom(4)
            .build();
        status_bar.add_css_class("status-bar");

        let status_dot = Label::builder().label("●").build();
        status_dot.add_css_class("status-dot-disconnected");

        let status_label = Label::builder()
            .label(lang.status_disconnected())
            .halign(gtk4::Align::Start)
            .build();
        status_label.add_css_class("status-text");

        let left_status_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .hexpand(true)
            .halign(gtk4::Align::Start)
            .build();
        left_status_box.append(&status_dot);
        left_status_box.append(&status_label);

        let bottom_notice_label = Label::builder()
            .label("")
            .hexpand(true)
            .halign(gtk4::Align::Center)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();
        bottom_notice_label.add_css_class("dim-label");

        let version_label = Label::builder()
            .label(concat!("v", env!("CARGO_PKG_VERSION")))
            .halign(gtk4::Align::End)
            .build();
        version_label.add_css_class("dim-label");

        let io_bytes_label = Label::builder().label("").halign(gtk4::Align::End).build();
        io_bytes_label.add_css_class("dim-label");
        io_bytes_label.add_css_class("monospace");

        let right_status_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .hexpand(true)
            .halign(gtk4::Align::End)
            .build();
        right_status_box.append(&io_bytes_label);
        right_status_box.append(&version_label);

        status_bar.append(&left_status_box);
        status_bar.append(&bottom_notice_label);
        status_bar.append(&right_status_box);

        let toolbar_view = libadwaita::ToolbarView::new();
        toolbar_view.add_top_bar(&header.header_bar);
        toolbar_view.set_content(Some(&toast_overlay));
        toolbar_view.add_bottom_bar(&status_bar);
        window.set_content(Some(&toolbar_view));

        let saved_theme = settings.borrow().settings().ui.theme.clone();
        let theme = Theme::from_str_name(&saved_theme);
        ThemeManager::apply(theme);
        if saved_theme != theme.id() {
            settings.borrow_mut().set_theme(theme.id());
        }

        let main_win = Rc::new(Self {
            window,
            header,
            connection_panel,
            terminal,
            input,
            settings,
            connection_tx: RefCell::new(None),
            runtime,
            toast_overlay,
            status_label,
            bottom_notice_label,
            status_dot,
            bottom_notice_generation: Rc::new(Cell::new(0)),
            connect_button,
            clear_button,
            connection_generation: Cell::new(0),
            reconnect_generation: Cell::new(0),
            rx_bytes: Cell::new(0),
            tx_bytes: Cell::new(0),
            last_terminal_grid: Cell::new((0, 0, 0, 0)),
            io_bytes_label,
            lang,
            input_separator,
        });

        // Injecter l'accueil seulement après le mapping réel de la fenêtre pour
        // laisser la pile d'outils et le terminal stabiliser leur géométrie.
        defer_terminal_welcome_until_mapped(
            &main_win.window,
            &main_win.bottom_notice_label,
            &main_win.bottom_notice_generation,
            main_win.lang,
        );

        // setup_actions doit précéder restore_ui_from_settings : les actions
        // set-theme et set-language doivent exister pour que restore puisse
        // appeler change_state sur elles.
        Self::setup_actions(&main_win);
        main_win.restore_ui_from_settings();
        Self::setup_signals(&main_win);

        // Connexion automatique au démarrage si l'option est activée.
        main_win.maybe_start_auto_connect();

        main_win.window.present();
        Some(main_win)
    }
}

#[cfg(test)]
mod tests;
