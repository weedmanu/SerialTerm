# Développement de SerialTerm

Ce guide rassemble les commandes et les règles de validation du projet. La liaison série reste la priorité : les changements doivent préserver les données envoyées, la réactivité de l'interface et la récupération après une déconnexion.

## Préparer l'environnement

Sous Linux, installez les bibliothèques GTK 4, Libadwaita et udev, puis les outils Rust. Le script du dépôt prépare les distributions prises en charge et active les hooks Git :

```bash
./scripts/install-deps.sh
```

Pour activer les hooks séparément :

```bash
./scripts/install-hooks.sh
```

Sous Windows ou WSL, utilisez les scripts PowerShell :

```powershell
pwsh -ExecutionPolicy Bypass -File .\scripts\install-deps-windows.ps1
pwsh -ExecutionPolicy Bypass -File .\scripts\pre-commit-checks-windows.ps1
```

Le compilateur Rust minimal déclaré par le projet est la version 1.85. La construction Linux requiert aussi `libgtk-4-dev`, `libadwaita-1-dev`, `libudev-dev` et `pkgconf`.

## Vérifier un changement

Le script suivant lance les contrôles utilisés avant un commit et un envoi vers le dépôt distant :

```bash
bash scripts/pre-commit-checks.sh
```

Il vérifie le formatage, la compilation, Clippy, la documentation, les tests, les avis de sécurité et les règles de licences et de dépendances. `cargo-machete` est également lancé lorsqu'il est installé. Les avertissements de Clippy, les dépendances retirées du registre et les versions multiples interdites par `deny.toml` font échouer le contrôle.

Les tests GTK utilisent `xvfb-run` lorsqu'il est disponible. Pour les lancer directement sur une machine sans session graphique :

```bash
GDK_BACKEND=x11 LIBGL_ALWAYS_SOFTWARE=1 MESA_LOADER_DRIVER_OVERRIDE=llvmpipe xvfb-run -a cargo test --all-targets
```

Les mêmes contrôles principaux sont exécutés par la [CI](.github/workflows/ci.yml). Les hooks Git se trouvent dans `.githooks/`. Le hook de message refuse une première ligne vide ou de plus de 72 caractères.

Pour examiner les dépendances plus en détail :

```bash
cargo tree -d
cargo audit -q
cargo deny check
```

`cargo llvm-cov` peut servir à mesurer la couverture, et `cargo nextest` à exécuter les tests localement. Ces outils ne sont pas requis par le contrôle standard.

## Construire et essayer l'application

```bash
cargo build --release
./target/release/serial-term
```

Le générateur de charge de l'interface permet un essai prolongé sans appareil série :

```bash
SERIAL_TERM_SOAK_INTERVAL_MS=8 \
SERIAL_TERM_SOAK_LINES_PER_TICK=12 \
SERIAL_TERM_SOAK_DIAGNOSTICS_SECS=60 \
SERIAL_TERM_SOAK_DURATION_SECS=1800 \
bash scripts/run-soak-test.sh
```

Cet essai dure 30 minutes. Il renseigne sur la stabilité de l'affichage sous charge, mais ne remplace pas les essais avec un périphérique réel. Avant une publication, vérifiez aussi l'ouverture du port, l'envoi, le retrait à chaud, la reconnexion et le paquet produit.

Le paquet Debian se construit avec `./scripts/build-deb.sh`. Consultez [PACKAGING_DEBIAN.md](PACKAGING_DEBIAN.md) pour les prérequis et les fichiers générés. Les scripts `build-exe.ps1` et `build-installer.ps1` préparent les versions Windows.

## Organisation du code

`src/core/` gère les paramètres et la communication série. `src/application/` valide les données de l'interface. `src/ui/` contient les fenêtres et l'affichage du terminal. Le code de `core` ne doit pas dépendre de GTK.

Le lint `clippy::unwrap_used` est activé dans [src/main.rs](src/main.rs). Dans le code applicatif, traitez les erreurs au lieu d'appeler `unwrap` ou `expect`. Ces méthodes restent possibles dans les tests. Les sources importées dans `vendor/` sont maintenues séparément.

## Dépendances conservées dans `vendor/`

| Bibliothèque | Motif | Réexamen |
| --- | --- | --- |
| [serialport](vendor/serialport) | Correctifs locaux et maintenance amont incertaine, notamment sous Windows. | Vérifier régulièrement si une version amont stable peut remplacer ce fork. |
| [unescaper](vendor/unescaper) | Correctifs locaux dans une bibliothèque peu maintenue. | Chercher une version amont maintenue ou une autre bibliothèque adaptée. |

Une mise à jour majeure de Rust, de GTK ou de ces bibliothèques doit inclure un nouvel examen des forks.

## Ce qui reste à valider

Les tests automatisés couvrent la logique série et plusieurs comportements de l'interface. Ils ne prouvent pas encore le fonctionnement sur tous les adaptateurs série ni sur toutes les versions de Windows. La [feuille de route](PLAN_CORRECTIONS_AMELIORATIONS.md) suit les essais de reconnexion, les mesures de mémoire prolongées et les améliorations du paquet.

Les anciens résultats conservés dans [proofs/](proofs/) sont des archives. Pour qualifier une nouvelle version, produisez des résultats datés à partir du code et des dépendances de cette version.
