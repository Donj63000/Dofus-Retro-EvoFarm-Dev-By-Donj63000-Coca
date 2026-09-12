Dofus Retro EvoFarm [Dev By Donj63000(Coca)]
==========================================

EvoFarm est un compagnon de farm pour tous les joueurs de Dofus Retro.
Cette application de bureau en Rust permet d'enregistrer ses résultats réels,
de calculer les gains nets et de comparer la rentabilité de ses activités.

L'interface reprend un univers bleu nuit, argent et cyan. Le logo et le fond
fournis sont intégrés dans l'exécutable, avec les portraits des douze classes.
Aucun téléchargement d'image n'est nécessaire au lancement.

Activités et fonctionnalités
---------------------------

- Zones : durée de session, gains totaux et kamas par heure.
- Donjons : durée, gains bruts, prix de la clef et bénéfice net par run.
- Duo / Trio : loot, clefs, pierre de capture et revente de la capture pleine.
- PL arène : durée de ronde, places vendues, prix et nombre de captures.
- Saisie par classe, édition et suppression des entrées, recherches par nom.
- Bilans sur 24 h, 7 jours, 30 jours ou l'historique complet.
- Indicateurs, classements et graphiques des sessions, activités et classes.
- Recommandations d'activités selon la classe et le temps disponible.
- Import JSON, rechargement des données locales et conservation des brouillons.
- Sauvegardes nommées : création, chargement, renommage et suppression.

Utilisation
-----------

Lancer EvoFarm.exe. Aucun installateur ni service en arrière-plan n'est requis.
La distribution est destinée à Windows 64 bits.
Téléchargements officiels :
https://github.com/Donj63000/Dofus-Retro-EvoFarm-Dev-By-Donj63000-Coca/releases/latest

Le logiciel fonctionne localement, sans synchronisation cloud ni appel HTTP.
Un fichier externe est lu uniquement après sélection via la fonction d'import.

Le logo sert aussi d'icône de fenêtre, de barre des tâches et d'exécutable.
Pour l'ajouter au menu Démarrer, créer ou épingler un raccourci vers EvoFarm.exe.
Windows peut conserver l'image d'un ancien raccourci dans son cache ; recréer
ce raccourci permet d'utiliser l'icône du nouvel exécutable.

Sauvegardes et reprise des anciennes données
-----------------------------------------

Les données sont enregistrées au format JSON. Sous Windows :

  %LOCALAPPDATA%\EvoFarm\data.json
  %LOCALAPPDATA%\EvoFarm\data.json.bak
  %LOCALAPPDATA%\EvoFarm\saves\

Le dossier saves contient les sauvegardes nommées. Une mise à jour du programme
par remplacement de l'exécutable conserve les données dans ce dossier.

La copie .bak permet de récupérer une sauvegarde précédente si le fichier
principal est invalide. Les anciens emplacements de sauvegarde sont reconnus
automatiquement lorsqu'aucune sauvegarde EvoFarm n'existe. Leurs fichiers sont
conservés : les prochains enregistrements utilisent le nouveau dossier EvoFarm.
Le format de données, les dates, les classes et les calculs sont préservés.

Si le fichier principal et sa copie de secours sont tous deux invalides dans
le dossier prioritaire, une erreur est signalée. Le logiciel ne recharge pas
silencieusement des données plus anciennes issues d'un autre dossier.

L'import JSON remplace les données chargées et met à jour la sauvegarde locale.
La suppression explicite des sauvegardes couvre aussi les anciens emplacements
reconnus pour éviter de récupérer des données supprimées au prochain lancement.

Développement et vérification
---------------------------

Installer Rust et Cargo, puis ouvrir le dossier du projet. Sous Windows,
utiliser la chaîne Rust MSVC et les outils de compilation C++ du SDK Windows.

  cargo run --locked

Pour vérifier le code :

  cargo fmt --all -- --check
  cargo test --locked --all-targets
  cargo clippy --locked --all-targets -- -D warnings
  powershell -ExecutionPolicy Bypass -File .\scripts\test-packaging.ps1
  cargo install cargo-audit --version 0.22.1 --locked
  cargo audit

Les tests des sauvegardes utilisent des emplacements temporaires. Les tests
d'images vérifient le décodage, les proportions et la transparence du logo.
Les ressources ICO contiennent les tailles 16, 24, 32, 48, 64, 128 et 256 px.

Pour vérifier le rendu natif sur un bureau Windows disponible :

  cargo test --locked --release --bin evofarm native_visual_review -- --ignored --test-threads=1 --nocapture

Ce contrôle utilise des données fictives et ignore les saisies utilisateur.
Il génère des captures et un manifeste dans target/qa/visual : six écrans,
deux tailles de fenêtre, trois facteurs DPI et des vues complètes des tableaux.
La fenêtre de contrôle se ferme automatiquement, sans charger ni modifier
les sauvegardes personnelles. Ce test visuel est exclu du lancement ordinaire
des tests pour permettre leur exécution sans bureau graphique.

Distribution Windows
-------------------

Fermer EvoFarm, puis exécuter :

  powershell -ExecutionPolicy Bypass -File .\scripts\build-release.ps1

Le script vérifie la compilation release avec Cargo.lock, puis produit :

  EvoFarm.exe
  EvoFarm-Windows-x64.zip
  SHA256SUMS

L'archive portable contient l'exécutable et ce mode d'emploi. Les images sont
intégrées au programme. Les sources et les tests sont disponibles sur GitHub.
Les sauvegardes personnelles, caches et captures de contrôle sont exclus.
SHA256SUMS contient les empreintes SHA-256 de l'exécutable et de l'archive.
Une erreur de compilation, de signature ou d'archivage interrompt le script.

Les distributions GitHub ne sont pas signées avec un certificat Authenticode.
Pour signer une compilation locale, configurer ensemble SIGNTOOL_EXE,
SIGN_CERT_PATH et SIGN_CERT_PASSWORD. La configuration partielle est refusée.

Structure du projet
-------------------

- src/main.rs : lancement, état global et coordination des sauvegardes.
- src/ui.rs : formulaires, listes, bilans et recherche d'activité.
- src/theme.rs : identité visuelle et composants partagés.
- src/calculations.rs : calculs, parsing, validation et formatage.
- src/reports.rs : agrégations, recommandations et séries de graphiques.
- src/storage.rs : lecture, écriture atomique et reprise des sauvegardes.
- src/icon_asset.rs : redimensionnement du logo sans déformation.
- build_support/windows_icon.rs : génération de l'ICO multirésolution.
- build.rs : intégration de l'icône et des métadonnées Windows.
- scripts/build-release.ps1 : compilation et distribution.

Remerciements et crédits
-----------------------

Remerciements spéciaux à Clody.

Clody doit être crédité pour une partie du travail réalisé sur ce logiciel,
pour une partie des idées qui l'ont fait avancer et pour une partie de la
manière dont le projet a été pensé.

EvoFarm n'est pas l'œuvre d'un seul auteur. Ce logiciel n'a pas été conçu ni
porté par une seule personne ; cette contribution fait pleinement partie de
son histoire et de sa conception.
