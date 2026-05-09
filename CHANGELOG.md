# Changelog

Toutes les modifications notables de **SerialTerm** sont documentées dans ce fichier.

Le format suit [Keep a Changelog 1.1.0](https://keepachangelog.com/fr/1.1.0/) et le projet adhère à [Semantic Versioning](https://semver.org/lang/fr/).

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

**MSRV :** `1.75`. Une élévation de la MSRV est traitée comme un changement `MINOR` documenté dans la rubrique « Changed » de la version concernée.

## [1.0.0] - 2026-05-08

### Changed

- Passage de la version applicative à `1.0.0`.
- Alignement des métadonnées Debian et de la documentation de packaging.
- Correction du workflow de packaging Debian pour publier les artefacts générés dans `dist/debian/`.
- Harmonisation des en-têtes source avec la licence `GPL-3.0-or-later` déclarée par le projet.

## [0.95.0] - 2026-05-01

### Changed

- Mise à jour de la version applicative vers `0.95.0`.
- Harmonisation du nom produit **SerialTerm** dans les scripts et la documentation.
- Mise à jour des exemples d'installation Debian vers `serial-term_0.95.0*.deb`.

[1.0.0]: https://github.com/TutoElectroWeb/SerialTerm/releases/tag/v1.0.0
[0.95.0]: https://github.com/TutoElectroWeb/SerialTerm/releases/tag/v0.95.0
