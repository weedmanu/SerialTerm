# SerialTerm

[Français] | [English](README.en.md)

SerialTerm est une application graphique pour communiquer avec un appareil par port série. Elle permet de régler la connexion, de lire les données reçues et d'enregistrer les échanges dans un fichier texte. L'interface utilise GTK 4 et Libadwaita.

La version 1.0.0 prend en charge les connexions série.

## Fonctions

- Réglage du port, du débit, des bits de données, de la parité, des bits d'arrêt et du contrôle de flux.
- Affichage des couleurs et des styles ANSI courants, avec recherche et historique limité.
- Reconnexion automatique en option après une déconnexion.
- Choix de la fin de ligne lors de l'envoi : aucune, LF, CR ou CRLF.
- Export du texte reçu et envoyé, avec ou sans horodatage.
- Calculatrice, convertisseur décimal, hexadécimal et binaire, et trois thèmes d'affichage.
- Interface en français et en anglais.

SerialTerm affiche les séquences ANSI prises en charge dans une vue de texte. Il ne fournit pas une émulation complète de terminal à grille.

## Installation

### Paquet Debian

```bash
sudo dpkg -i dist/debian/serial-term_*.deb
sudo apt -f install
```

Pour accéder aux ports série sans privilèges administrateur, ajoutez votre compte au groupe `dialout`, puis ouvrez une nouvelle session :

```bash
sudo usermod -aG dialout "$USER"
```

### Compilation depuis les sources

Installez Rust 1.85 ou une version plus récente, GTK 4.14, Libadwaita 1.5 et leurs bibliothèques de développement. Vérifiez la version de Rust avant de compiler.

```bash
sudo apt install build-essential libgtk-4-dev libadwaita-1-dev libudev-dev pkgconf
rustc --version
cargo build --release
./target/release/serial-term
```

Le script [install-deps.sh](scripts/install-deps.sh) prépare aussi Debian, Ubuntu, Fedora et Arch Linux.

## Premiers pas

1. Branchez l'appareil série.
2. Choisissez son port. Sous Linux, l'application privilégie les chemins stables de `/dev/serial/by-id/` lorsqu'ils existent.
3. Réglez le débit et les autres paramètres de la liaison.
4. Cliquez sur **Connecter**, puis saisissez une commande en bas de la fenêtre.

Chaque connexion démarre avec un affichage vide. Les anciennes données encore présentes dans les tampons du port sont purgées. Une sauvegarde doit donc être faite avant de se reconnecter si vous souhaitez conserver la session affichée.

Les préférences sont enregistrées dans `~/.config/serial-term/settings.json`. Si ce fichier contient un JSON invalide, l'application avertit l'utilisateur et en conserve une copie sous le nom `settings.json.bak` avant la prochaine sauvegarde.

Pour contribuer, consultez le [guide de développement](DEVELOPMENT.md). Le [plan de corrections](PLAN_CORRECTIONS_AMELIORATIONS.md) recense les travaux en cours.

## Licence

SerialTerm est distribué sous licence [GPL-3.0-or-later](LICENSE).
