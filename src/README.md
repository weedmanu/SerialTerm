# Source Code

Ce répertoire contient le code source de l'application **SerialTerm**.
L'architecture suit le principe de séparation des préoccupations (Clean Architecture) :

- `core/` : Logique métier et protocoles série. Totalement agnostique de l'interface graphique.
- `ui/` : Interface utilisateur implémentée avec GTK4 et Libadwaita.
- `application/` : Couche d'orchestration (Use Cases) reliant l'interface aux fonctions du backend.
- `app.rs` / `main.rs` : Point d'entrée et initialisation (bootstrap) du programme.
