EvoFarm

Application Rust desktop pour comparer la rentabilite de sessions de farm.

Lancement en developpement:
1. Ouvrir le dossier du projet.
2. Executer `cargo run`.

Build release + regeneration des artefacts de distribution:
1. Executer `powershell -ExecutionPolicy Bypass -File .\scripts\build-release.ps1`.
2. Recuperer `EvoFarm.exe` et `evofarm.zip` a la racine du projet.

Fichiers principaux:
- `src/main.rs` : point d'entree, chargement des assets et integration eframe.
- `src/ui.rs` : interface graphique, branding visible et flux de chargement/import.
- `build.rs` : generation de l'icone Windows du `.exe`.
- `scripts/build-release.ps1` : build release puis regeneration du `.exe` et du `.zip`.
