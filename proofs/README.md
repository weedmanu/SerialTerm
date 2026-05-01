# Preuves et audits

Ce dossier contient des jeux de preuves datés.

Chaque jeu regroupe :

- un résumé lisible ;
- un index des fichiers bruts ;
- les journaux bruts dans `proofs/raw/` ;
- les empreintes SHA256 associées.

Les anciens résumés datés et index associés ont été retirés du dépôt. Les journaux bruts de référence restent conservés dans `proofs/raw/`.

Fichiers bruts actuels de référence :

- [proofs/raw/cargo_fmt_check.log](raw/cargo_fmt_check.log)
- [proofs/raw/cargo_check.log](raw/cargo_check.log)
- [proofs/raw/cargo_test_all_targets.log](raw/cargo_test_all_targets.log)
- [proofs/raw/cargo_clippy_strict.log](raw/cargo_clippy_strict.log)
- [proofs/raw/cargo_audit.log](raw/cargo_audit.log)
- [proofs/raw/cargo_deny_check.log](raw/cargo_deny_check.log)
- [proofs/raw/cargo_tree_duplicates.log](raw/cargo_tree_duplicates.log)
- [proofs/raw/cargo_llvm_cov_summary.log](raw/cargo_llvm_cov_summary.log)
- [proofs/raw/soak_2026-03-24.log](raw/soak_2026-03-24.log)
- [proofs/raw/SHA256SUMS_2026-03-24.txt](raw/SHA256SUMS_2026-03-24.txt)

Lecture recommandée :

1. lire ce fichier de synthèse ;
2. ouvrir ensuite les journaux bruts utiles dans `proofs/raw/` ;
3. vérifier enfin les empreintes SHA256 associées.

Le dernier lot publié avant nettoyage des archives documentait `121` tests, un durcissement testé du hot-unplug série Linux, un assainissement Debian/MATE du thème GTK `Yaru-MATE-*`, un soak qualifié de 30 minutes en charge lourde sans warning GTK/Adwaita visible, ainsi que les duplications transitives encore ouvertes à cette date.

La version v0.95 du dépôt conserve un périmètre strictement série. Les nouveaux lots de preuves doivent se concentrer sur la robustesse du panneau série (timeout I/O persisté, alias stables `/dev/serial/by-id/...`, hot-unplug, reconnexion automatique) et la stabilité GTK headless.
