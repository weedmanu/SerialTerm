# Plan de corrections et d'améliorations

Cette feuille de route priorise les travaux nécessaires pour rendre SerialTerm reproductible, robuste et plus complet. Une phase est terminée uniquement lorsque ses critères de validation sont satisfaits.

## P0 - Garantir une livraison reproductible

- [ ] Vérifier que les forks `vendor/serialport` et `vendor/unescaper` sont présents dans chaque archive source publiée.
- [ ] Documenter les correctifs portés par ces forks et leurs critères de retour aux versions crates.io.
- [ ] Vérifier que `assets/icon.svg`, les scripts de construction et les workflows CI restent dans les branches servant aux publications.
- [ ] Ajouter `libudev-dev` aux dépendances de construction Debian.
- [ ] Remplacer `Architecture: amd64` par `Architecture: any`, puis valider amd64 et arm64.
- [ ] Construire les paquets uniquement depuis un tag Git afin de relier chaque binaire à ses sources.

Critères de validation :

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
dpkg-buildpackage -us -uc
```

## P1 - Fiabiliser les communications et les données

- [ ] Remplacer l'écriture série partielle par `write_all`.
- [ ] Émettre un accusé d'envoi depuis l'acteur avant de mettre à jour TX et l'écho local.
- [ ] Tester les écritures partielles, la saturation du canal et la déconnexion pendant un envoi.
- [ ] Sauvegarder les paramètres dans un fichier temporaire puis effectuer un renommage atomique.
- [ ] Conserver en `.bak` une configuration JSON invalide et prévenir l'utilisateur.
- [ ] Déplacer toutes les énumérations de ports hors du thread GTK, y compris au démarrage et pendant la reconnexion.

Critères de validation : aucune troncature sous charge, interface réactive pendant l'énumération et récupération possible après interruption d'une sauvegarde.

## P2 - Clarifier et enrichir l'expérience terminal

- [x] Afficher uniquement le port `/dev/tty...` dans le sélecteur supérieur.
- [x] Afficher le nom convivial du périphérique dans la barre d'état, à côté de l'état et du point coloré, sans répéter le port.
- [ ] Choisir entre l'appellation « moniteur série ANSI » et une émulation terminal complète fondée sur une grille VTE.
- [ ] Ajouter une vue HEX/TEXTE simultanée et l'envoi d'octets arbitraires.
- [ ] Ajouter des profils nommés par périphérique et un historique persistant des commandes.
- [ ] Ajouter les commandes DTR, RTS et BREAK.
- [ ] Ajouter une capture binaire brute et l'envoi de fichiers avec progression et annulation.

Critères de validation : les fonctionnalités annoncées dans les README correspondent exactement aux séquences et formats réellement pris en charge.

## P3 - Observabilité et qualité

- [ ] Afficher le débit RX/TX instantané, la durée de session et le nombre d'erreurs.
- [ ] Tester la reconnexion avec changement de nom `/dev/ttyUSBx` et alias stable `by-id`.
- [ ] Ajouter un test prolongé du scrollback et du parseur ANSI avec mesure mémoire.
- [ ] Exécuter `cargo audit` et `cargo deny` en CI.
- [ ] Archiver le rapport CI et les sommes SHA-256 pour chaque version.

Critères de validation : rapport CI disponible pour chaque version et paquet reconstruisible depuis le tag correspondant.

## Ordre recommandé

1. Terminer P0 avant toute nouvelle publication.
2. Traiter l'écriture série et la persistance de P1 avant les enrichissements fonctionnels.
3. Livrer P2 par petits lots testables.
4. Intégrer P3 à la préparation de la prochaine version stable.