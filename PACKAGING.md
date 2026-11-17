# Packaging

Guides de création des paquets SerialTerm.

La validation développeur est décrite dans [DEVELOPMENT.md](DEVELOPMENT.md).
La CI d'intégration continue est décrite dans [.github/workflows/ci.yml](.github/workflows/ci.yml).

## Guides disponibles

| Format                            | Guide                                        | CI                                                           |
| --------------------------------- | -------------------------------------------- | ------------------------------------------------------------ |
| **Debian (.deb)**                 | [PACKAGING_DEBIAN.md](PACKAGING_DEBIAN.md)   | [package-deb.yml](.github/workflows/package-deb.yml)         |
| **Windows (.exe / installateur)** | Section ci-dessous                           | Manuel (scripts PowerShell)                                  |

## Windows (.exe)

Le projet fournit des scripts PowerShell pour Windows 11.

### Prérequis

Installer PowerShell 7, Rust, MSYS2, la cible GNU Windows et les bibliothèques GTK 4 et Libadwaita :

```powershell
pwsh -ExecutionPolicy Bypass -File .\scripts\install-deps-windows.ps1
```

Validation Windows versionnée recommandée avant packaging :

```powershell
pwsh -ExecutionPolicy Bypass -File .\scripts\pre-commit-checks-windows.ps1
```

### Créer l’exécutable

```powershell
pwsh -ExecutionPolicy Bypass -File .\scripts\build-exe.ps1 -IncludeGtkRuntime
```

### Créer l’installateur

```powershell
winget install JRSoftware.InnoSetup
pwsh -ExecutionPolicy Bypass -File .\scripts\build-installer.ps1 -IncludeGtkRuntime
```

### Fichiers produits

- `dist/windows/SerialTerm/serial-term.exe`
- `dist/windows/serial-term-win64-release.zip`
- `dist/windows/installer/serial-term-setup-win64-v<version>.exe`

L’option `-IncludeGtkRuntime` copie les bibliothèques GTK depuis `C:\msys64\mingw64\bin`.
Sans cette option, les bibliothèques GTK doivent être installées sur la machine cible.

## Règle de publication

Exécutez les contrôles Linux (`bash scripts/pre-commit-checks.sh`) ou les contrôles Windows (`pwsh -ExecutionPolicy Bypass -File .\scripts\pre-commit-checks-windows.ps1`) avant de créer un paquet.
Conservez les résultats des contrôles avec la version publiée.
