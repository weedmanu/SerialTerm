# Scripts du projet

Sous Linux, `install-deps.sh` installe les outils nécessaires et `install-hooks.sh` active les vérifications Git. `pre-commit-checks.sh` lance les contrôles de qualité. `build-deb.sh` construit le paquet Debian. `run-soak-test.sh` lance un essai prolongé de l'affichage.

Sous Windows, `install-deps-windows.ps1` prépare les outils et `pre-commit-checks-windows.ps1` lance les contrôles disponibles. `build-exe.ps1` prépare la version portable. `build-installer.ps1` construit l'installateur.

Les commandes et leurs prérequis sont décrits dans le [guide de développement](../DEVELOPMENT.md) et dans les [guides de création des paquets](../PACKAGING.md).
