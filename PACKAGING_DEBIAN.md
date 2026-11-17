# Packaging Debian

Guide de production du paquet `.deb` pour SerialTerm.

Consultez également [PACKAGING.md](PACKAGING.md) et [DEVELOPMENT.md](DEVELOPMENT.md).

## Fichiers du paquet

`debian/control` déclare les dépendances. `debian/rules` construit et installe le programme. Les autres fichiers fournissent l'historique des versions, la licence, la page de manuel et le lanceur de bureau.

## Prérequis

```bash
./scripts/install-deps.sh   # Installe build-essential, debhelper, devscripts, Rust, GTK 4
```

## Construire le paquet

```bash
bash scripts/build-deb.sh
```

Le script exécute `cargo build --release`, puis appelle `debuild`. Ce dernier utilise `debian/rules` pour produire `serial-term_<version>_amd64.deb`.

Le paquet est placé dans `dist/debian/`.

## Installation locale

```bash
sudo dpkg -i dist/debian/serial-term_*.deb
```

## Référence des fichiers

### debian/control

Métadonnées du paquet :
- `Package` : `serial-term`
- `Architecture` : `amd64`
- `Depends` : générées à partir des bibliothèques liées, avec les dépendances Debian requises
- `Maintainer` : Manu
- `Build-Depends` : inclut `debhelper-compat (= 13)`.

### debian/rules

Exécutable (`chmod +x debian/rules`). Utilise `dh` (debhelper mode modernisé) :

```makefile
override_dh_auto_build:
    cargo build --release

override_dh_auto_install:
    # Installe binaire + icône + entrée .desktop
```

### debian/changelog

Format strict `dch`. Pour incrémenter la version :

```bash
dch -i   # Ajoute une entrée et incrémente la version
```

### debian/copyright

Licence GPL-3.0-or-later au format DEP-5.

### debian/serial-term.1

Page de manuel installée avec le paquet pour documenter la commande `serial-term`.

### debian/serial-term.desktop

Entrée FDO bilingue FR/EN. Champs principaux :
- `Exec` / `TryExec` : binaire `serial-term`
- `StartupWMClass` : `io.github.TutoElectroWeb.SerialTerm`
- `Categories` : `System;TerminalEmulator;GTK;`

## CI automatisée

Le workflow [`.github/workflows/package-deb.yml`](.github/workflows/package-deb.yml) déclenche un build `.deb` automatiquement lors d'un push de tag `v*` ou manuellement via `workflow_dispatch`.

## Dépendances de build

| Outil | Rôle |
|-------|------|
| `rustc` + `cargo` ≥ 1.85 | Compilateur |
| `libgtk-4-dev`, `libadwaita-1-dev` | Fichiers de développement GTK 4 et Libadwaita |
| `libudev-dev` | Fichiers de développement udev |
| `debhelper`, `devscripts`, `fakeroot` | Outils Debian |
| `pkgconf`, `cmake`, `clang` | Outils de compilation |

## Dépannage

### `debuild: command not found`

```bash
sudo apt install devscripts
```

### `cargo not found`

```bash
./scripts/install-deps.sh
```

### Vérifier les dépendances du .deb généré

```bash
dpkg -I dist/debian/serial-term_*.deb | grep Depends
```

### Vérifier avec Lintian

```bash
lintian dist/debian/serial-term_*.deb
```

## Autres architectures

Le paquet est actuellement déclaré pour `amd64` dans `debian/control`. Une version `arm64` nécessiterait une chaîne de compilation, des bibliothèques GTK et udev pour cette architecture, ainsi qu'une validation du paquet sur une machine adaptée. Ajouter la cible Rust ne suffit pas à produire un paquet Debian vérifié.

## Nouvelle version

1. Incrémenter la version dans `Cargo.toml`.
2. Ajouter une entrée dans `debian/changelog` (`dch -i`).
3. Relancer `bash scripts/build-deb.sh`.
