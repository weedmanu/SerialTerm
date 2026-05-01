# UI

Ce module prend en charge l'affichage graphique via l'écosystème **GTK4** et **Libadwaita**.

Organisation :

- `window/` : Définition de la fenêtre principale (Shell, Signaux, Actions).
- `connection_panel.rs` : Formulaires de connexion série.
- `terminal_panel.rs` : Intégration du terminal émulé VTE.
- `log_viewer.rs` : Outil de consultation des logs et filtrage textuel.
- `theme.rs` : Logique de basculement entre modes Clair / Sombre.
