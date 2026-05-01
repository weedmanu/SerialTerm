# Packaging Debian

Guide de production du paquet `.deb` pour SerialTerm.

Référence : [PACKAGING.md](PACKAGING.md) (index général) · [DEVELOPMENT.md](DEVELOPMENT.md) (validation développeur)

## Structure des fichiers Debian

```
debian/
├── control                      # Métadonnées du paquet
├── rules                        # Règles de build (exécutable)
├── changelog                    # Historique des versions
├── copyright                    # Licence (format DEP-5)
├── compat                       # Version debhelper (13)
├── source/format                # Format source (3.0 native)
└── serial-term.desktop      # Entrée FDO Desktop (bilingue FR/EN)
```

## Prérequis

```bash
./scripts/install-deps.sh   # Installe build-essential, debhelper, devscripts, Rust, GTK4…
```

## Build du .deb

```bash
bash scripts/build-deb.sh
```

Le script exécute `cargo build --release` puis appelle `debuild` qui:
- invoque `debian/rules`,
- génère `serial-term_<version>_amd64.deb`, `.dsc` et `.tar.xz`.

Les artefacts sont déposés dans `build-dir/`.

## Installation locale

```bash
sudo dpkg -i build-dir/serial-term_*.deb
```

## Référence des fichiers

### debian/control

Métadonnées du paquet :
- `Package` : `serial-term`
- `Architecture` : `amd64` (cross-architecture possible via `rustup target add`)
- `Depends` : `libc6 (≥ 2.31)`, `libgtk-4-1 (≥ 4.0)`, `libadwaita-1 (≥ 0.7)`
- `Maintainer` : M@nu

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

### debian/serial-term.desktop

Entrée FDO bilingue FR/EN. Champs clés :
- `Exec` / `TryExec` : binaire `serial-term`
- `StartupWMClass` : `io.github.TutoElectroWeb.SerialTerm`
- `Categories` : `System;TerminalEmulator;GTK;`

## CI automatisée

Le workflow [`.github/workflows/package-deb.yml`](.github/workflows/package-deb.yml) déclenche un build `.deb` automatiquement lors d'un push de tag `v*` ou manuellement via `workflow_dispatch`.

## Dépendances de build

| Outil | Rôle |
|-------|------|
| `rustc` + `cargo` ≥ 1.75 | Compilateur |
| `libgtk-4-dev`, `libadwaita-1-dev` | Headers GTK4/Libadwaita |
| `libudev-dev` | Headers udev |
| `debhelper`, `devscripts`, `fakeroot` | Outils Debian |
| `pkg-config`, `cmake`, `clang` | Build helpers |

## Troubleshooting

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
dpkg -I build-dir/serial-term_*.deb | grep Depends
```

### Lint Lintian

```bash
lintian build-dir/serial-term_*.deb
```

## Cross-compilation

```bash
rustup target add aarch64-unknown-linux-gnu
cargo build --target aarch64-unknown-linux-gnu --release
```

Adapter ensuite `debian/rules` et `debian/control` (champ `Architecture`).

## Nouvelle version

1. Incrémenter la version dans `Cargo.toml`.
2. Ajouter une entrée dans `debian/changelog` (`dch -i`).
3. Relancer `bash scripts/build-deb.sh`.
