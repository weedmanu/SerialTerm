# Guide Développeur

Ce document décrit l'outillage de développement retenu pour SerialTerm, le gate de validation du dépôt et les règles à suivre pour atteindre le niveau d'exigence interne retenu sur la connexion série.

Date de référence : 2026-03-26.

## Rôle de ce dépôt

Ce dépôt est le dépôt principal de développement actif.

Règle de travail retenue :

- développement, correctifs, tests, architecture et fonctionnalités : ici ;
- publication et synchronisation vers le dépôt public de diffusion : ensuite vers le dépôt public de diffusion.

Conséquence pratique :

- si vous démarrez une tâche de développement, vous devez la commencer ici ;
- le dépôt public de diffusion ne doit pas devenir la source principale des modifications de code.

Position documentaire retenue :

- [README.md](README.md) et [README.en.md](README.en.md) sont des documents utilisateur ;
- ce guide est le document développeur de référence ;
- la stratégie produit et la checklist d'acceptabilité sont désormais suivies hors dépôt (les anciens documents `BACKLOG.md`, `PLAN_DEVELOPPEMENT.md`, `PLAN_SPRINTS.md` et `TODO_PRE_RELEASE.md` ont été retirés).

## Objectif

Le dépôt suit désormais une ligne explicite :

- la connexion série est le cœur du produit ;
- le niveau cible est exigeant sur sécurité, fiabilité, validation et acceptabilité.

## Consigne générale par bloc technologique

Règle générale retenue pour ce dépôt, et à réappliquer dans les autres dépôts selon leur stack :

1. chaque bloc technologique actif doit avoir sa liste d'outils requise ;
2. chaque bloc technologique actif doit avoir un script de préparation locale cohérent ;
3. chaque bloc technologique actif doit avoir un hook versionné strict, aligné sur ses vrais contrôles qualité.

Interprétation pratique :

- pour un bloc Rust + GTK, le hook strict doit couvrir le formatage, la compilation de contrôle, l'analyse statique, les tests headless GTK et les vérifications supply-chain retenues ;
- pour un bloc STM32 ou embarqué dans un autre dépôt, la même logique s'applique avec les outils du bloc : toolchain croisée, build firmware, flash/debug si pertinent, formatage C/C++, analyse statique et tests disponibles ;
- un outil seulement "utile" ne devient pas automatiquement un contrôle bloquant du hook ; il doit être stabilisé, reproductible et défendable pour le projet concerné.

## Définition du gate développeur

Deux niveaux de gate coexistent.

### Gate standard

Le gate standard est le minimum de travail acceptable pour qu'un lot technique soit considéré comme propre dans le dépôt :

1. `cargo fmt --all -- --check`
2. `cargo check --all-targets`
3. `cargo clippy --all-targets --all-features -- -D warnings`
4. `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items`
5. `GDK_BACKEND=x11 LIBGL_ALWAYS_SOFTWARE=1 MESA_LOADER_DRIVER_OVERRIDE=llvmpipe xvfb-run -a cargo test --all-targets` si disponible, sinon `cargo test --all-targets`
6. `cargo audit -q`
7. `cargo deny check`
8. `cargo machete --with-metadata` si `cargo-machete` est installé

Règle d'interprétation stricte :

- aucun warning du gate n'est toléré ;
- un warning `cargo deny` sur `yanked` ou `multiple-versions` est traité comme un échec du gate, pas comme une note informative ;
- ce niveau d'exigence n'autorise pas une politique "warn aujourd'hui, on verra plus tard".

### Gate renforcé

Le gate renforcé ajoute aux vérifications précédentes :

1. traitement ou arbitrage explicite des duplications transitives ;
2. soak test significatif ;
3. checklist d'acceptabilité produit et packaging qualifiée (suivi hors dépôt) ;
4. preuves datées publiées dans `proofs/`.

Le gate standard rend un lot acceptable techniquement.
Le gate renforcé rend un jalon coeur produit réellement défendable.

## Pourquoi garder les outils versionnés dans le dépôt

Oui, il est utile de laisser les outils de validation et les scripts de flux dans le dépôt, y compris sur une branche de développement publique.

Raisons retenues :

- reproductibilité ;
- lisibilité ;
- maintenance ;
- auditabilité.

