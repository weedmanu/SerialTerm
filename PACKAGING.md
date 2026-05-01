# Packaging

Index des guides de packaging de SerialTerm.

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

Installer PowerShell 7, Rust, MSYS2, la cible GNU Windows et la toolchain GTK4/Libadwaita :

```powershell
pwsh -ExecutionPolicy Bypass -File .\scripts\install-deps-windows.ps1
```

Validation Windows versionnée recommandée avant packaging :

```powershell
pwsh -ExecutionPolicy Bypass -File .\scripts\pre-commit-checks-windows.ps1
```

### Build EXE

```powershell
pwsh -ExecutionPolicy Bypass -File .\scripts\build-exe.ps1 -IncludeGtkRuntime
```

### Build installateur

```powershell
winget install JRSoftware.InnoSetup
pwsh -ExecutionPolicy Bypass -File .\scripts\build-installer.ps1 -IncludeGtkRuntime
```

### Artefacts générés

- `dist/windows/SerialTerm/serial-term.exe`
- `dist/windows/serial-term-win64-release.zip`
- `dist/windows/installer/serial-term-setup-win64-v<version>.exe`

Le flag `-IncludeGtkRuntime` copie les DLL GTK depuis `C:\msys64\mingw64\bin`.
Sans ce flag, l'exécutable dépend d'un runtime GTK installé sur la machine cible.

## Règle de publication

Valider localement avec le gate standard Linux (`bash scripts/pre-commit-checks.sh`) ou le gate Windows documenté (`pwsh -ExecutionPolicy Bypass -File .\scripts\pre-commit-checks-windows.ps1`) avant tout packaging.
Ne pas produire d'artefact depuis un état non validé.
