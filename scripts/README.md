# Scripts

Ensemble de scripts utilitaires pour l'environnement de développement et le déploiement continu.

Fichiers bash (`.sh`) :

- `install-deps.sh` : Résolution des dépendances systèmes pour Debian/Ubuntu.
- `build-deb.sh` : Orchestrateur de compilation pour la création de l'installateur `.deb`.
- `pre-commit-checks.sh` : Exécute la batterie de contrôles locale utilisée par le hook Git de pré-commit (`fmt`, `check`, `clippy`, `doc -D warnings`, `test`, `audit`, `deny`).
- `run-soak-test.sh` : Lance l'application en mode soak test avec générateur de charge terminal intégré, diagnostics périodiques, durée bornable via environnement, fermeture automatique et capture des logs runtime. La qualification renforcée visée par le dépôt correspond à un run de 30 minutes en charge lourde, pas à plusieurs heures passives.

Fichiers PowerShell (`.ps1`) :

- `install-deps-windows.ps1` : Récupération automatique avec winget/MSYS2 des libs C/GTK dédiées à l'OS Windows.
- `pre-commit-checks-windows.ps1` : Validation versionnée du bloc Windows (`fmt`, `check`, `clippy`, `doc -D warnings`, `test`, `audit`, `deny`, `machete`, build `x86_64-pc-windows-gnu`).
- `build-exe.ps1` : Création de la version portable pour Windows (`.zip` et `.exe`).
- `build-installer.ps1` : InnoSetup CLI runner pour le paquet setup d'installation global Windows.