La documentation de ces outils n'a pas besoin d'encombrer les README utilisateur. C'est pour cela qu'elle est centralisée ici.

## Rôle des outils

### Outils Rust

#### Outils requis pour ce bloc Rust + GTK

> **Hooks Git versionnés actifs :**
>
> - `.githooks/pre-commit` : validation gate standard avant chaque commit.
> - `.githooks/pre-push` : re-validation gate standard avant chaque push.
> - `.githooks/commit-msg` : lint du message de commit (ligne 1 non vide, ≤ 72 caractères).
>
> Activation :
>
> ```bash
> git config core.hooksPath .githooks
> chmod +x .githooks/pre-commit .githooks/pre-push .githooks/commit-msg
> ```

#### Outils requis pour ce bloc Rust + GTK

- `cargo`
  - Point d'entrée standard de compilation, tests et sous-commandes Cargo.

- `rustfmt`
  - Requis par `cargo fmt --all -- --check`.

- `clippy`
  - Requis par `cargo clippy --all-targets --all-features -- -D warnings`.

- `cargo-audit`
  - Requis par le hook strict pour l'audit sécurité de la chaîne Cargo.

- `cargo-deny`
  - Requis par le hook strict pour la politique d'advisories, licences, sources et bans.
  - Dans ce dépôt, `yanked` et `multiple-versions` sont bloquants.

- `xvfb-run`
  - Requis pour exécuter les tests GTK headless de manière fiable en environnement sans session graphique active.
  - Dans ce dépôt, la commande headless de référence force le renderer logiciel Mesa avec `GDK_BACKEND=x11 LIBGL_ALWAYS_SOFTWARE=1 MESA_LOADER_DRIVER_OVERRIDE=llvmpipe` sous `Xvfb` pour supprimer le bruit `libEGL`/`DRI3` sans masquer les échecs GTK réels.

- `libgtk-4-dev`, `libadwaita-1-dev`, `pkg-config`, `libudev-dev`
  - Requis pour compiler correctement le bloc desktop Linux Rust + GTK de ce dépôt.

#### Outils utiles mais hors hook standard

- `cargo tree -d`
  - Inspection des duplications transitives.
  - Utile pour le gate renforcé et l'arbitrage supply-chain, mais non bloquant dans le hook standard actuel.

- `cargo llvm-cov`
  - Qualification couverture et preuves plus profondes.
  - Utile pour les jalons qualité, trop coûteux pour le hook standard.

- `cargo nextest`
  - Exécution de tests accélérée ou mieux instrumentée.
  - Utile localement, non imposé comme hook tant que la commande de référence reste `cargo test --all-targets`.

- `cargo machete`
  - Détection de dépendances Cargo inutiles.
  - Bloquant dans le hook standard si installé (voir `scripts/pre-commit-checks.sh`). S'installe via `scripts/install-deps.sh`.

- `cargo outdated`
  - Suivi des écarts de versions.
  - Informatif, pas un contrôle bloquant de pré-commit.

- `cargo deb`
  - Utile pour la production d'artefacts Debian.
  - Non pertinent dans le hook standard de développement.

- `cargo build --release`
  - Compile le binaire optimisé de production.
  - À utiliser avant un soak test, un paquet Debian ou une validation de performance.

- `cargo fmt --all -- --check`
  - Vérifie le formatage sans modifier les fichiers.
  - Toute divergence de formatage bloque le gate standard.

- `cargo check --all-targets`
  - Vérifie rapidement la compilation du binaire et des cibles de tests sans aller jusqu'à l'exécution.
  - Fait partie du hook strict de ce dépôt.

- `cargo clippy --all-targets --all-features -- -D warnings`
  - Exécute l'analyse statique Rust sur le binaire et les tests.
  - Tout warning Clippy devient une erreur de validation.

- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items`
  - Vérifie la documentation compilée, y compris les liens intra-doc, sans dépendances externes.
  - Fait partie du hook strict de ce dépôt.

- `cargo test --all-targets`
  - Exécute tous les tests du projet.
  - Couvre les cas d'usage applicatifs, les tests core et une partie des utilitaires UI non interactifs.
  - En headless GTK, la commande de référence retenue est `GDK_BACKEND=x11 LIBGL_ALWAYS_SOFTWARE=1 MESA_LOADER_DRIVER_OVERRIDE=llvmpipe xvfb-run -a cargo test --all-targets`.

- `cargo audit -q`
  - Vérifie les vulnérabilités connues de la chaîne de dépendances.
  - À horizon de fermeture complète, l'objectif est zéro advisory non arbitré.

- `cargo deny check`
  - Vérifie advisories, bans, licenses et sources.
  - À horizon de fermeture complète, l'objectif est aussi de réduire les duplications transitives évitables.

- `cargo tree -d`
  - Permet d'inspecter les dépendances dupliquées.
  - À utiliser quand la cible du lot implique une qualité supply-chain plus stricte que le gate standard.

### Outils système

- `GDK_BACKEND=x11 LIBGL_ALWAYS_SOFTWARE=1 MESA_LOADER_DRIVER_OVERRIDE=llvmpipe xvfb-run -a`
  - Lance les tests dans un display virtuel X11.
  - Sert à fiabiliser les tests touchant GTK sans session graphique active, sans bruit `libEGL`/`DRI3` sous `Xvfb`.

- `git config core.hooksPath .githooks`
  - Active le hook Git versionné du dépôt.

### Scripts du dépôt

- [scripts/pre-commit-checks.sh](scripts/pre-commit-checks.sh)
  - Point d'entrée unique de la validation locale standard.
  - Exécute `fmt`, `check`, `clippy`, `test`, `audit`, puis `deny`.

- [.githooks/pre-commit](.githooks/pre-commit)
  - Hook Git minimal.
  - Son rôle est d'appeler le script versionné de validation.

- [.githooks/pre-push](.githooks/pre-push)
  - Hook Git exécuté avant chaque `git push`.
  - Re-valide le gate standard pour éviter de pousser un lot non vérifié.

- [.githooks/commit-msg](.githooks/commit-msg)
  - Hook Git exécuté à chaque commit.
  - Vérifie que la ligne 1 du message n'est pas vide et ne dépasse pas 72 caractères.

- [scripts/install-deps.sh](scripts/install-deps.sh)
  - Installe les dépendances système de build sous Linux.
  - Installe aussi l'outillage utile au flux local : composants Rust, `xvfb`, `cargo-audit`, `cargo-deny`, et les outils Cargo complémentaires retenus pour ce bloc.

- [scripts/run-soak-test.sh](scripts/run-soak-test.sh)
  - Lance l'application en mode soak test avec diagnostics périodiques.
  - Fait partie du gate renforcé, pas du gate standard.

- [scripts/install-hooks.sh](scripts/install-hooks.sh)
  - Active `core.hooksPath = .githooks` et rend exécutables les hooks et le runner pré-commit.
  - Idempotent. Appelé automatiquement à la fin de `scripts/install-deps.sh`.

- [scripts/build-deb.sh](scripts/build-deb.sh)
  - Prépare l'artefact Debian local.
  - À utiliser après validation qualité.

- [scripts/build-exe.ps1](scripts/build-exe.ps1)
  - Prépare l'artefact Windows portable.

- [scripts/pre-commit-checks-windows.ps1](scripts/pre-commit-checks-windows.ps1)
  - Hook versionné de référence pour un agent ou développeur travaillant depuis Windows / WSL.
  - Couvre `fmt`, `check`, `clippy`, `doc -D warnings`, `test`, `audit`, `deny`, `machete` si disponible, puis le build `cargo build --release --target x86_64-pc-windows-gnu`.

- [scripts/build-installer.ps1](scripts/build-installer.ps1)
  - Prépare l'installateur Windows.

## Commandes utiles au projet

### Mise en route développeur

```bash
./scripts/install-deps.sh
# install-deps.sh appelle déjà scripts/install-hooks.sh à la fin.
# Pour activer les hooks indépendamment :
./scripts/install-hooks.sh
```

L'activation manuelle reste possible :

```bash
git config core.hooksPath .githooks
chmod +x .githooks/pre-commit .githooks/pre-push .githooks/commit-msg scripts/pre-commit-checks.sh
```

### Configuration Claude Code

Le dépôt versionne aussi une configuration Claude Code en deux niveaux :

- `.claude/settings.json` porte uniquement les règles partageables du dépôt : mode par défaut, permissions communes et hooks de sécurité/versioning ;
- `.claude/settings.local.json` reste réservé aux préférences personnelles ou machine ;
- `.claude/hooks/` contient les hooks partageables destinés à être commités ;
- `.claude/local-hooks/` contient les hooks strictement locaux, exclus du suivi Git via `.git/info/exclude`.

Règles de maintenance retenues :

- ne monter dans le partage que des règles reproductibles et défendables pour tout contributeur du dépôt ;
- garder le filtrage conditionnel directement dans les scripts de hook quand nécessaire ; dans cette configuration, le schéma local rejette le champ `if` sur les handlers ;
- conserver un hook `Stop` qui impose, après une réponse finale annonçant des modifications, une ligne dédiée de clôture du type `Validation: ...`, `Validation non faite: ...` ou `Non validé: ...`.

### Mise en route Windows / WSL

```powershell
pwsh -ExecutionPolicy Bypass -File .\scripts\install-deps-windows.ps1
pwsh -ExecutionPolicy Bypass -File .\scripts\pre-commit-checks-windows.ps1
```

### Boucle de développement rapide

```bash
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```

### Validation locale standard

```bash
bash scripts/pre-commit-checks.sh
```

### Inspection stricte des dépendances

```bash
cargo tree -d
cargo audit -q
cargo deny check
```

### Build de production

```bash
cargo build --release
./target/release/serial-term
```

### Soak test

```bash
bash scripts/run-soak-test.sh
```

Variante plus agressive :

```bash
SERIAL_TERM_SOAK_INTERVAL_MS=8 \
SERIAL_TERM_SOAK_LINES_PER_TICK=12 \
SERIAL_TERM_SOAK_DIAGNOSTICS_SECS=60 \
bash scripts/run-soak-test.sh
```

Recette de qualification renforcée retenue :

```bash
SERIAL_TERM_SOAK_INTERVAL_MS=8 \
SERIAL_TERM_SOAK_LINES_PER_TICK=12 \
SERIAL_TERM_SOAK_DIAGNOSTICS_SECS=60 \
SERIAL_TERM_SOAK_DURATION_SECS=1800 \
bash scripts/run-soak-test.sh
```

Interprétation retenue :

- le critère de fermeture n'est pas un soak de plusieurs heures à charge faible ;
- le critère utile est un soak de 30 minutes continues en charge lourde, journalisé et reproductible ;
- si ce run échoue, le lot runtime n'est pas fermable proprement.

### Packaging

```bash
./scripts/build-deb.sh
```

## Ordre conseillé avant commit

Ordre de référence :

1. `cargo fmt --all -- --check`
2. `cargo check --all-targets`
3. `cargo clippy --all-targets --all-features -- -D warnings`
4. `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items`
5. `GDK_BACKEND=x11 LIBGL_ALWAYS_SOFTWARE=1 MESA_LOADER_DRIVER_OVERRIDE=llvmpipe xvfb-run -a cargo test --all-targets`
6. `cargo audit -q`
7. `cargo deny check`

En pratique, `bash scripts/pre-commit-checks.sh` encapsule déjà cet ordre.

## Ordre conseillé avant jalon de fermeture complet

Pour un jalon coeur produit visant la fermeture complète, ajouter explicitement :

1. `cargo tree -d`
2. soak test documenté de 30 minutes en charge lourde
3. qualification de la checklist d'acceptabilité produit et packaging (suivi hors dépôt)
4. mise à jour des preuves datées

## Alignement avec la CI et le packaging

Référence CI : [.github/workflows/ci.yml](.github/workflows/ci.yml)

Correspondance actuelle :

- local et pré-commit : `fmt`, `check`, `clippy`, `doc -D warnings`, `test`, `audit`, `deny`, `machete` si disponible ;
- CI : `fmt`, `check`, `clippy`, `doc -D warnings`, `test`, `audit`, `deny`, `machete` hors `vendor/`, puis `build --release` ;
- packaging : s'appuie sur un build release déjà validé ;
- gate renforcé : va au-delà de la CI actuelle et suppose des validations supplémentaires documentées.

Conséquence pratique :

- la CI ne suffit pas à elle seule à qualifier un jalon de fermeture complet ;
- un tel jalon exige aussi des scénarios manuels, du soak test et des preuves.

## Portée des validations actuelles

Ce qui est déjà couvert ou partiellement couvert aujourd'hui :

- qualité de code Rust ;
- tests unitaires du dépôt ;
- audit des dépendances ;
- politique de licences et de sources ;
- base de soak test manuel ;
- preuves datées historiques.

Ce qui reste à couvrir pleinement pour la fermeture complète :

- UI interactive pilotée ou protocole manuel stable ;
- robustesse complète sur hot-unplug série ;
- réduction des warnings de duplication évitables ;
- mise à jour systématique des preuves à chaque jalon fort.

## Discipline documentaire

Quand un jalon significatif est fermé :

1. mettre à jour le suivi de backlog/plan/checklist d'acceptabilité (désormais maintenu hors dépôt) ;
2. publier un nouveau lot de preuves dans `proofs/` si le jalon doit être figé.

## Suivi des forks supply-chain (`vendor/`)

Le dépôt maintient deux forks vendorés. Pour chacun, conserver dans ce tableau le motif et le critère explicite de sortie.

| Fork | Motif | Critère de sortie | Action de suivi |
| ---- | ----- | ----------------- | --------------- |
| [vendor/serialport](vendor/serialport) | Upstream cherche mainteneurs (notamment Windows) ; correctifs `nix 0.29` / `bitflags` intégrés ici. | Reprise active du upstream OU adoption d'un fork de référence stable et publié. | Vérifier trimestriellement l'état du upstream. |
| [vendor/unescaper](vendor/unescaper) | Crate amont non maintenue (>5 ans), corrections nécessaires. | Apparition d'un mainteneur upstream ou substitution par une crate équivalente éprouvée. | Évaluer alternatives en cas de besoin de montée Rust majeure. |

Règle pratique :

- toute montée Rust ou GTK majeure revérifie chaque ligne ;
- toute disparition d'une raison de fork doit conduire à retirer le `vendor/` correspondant et à re-pinner la dépendance amont.

## État de la dette `unwrap`/`expect`

Audit du 2026-04-28 :

- politique : `clippy::unwrap_used` est activé niveau crate ([src/main.rs](src/main.rs)) ;
- toutes les occurrences `.unwrap()` ou `.expect()` du code applicatif `src/` sont sous `#[cfg(test)]`, sauf une seule occurrence historique en couche UI ([src/ui/window/actions.rs](src/ui/window/actions.rs)) qui a été remplacée par un `let … else { log::warn!… ; return; }` afin de respecter la politique zéro-panic ;
- les fichiers `vendor/*` ne sont pas soumis à cette règle (forks externes).

