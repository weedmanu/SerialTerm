# Changelog

Toutes les modifications notables de **SerialTerm** sont documentées dans ce fichier.

Le journal suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) et le versionnement [SemVer](https://semver.org/lang/fr/).

## 1.1.0 · 2026-11-17

### Corrigé

- La reconnexion automatique attend le port d’origine et conserve l’historique.
  Sous Linux, un port USB ou Bluetooth sans alias stable requiert une
  reconnexion manuelle.
- Une ouverture réussie purge les tampons du port et réinitialise le parseur
  ANSI et les compteurs. Seule une connexion manuelle réussie efface
  l’historique précédent. Une erreur de purge empêche l’ouverture.
- Les échecs d’enregistrement du journal sont affichés dans l’interface.
- Le workflow Debian valide le commit puis produit le paquet et son archive
  source à partir des mêmes fichiers, avec leurs sommes SHA-256.
- Le compteur TX et l'écho local attendent la confirmation de l'acteur série.
  Une file d'envoi pleine conserve la saisie et affiche une erreur.
- Le délai d'envoi couvre l'écriture complète et l'attente du tampon pilote,
  sans `tcdrain` bloquant. La déconnexion abandonne les octets TX encore en attente.
- L'affichage conserve au plus 1 048 576 caractères. Les exports gardent au plus
  4 Mio de fragments terminés et un fragment courant de 64 Kio environ.
  Chaque cache de couleurs RGB est limité à 1 024 couleurs. Les nouvelles couleurs
  au-delà du plafond utilisent la couleur par défaut jusqu'à la prochaine purge.
- Sauvegarde des paramètres par fichier temporaire synchronisé puis renommage.
  Un JSON invalide déclenche un avertissement et est conservé en `.json.bak`
  avant remplacement.
- Énumération des ports hors du thread GTK, y compris au démarrage et en reconnexion.
- Dépendance de construction Debian `libudev-dev` déclarée.

### Modifié

- Tokio 1.54.1 et Jiff remplacent les dépendances signalées par RustSec.

- Minimum Rust annoncé aligné sur les dépendances verrouillées : 1.85.
- Le réglage de délai série est explicitement présenté comme un délai d'envoi.

## Politique SemVer

| Composant | Convention                                                                                                                                                                  |
| --------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `MAJOR`   | rupture d'API publique du binaire (CLI, fichier `settings.json`), suppression d'une plateforme supportée, retrait d'une fonctionnalité utilisateur.                          |
| `MINOR`   | ajout fonctionnel rétro-compatible (nouvel outil intégré, nouvelle langue), nouvelle clé optionnelle dans `settings.json` avec valeur par défaut conservée.                  |
| `PATCH`   | correction de bug, durcissement sécurité, mise à jour de dépendances, amélioration de documentation, refactor sans impact externe.                                           |

**Périmètre considéré comme API publique :**

- les arguments de ligne de commande de `serial-term` ;
- la structure documentée du fichier `~/.config/serial-term/settings.json` ;
- l'identifiant Freedesktop `io.github.TutoElectroWeb.SerialTerm` et le `.desktop` associé ;
- le contrat des paquets Debian publiés (`serial-term`).

**Hors périmètre (peut changer en `PATCH`) :**

- les modules internes Rust (`src/core`, `src/application`, `src/ui`) ;
- les forks supply-chain dans `vendor/` ;
- la mise en page exacte de l'interface graphique ;
- les libellés exacts des messages traduits (FR/EN).

**MSRV :** `1.85`. Une élévation de la MSRV est traitée comme un changement `MINOR` documenté dans la rubrique « Changed » de la version concernée.

## [1.0.0] : 2026-05-08

### Changed

- Passage de la version applicative à `1.0.0`.
- Alignement des métadonnées Debian et de la documentation de packaging.
- Correction du workflow de packaging Debian pour publier les artefacts générés dans `dist/debian/`.
- Harmonisation des en-têtes source avec la licence `GPL-3.0-or-later` déclarée par le projet.

## [0.95.0] : 2026-05-01

### Changed

- Mise à jour de la version applicative vers `0.95.0`.
- Harmonisation du nom produit **SerialTerm** dans les scripts et la documentation.
- Mise à jour des exemples d'installation Debian vers `serial-term_0.95.0*.deb`.

[1.0.0]: https://github.com/TutoElectroWeb/SerialTerm/releases/tag/v1.0.0
[0.95.0]: https://github.com/TutoElectroWeb/SerialTerm/releases/tag/v0.95.0
