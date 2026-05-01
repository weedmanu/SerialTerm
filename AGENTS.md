# AGENTS

Ce dépôt est le dépôt principal de développement actif.

## Rôle du dépôt

- `weedmanu/SerialTerm` est le dépôt de développement principal.
- `TutoElectroWeb/SerialTerm` sert de dépôt de publication, de synchronisation et de diffusion.

## Règle pour les agents IA

Si une demande implique du développement actif, des tests, de la refactorisation, de l'architecture, du packaging Debian ou des correctifs de code, le travail doit être fait ici.

Si une demande vise seulement la publication ou la synchronisation vers `TutoElectroWeb/SerialTerm`, l'agent peut alors travailler dans le dépôt TutoElectroWeb.

## Consigne générale par bloc technologique

Pour tout bloc technologique actif, les agents doivent maintenir trois éléments ensemble, pas séparément :

1. la liste des outils requis pour travailler correctement sur ce bloc ;
2. le script d'installation ou de préparation locale correspondant ;
3. un hook versionné strict avec tous les contrôles réellement pertinents pour ce bloc.

Règle d'application :

- un bloc Rust + GTK doit documenter et préparer au minimum `cargo`, `rustfmt`, `clippy`, les outils d'audit Cargo, `xvfb-run` et les dépendances système GTK/Libadwaita ;
- un bloc embarqué STM32 doit documenter et préparer la chaîne de cross-compilation, le flash/debug, le formatage et les vérifications de build réellement utilisées par le dépôt concerné ;
- un bloc Windows (scripts PowerShell, build EXE/installateur) doit documenter et préparer au minimum PowerShell 7+, les dépendances GTK4 pour Windows (MSYS2 ou vcpkg), la chaîne `cargo build --release --target x86_64-pc-windows-gnu` et les scripts de packaging versionnés ; le hook Windows n'est pas obligatoire localement mais doit être documenté pour les agents travaillant depuis une machine Windows ou via WSL ;
- le hook doit rester strict mais cohérent avec le projet : il doit bloquer sur les contrôles qualité réellement exigés par le bloc, pas sur des outils annexes non stabilisés ou non applicables.

Conséquence pratique :

- ne pas ajouter une simple mention documentaire sans installer ou vérifier l'outillage correspondant ;
- ne pas annoncer un hook strict si des contrôles attendus du bloc manquent encore ;
- quand un nouveau bloc apparaît dans un dépôt, compléter la doctrine générale selon le même schéma.

## Flux recommandé

1. développer dans `weedmanu/SerialTerm` ;
2. valider localement ;
3. publier ou synchroniser ensuite vers `TutoElectroWeb/SerialTerm`.
