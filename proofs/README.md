# Rapports de vérification

Le dossier `raw/` conserve des journaux et des codes de sortie de contrôles antérieurs. Certains journaux de cette période ne sont plus présents. Les fichiers `.exit` seuls indiquent un résultat, sans donner le détail de l'exécution.

Les journaux encore disponibles concernent le formatage, la compilation, Clippy et l'audit des dépendances :

- [Formatage](raw/cargo_fmt_check.log)
- [Compilation](raw/cargo_check.log)
- [Clippy](raw/cargo_clippy_strict.log)
- [Audit des dépendances](raw/cargo_audit.log)

Les [sommes SHA-256 du 24 mars 2026](raw/SHA256SUMS_2026-03-24.txt) se rapportent à l'ancien lot de fichiers. Pour valider le code actuel, exécutez les commandes du [guide de développement](../DEVELOPMENT.md).