Procédure de vérification :

```bash
grep -rnE "\.(unwrap|expect)\(" src/ --include="*.rs" \
  | grep -v "#\[cfg(test)\]" \
  | grep -v vendor/
```

## Backlog technique post-1.0 (P2)

Points à instruire après clôture du jalon coeur produit, sans déclencher de modifications avant arbitrage explicite :

1. **Couverture UI** : ajouter du snapshot testing autour de `src/ui/terminal_panel/ansi.rs` (rendu ANSI) et de `src/ui/window/shell.rs` (lifecycle), en complément du headless `xvfb-run` actuel.
2. **Stratégie de sortie des forks** : voir le tableau ci-dessus ; arbitrer formellement un calendrier ou un trigger pour chaque ligne.
3. **E2E multi-plateformes** : qualifier le bloc Windows sur les flux Série via `serialport`.
4. **Soak CI hebdomadaire** : automatiser `scripts/run-soak-test.sh` en tâche planifiée (cron / GitHub Actions schedule), avec publication d'artefacts dans `proofs/raw/soak_*.exit`.

## Références

- [README.md](README.md)
- [README.en.md](README.en.md)
- [AGENTS.md](AGENTS.md)
- [PACKAGING.md](PACKAGING.md)
- [.github/workflows/ci.yml](.github/workflows/ci.yml)
- [scripts/README.md](scripts/README.md)
- [src/README.md](src/README.md)
