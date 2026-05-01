# Debian Packaging

Contient toutes les métadonnées et directives nécessaires à la construction d'un paquet logiciel propre au système de distribution Debian (`.deb`).

L'organisation suit les standards `debhelper` via le répertoire `debian/` :

- `control` : Définit les listes de dépendances, descriptions et maintainer.
- `rules` : Contient les cibles de type Makefile appelées par l'outil de packaging `debuild`.
- `serial-term.desktop` : Fichier de lanceur d'application standard FreeDesktop.org pour l'intégration de GNOME/KDE, avec labels localisés, indexation de recherche et rattachement propre a l'`application_id` GTK.
