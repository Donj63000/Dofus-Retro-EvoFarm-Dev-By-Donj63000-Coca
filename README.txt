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

Téléchargements officiels :
https://github.com/Donj63000/Dofus-Retro-EvoFarm-Dev-By-Donj63000-Coca/releases/latest

Windows x64 : lancer EvoFarm.exe, disponible seul ou dans le ZIP portable.
macOS Intel et Apple Silicon : extraire le ZIP universel, déplacer EvoFarm.app
dans Applications et l'ouvrir. Le même bundle contient les deux architectures.
Linux x64 : extraire l'archive TAR.GZ puis lancer ./EvoFarm/evofarm.

Aucun compte, abonnement ou service en arrière-plan n'est requis.
Le logiciel fonctionne localement, sans synchronisation cloud. Sélectionner
une activité, renseigner une session puis consulter les bilans.

Windows
-------

Le logo sert aussi d'icône de fenêtre, de barre des tâches et d'exécutable.
Pour l'ajouter au menu Démarrer, créer ou épingler un raccourci vers EvoFarm.exe.
Windows peut conserver l'image d'un ancien raccourci dans son cache ; recréer
ce raccourci permet d'utiliser l'icône du nouvel exécutable.

Le programme intègre son runtime C. Les distributions GitHub ne disposent pas
d'une signature Authenticode.

macOS
-----

Le minimum de déploiement est macOS 11.0. La CI contrôle nativement les versions
Intel x64 et Apple Silicon ARM64 sur macOS 15. Le logo est intégré au bundle
pour le Finder, le Dock et la fenêtre.

L'application possède une signature ad hoc, sans certificat Developer ID ni
notarisation Apple. Si macOS bloque la première ouverture, essayer de lancer
l'application puis ouvrir Réglages Système > Confidentialité et sécurité >
Ouvrir quand même, et confirmer avec Ouvrir.
Parcours officiel et explications Apple :
https://support.apple.com/fr-fr/102445

Linux
-----

La distribution est construite sur Ubuntu 22.04 x64 avec une base glibc 2.35.
Elle vise les systèmes compatibles avec cette base ou une version ultérieure,
avec un bureau X11 ou Wayland, OpenGL et les bibliothèques graphiques requises.
Les dialogues d'import utilisent XDG Desktop Portal avec un backend GTK,
GNOME ou KDE. Une glibc compatible ne suffit pas si ces composants manquent.

Composants d'exécution pour Ubuntu 22.04 :

  sudo apt install libx11-6 libx11-xcb1 libxcb1 libxkbcommon0 libxkbcommon-x11-0 libgl1 libegl1 libwayland-client0 xdg-desktop-portal xdg-desktop-portal-gtk

Extraction et lancement :

  tar -xzf EvoFarm-Linux-x64.tar.gz
  ./EvoFarm/evofarm

Le dossier EvoFarm contient le programme, ce mode d'emploi, logo.png,
fr.donj63000.evofarm.desktop et install-desktop.sh.

Pour ajouter facultativement une entrée au menu des applications et son icône,
sans droits administrateur :

  ./EvoFarm/install-desktop.sh

Le script installe le raccourci et l'icône pour l'utilisateur courant dans
XDG_DATA_HOME, ou ~/.local/share. Il référence le programme dans son dossier
extrait : conserver ce dossier au même emplacement. En cas de déplacement,
relancer le script depuis le nouvel emplacement. Rien n'est installé
automatiquement lors de l'extraction ou du lancement d'EvoFarm.

Vérifier un téléchargement
-------------------------

Le fichier SHA256SUMS de la release contient quatre lignes : les empreintes
de l'exécutable Windows et des trois archives Windows, Linux et macOS.
Comparer l'empreinte calculée avec la ligne correspondant au fichier choisi :

  Windows PowerShell : Get-FileHash .\EvoFarm.exe -Algorithm SHA256
  Linux : sha256sum EvoFarm-Linux-x64.tar.gz
  macOS : shasum -a 256 EvoFarm-macOS-universal.zip

Sauvegardes et reprise des anciennes données
-----------------------------------------

Les données sont enregistrées au format JSON dans le dossier du système :

  Windows : %LOCALAPPDATA%\EvoFarm\
  macOS : ~/Library/Application Support/EvoFarm/
  Linux : $XDG_DATA_HOME/EvoFarm/, ou ~/.local/share/EvoFarm/

Chaque dossier contient :

  data.json      Données et brouillons courants
  data.json.bak  Copie de secours
  saves/         Sauvegardes nommées

Pour mettre à jour, fermer l'application puis remplacer l'exécutable, le bundle
ou le dossier portable. Les données restent dans leur dossier séparé.

La copie .bak permet de récupérer une sauvegarde précédente si le fichier
principal est invalide. Les anciens emplacements de sauvegarde sont reconnus
automatiquement lorsqu'aucune sauvegarde EvoFarm n'existe. Leurs fichiers sont
conservés : les prochains enregistrements utilisent le nouveau dossier EvoFarm.
Le format de données, les dates, les classes et les calculs sont préservés.

Si le fichier principal et sa copie de secours sont tous deux invalides dans
le dossier prioritaire, une erreur est signalée. Le logiciel ne recharge pas
silencieusement des données plus anciennes issues d'un autre dossier.

