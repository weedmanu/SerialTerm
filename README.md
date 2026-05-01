# SerialTerm

[FR] | [EN](README.en.md)

SerialTerm est une application de terminal graphique GTK4/Libadwaita dédiée à la communication série. Elle permet de configurer finement un port série (débit, bits, parité, contrôle de flux), d'afficher en temps réel les flux ANSI complets et de sauvegarder les sessions.

> **Version v0.95 — Série uniquement.** Cette version se concentre exclusivement sur les usages série.

## Fonctionnalités

- Connexion série configurable (débit, bits, parité, arrêt, flux, timeout) ;
- Émulation terminal ANSI complète (couleurs, SGR, redimensionnement PTY) ;
- Affichage en temps réel avec scrollback, recherche, copier/coller ;
- Détection automatique des branchements/débranchements USB et reconnexion automatique ;
- Outils intégrés : calculatrice et convertisseur DEC/HEX/BIN ;
- Sauvegarde des logs en fichier texte (avec ou sans horodatage) ;
- Thèmes (Clair, Sombre, Hacker) ;
- Configuration persistante en JSON ;
- Interface bilingue (FR/EN).

## Installation

### Paquet Debian (.deb)

```bash
sudo dpkg -i dist/debian/serial-term_0.95.0*.deb
sudo apt -f install   # si dépendances manquantes
```

Pour accéder aux ports série sans `sudo` :

```bash
sudo usermod -a -G dialout $USER
# puis se déconnecter / reconnecter
```

### Compilation depuis les sources

Prérequis minimaux : Rust 1.75+, GTK 4.14+, Libadwaita 1.5+.

```bash
sudo apt install build-essential libgtk-4-dev libadwaita-1-dev pkg-config cargo
```

```bash
cargo build --release
./target/release/serial-term
```

Voir [scripts/install-deps.sh](scripts/install-deps.sh) pour Debian/Ubuntu, Fedora et Arch.

## Utilisation rapide

1. Brancher le périphérique série (Arduino, ESP32, STM32, USB-TTL…).
2. Sélectionner le port dans la liste déroulante (les alias stables `/dev/serial/by-id/...` sont privilégiés).
3. Choisir débit, bits, parité, arrêt, flux et timeout.
4. Cliquer sur **Connecter**.
5. Saisir des commandes dans la zone de saisie en bas (avec choix de fin de ligne : aucune, LF, CR, CRLF).

## Configuration persistante

Les paramètres sont stockés dans :

```
~/.config/serial-term/settings.json
```

Le fichier est créé à la première utilisation et contient les préférences UI, le dernier port utilisé, les paramètres série et les options de logs.

## Architecture

Architecture hexagonale stricte :

- `src/core/` : logique métier sans dépendance GTK (settings, série, connexion) ;
- `src/application/` : cas d'usage (validation, transformation) ;
- `src/ui/` : présentation GTK4/Libadwaita.

Voir [DEVELOPMENT.md](DEVELOPMENT.md) pour les conventions, l'outillage et le gate de validation.

## Dépendances principales

| Crate          | Rôle                                          |
|----------------|-----------------------------------------------|
| `gtk4`         | Bindings GTK4                                 |
| `libadwaita`   | Composants Adwaita                            |
| `tokio`        | Runtime asynchrone                            |
| `tokio-serial` | Accès aux ports série                         |
| `serde_json`   | Persistance des paramètres                    |
| `vte`          | Parser ANSI (séquences d'échappement)         |
| `anyhow`       | Gestion d'erreurs                             |

## Licence

GPL-3.0+ — voir [LICENSE](LICENSE).
