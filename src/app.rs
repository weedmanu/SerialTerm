//! @file       app.rs
//! @author     M@nu
//! @brief      Bootstrap de l'application GTK4/Libadwaita.
//! @version    1.0.0
//! @date       2025-03-05
//! @copyright  GPL-3.0-or-later
//!
//! Ce module contient l'unique fonction publique `run()` qui :
//! 1. Crée un [`libadwaita::Application`] avec son identifiant D-Bus.
//! 2. Connecte le signal `activate` pour construire la fenêtre principale.
//! 3. Lance la boucle d'événements GTK et retourne le code de sortie.
//!
//! La variable `main_window` est maintenue en vie pendant toute l'exécution
//! grâce à un `Rc<RefCell<Option<Rc<MainWindow>>>>` capturé dans la closure
//! du signal `activate`. Sans cela, GTK4 détruirait la fenêtre immédiatement
//! après la fin du bloc `move |app|`.

use std::cell::RefCell; // Mutabilité intérieure pour `Option<Rc<MainWindow>>`.
use std::rc::Rc; // Compteur de références simple-thread (pas besoin d'Arc).

use gtk4::prelude::*; // Traits GTK : `ApplicationExt`, `GtkApplicationExt`, …

use crate::ui::runtime::sanitize_problematic_desktop_theme;
use crate::ui::window::MainWindow; // Fenêtre principale construite par `shell.rs`.
use gtk4::gio;

/// Lit une variable d'environnement et retourne `true` si elle est définie
/// et n'est pas l'une des valeurs "fausse" canoniques (`0`, `false`, `no`, `off`).
pub fn soak_env_flag(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| {
        let value = value.trim();
        !value.is_empty()
            && !matches!(
                value,
                "0" | "false" | "FALSE" | "False" | "no" | "NO" | "off" | "OFF"
            )
    })
}

fn soak_mode_requested() -> bool {
    soak_env_flag("SERIAL_TERM_SOAK_GENERATOR")
}

/// Lance l'application GTK4/Libadwaita et retourne le code de sortie.
///
/// Cette fonction bloque jusqu'à la fermeture de toutes les fenêtres
/// (boucle d'événements `GLib`).
pub fn run() -> glib::ExitCode {
    let soak_mode = soak_mode_requested();
    sanitize_problematic_desktop_theme();

    // Créer l'application Adwaita avec son identifiant D-Bus unique.
    // Cet identifiant est utilisé par le bureau (notifications, répertoire de config…).
    let app = libadwaita::Application::builder()
        .application_id("io.github.TutoElectroWeb.SerialTerm") // ID reverse-DNS unique.
        .flags(if soak_mode {
            gio::ApplicationFlags::NON_UNIQUE
        } else {
            gio::ApplicationFlags::empty()
        })
        .build();

    // Maintenir la fenêtre en vie tout au long de l'exécution.
    // `Rc<RefCell<Option<…>>>` permet une mutation intérieure depuis la closure.
    let main_window: Rc<RefCell<Option<Rc<MainWindow>>>> = Rc::new(RefCell::new(None));

    // Renommer pour éviter le move partiel (le compilateur ne peut pas capturer
    // `main_window` et le réutiliser dans le même scope après le move).
    let mw = main_window; // Déplace l'ownership dans la closure `activate`.

    app.connect_activate(move |app| {
        // `activate` est appelé une fois à l'ouverture ou à chaque re-focus.
        if let Some(win) = MainWindow::new(app) {
            win.start_soak_mode_from_env();
            // Construire la fenêtre principale.
            *mw.borrow_mut() = Some(win); // Stocker pour empêcher le drop.
        }
        // Si `MainWindow::new` retourne `None` (erreur Tokio), l'app se ferme via `app.quit()`.
    });

    app.run() // Lancer la boucle d'événements — bloquant jusqu'à fermeture.
}

use gtk4::glib; // Re-import pour le type `glib::ExitCode` visible depuis `main.rs`.