L'import JSON remplace les données chargées et met à jour la sauvegarde locale.
Créer une sauvegarde nommée avant un import pour conserver l'historique courant.
Pour changer d'ordinateur ou de système, copier manuellement data.json depuis
le dossier de données puis importer cette copie sur l'autre ordinateur.
Il n'existe aucune synchronisation automatique entre appareils.

La suppression explicite des sauvegardes couvre aussi les anciens emplacements
reconnus pour éviter de récupérer des données supprimées au prochain lancement.

Développement et vérification
---------------------------

Installer Rust et Cargo, puis ouvrir le dossier du projet. La CI utilise Rust
1.93.0. Les scripts Unix et leurs tests nécessitent Python 3.11 ou ultérieur.

  cargo run --locked

Pour vérifier le code :

  cargo fmt --all -- --check
  cargo test --locked --all-targets
  cargo clippy --locked --all-targets -- -D warnings
  cargo install cargo-audit --version 0.22.1 --locked
  cargo audit
  python3 -m unittest discover -s scripts -p 'test_*.py'

Sous Windows, remplacer python3 par py -3.13 ou un interpréteur Python 3.11+.
Les tests du packaging Windows se lancent séparément :

  powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-packaging.ps1

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

Prérequis : outils C++ de Visual Studio, SDK Windows et cible Rust MSVC.
Fermer EvoFarm puis exécuter depuis le projet :

  rustup target add x86_64-pc-windows-msvc
  powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-release.ps1

Le script vérifie la compilation release avec Cargo.lock, puis produit :

  EvoFarm.exe
  EvoFarm-Windows-x64.zip
  SHA256SUMS

L'archive portable contient l'exécutable et ce mode d'emploi. Les images sont
intégrées au programme. Les sources et les tests sont disponibles sur GitHub.
Les sauvegardes personnelles, caches et captures de contrôle sont exclus.
Ce SHA256SUMS local contient seulement les deux empreintes Windows ; le
manifeste global publié sur GitHub couvre les quatre téléchargements.
Une erreur de compilation, de signature ou d'archivage interrompt le script.

Pour signer une compilation locale, configurer ensemble SIGNTOOL_EXE,
SIGN_CERT_PATH et SIGN_CERT_PASSWORD. La configuration partielle est refusée.

Distribution Linux
------------------

Compiler sur Linux x64. Pour une base comparable à la release, utiliser
Ubuntu 22.04 avec les outils de développement graphiques :

  sudo apt install build-essential pkg-config libx11-dev libx11-xcb-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libxkbcommon-x11-0 libgl1-mesa-dev libegl1-mesa-dev libwayland-dev
  rustup target add x86_64-unknown-linux-gnu
  python3 scripts/build-unix-release.py --platform linux

Produits : dist/EvoFarm-Linux-x64.tar.gz et dist/SHA256SUMS-linux.

Distribution macOS universelle
-----------------------------

Compiler sur un Mac avec Python 3.11+, les outils en ligne de commande Xcode
(xcode-select --install) et les deux cibles Rust :

  rustup target add x86_64-apple-darwin aarch64-apple-darwin
  python3 scripts/build-unix-release.py --platform macos

Le script compile les deux architectures et les réunit avec lipo, crée l'icône
ICNS à partir de logo.png, puis signe et vérifie le bundle.
Produits : dist/EvoFarm-macOS-universal.zip et dist/SHA256SUMS-macos.
MACOSX_DEPLOYMENT_TARGET vaut 11.0 par défaut. Le relever limite les systèmes
pouvant utiliser la compilation locale.

Les scripts de construction respectent CARGO_TARGET_DIR, relatif au projet ou
absolu. Le script Unix accepte --output-dir pour choisir le dossier de sortie.

Publication GitHub
------------------

La CI vérifie le code et les tests sur Windows 2022 x64, Ubuntu 22.04 x64,
macOS 15 ARM64 et macOS 15 Intel. Elle contrôle aussi le packaging et les
dépendances ; les programmes Linux et macOS passent un test de démarrage.
Les deux architectures macOS exécutent leurs tests nativement.

Un tag vX.Y.Z correspondant à Cargo.toml lance cette validation avant
publication de cinq fichiers dans la release :

  EvoFarm.exe
  EvoFarm-Windows-x64.zip
  EvoFarm-Linux-x64.tar.gz
  EvoFarm-macOS-universal.zip
  SHA256SUMS

Les binaires sont distribués dans les releases. Les sources restent disponibles
dans le dépôt et dans les archives source générées par GitHub.

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
- scripts/build-release.ps1 : compilation et distribution Windows.
- scripts/build-unix-release.py : archives Linux et bundle macOS universel.

Remerciements et crédits
-----------------------

Remerciements spéciaux à Clody.

Clody doit être crédité pour une partie du travail réalisé sur ce logiciel,
pour une partie des idées qui l'ont fait avancer et pour une partie de la
manière dont le projet a été pensé.

EvoFarm n'est pas l'œuvre d'un seul auteur. Ce logiciel n'a pas été conçu ni
porté par une seule personne ; cette contribution fait pleinement partie de
son histoire et de sa conception.
