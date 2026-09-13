<p align="center">
  <img src="logo.png" alt="Logo EvoFarm" width="180">
</p>

<h1 align="center">Dofus Retro EvoFarm</h1>
<p align="center"><strong>Dev By Donj63000(Coca)</strong><br>Votre compagnon de farm pour Dofus Retro.</p>

<p align="center">
  <a href="https://github.com/Donj63000/Dofus-Retro-EvoFarm-Dev-By-Donj63000-Coca/releases/latest/download/EvoFarm.exe"><strong>Windows</strong></a>
  ·
  <a href="https://github.com/Donj63000/Dofus-Retro-EvoFarm-Dev-By-Donj63000-Coca/releases/latest/download/EvoFarm-macOS-universal.zip"><strong>macOS</strong></a>
  ·
  <a href="https://github.com/Donj63000/Dofus-Retro-EvoFarm-Dev-By-Donj63000-Coca/releases/latest/download/EvoFarm-Linux-x64.tar.gz"><strong>Linux</strong></a>
  ·
  <a href="https://github.com/Donj63000/Dofus-Retro-EvoFarm-Dev-By-Donj63000-Coca/releases">Toutes les versions</a>
</p>

EvoFarm transforme vos résultats de jeu en bilans clairs : enregistrez vos sessions, calculez vos bénéfices nets et comparez les activités selon votre classe et votre temps disponible. L'application s'adresse à **tous les joueurs de Dofus Retro**.

Son interface bleu nuit, cyan et argent réunit les portraits des douze classes, des formulaires lisibles et des graphiques. Le logo, le fond et les icônes sont intégrés au programme.

**Un mois avec EvoFarm : 60 sessions, 20 jours d'activité et trois classes.** Cette démonstration suit un joueur du 14 août au 12 septembre 2026 : zones, donjons, captures en duo/trio et PL arène. Les formulaires remplis, classements, filtres et sauvegardes montrent comment exploiter son historique au quotidien.

Les données et prix sont **fictifs**. Les **11 107 000 kamas** affichés regroupent la valeur estimée des ressources des zones et les bénéfices après coûts des autres activités.

**[Les explications détaillées de la démo](docs/demo/README.md)** · **[Télécharger le mois fictif en JSON](https://raw.githubusercontent.com/Donj63000/Dofus-Retro-EvoFarm-Dev-By-Donj63000-Coca/master/docs/demo/mois-demo.json)** · **[Comment importer la démo](docs/demo/README.md#importer-la-demo)**

## EvoFarm en images

Les **12 captures ci-dessous** montrent les principales situations du quotidien, avec le même historique fictif. Cliquez sur une image pour l'afficher en grand.

### 1. Voir sa progression sur le mois

Le bilan sur 30 jours réunit les 60 sessions, les indicateurs et les courbes cumulées des quatre activités. Le joueur voit comment ses sorties contribuent à son résultat mensuel.

![EvoFarm après un mois fictif : 60 sessions et courbes de progression par activité](docs/demo/images/01-bilan-mensuel.png)

### 2. Examiner sa dernière semaine

Le filtre **7 jours** et le **Mode bâton** mettent en évidence les durées, les bonnes sessions et les pertes : ici, 12 sessions représentent 2 330 500 kamas de valeur suivie.

![Bilan hebdomadaire : durées, gains et session déficitaire](docs/demo/images/02-bilan-hebdomadaire.png)

### 3. Comparer ses activités et ses classes

Les meilleures activités, les résultats par classe et les sessions récentes permettent de comprendre d'où viennent les gains et de retrouver les sorties qui les expliquent.

![Détail des bilans : meilleures activités, classes et sessions récentes](docs/demo/images/03-activites-classes-historique.png)

### 4. Choisir quoi farmer avec une heure disponible

Le joueur sélectionne son **Crâ** et une durée de **01:00:00**. EvoFarm utilise son historique pour proposer des activités, leur durée moyenne et une estimation des gains.

![Recherche d'activité pour un Crâ disposant d'une heure](docs/demo/images/04-recherche-activite.png)

### 5. Enregistrer une session en zone

La durée et la valeur estimée des ressources donnent immédiatement un rendement. Le formulaire rempli côtoie les 24 sessions de zones, classées par kamas par heure.

![Zones : formulaire rempli, prévision immédiate et historique](docs/demo/images/05-zones.png)

### 6. Connaître le bénéfice d'un donjon

EvoFarm déduit le prix de la clef du gain brut : **183 000 kamas − 14 000 kamas = 169 000 kamas nets**. L'historique permet ensuite de comparer les runs enregistrés.

![Donjons : coût de la clef, bénéfice net et classement des runs](docs/demo/images/06-donjons.png)

### 7. Calculer un run en duo avec capture

Le loot et la revente de la capture sont confrontés au coût des **deux clefs** et de la pierre. Ce Dragon Cochon donne une prévision de **245 500 kamas nets** pour le run.

![Duo : loot, deux clefs, pierre de capture et bénéfice net](docs/demo/images/07-duo.png)

### 8. Passer au trio

Le mode **Trio** prend en compte trois clefs. La prévision du Blop Multicolore Royal atteint **254 000 kamas nets** ; les runs correspondants sont visibles dans l'historique filtré. Le résultat concerne le run entier, sans partage automatique entre joueurs.

![Trio : trois clefs, prévision de rendement et historique filtré](docs/demo/images/08-trio.png)

### 9. Suivre les recettes et les pertes du PL arène

Les places vendues produisent les recettes, puis EvoFarm retire le coût des captures. La liste montre aussi une session à **−41 000 kamas**, conservée dans les bilans pour refléter les résultats réels de la saisie.

![PL arène : places vendues, coût des captures et session déficitaire](docs/demo/images/09-pl-arene.png)

### 10. Corriger une session existante

Le bouton **Modifier** retrouve les champs de la session. Une correction de 278 000 à 293 000 kamas actualise aussitôt la prévision ; le joueur peut **Enregistrer** ou **Annuler**. Cette modification reste en cours dans la capture et ne change pas les totaux de la démo.

![Modification d'une session : champs remplis, prévision et commandes de validation](docs/demo/images/10-modifier-session.png)

### 11. Sauvegarder son historique et ses brouillons

Une sauvegarde nommée conserve les **60 sessions et les quatre brouillons**. Le dialogue résume le contenu avant de garder cet état du mois.

![Sauvegarde nommée : nom et résumé de l'historique avec ses brouillons](docs/demo/images/11-sauvegarder.png)

### 12. Retrouver ses sauvegardes du mois

La bibliothèque présente les instantanés de l'historique avec leur date et leur contenu. Le joueur peut **Charger**, **Renommer** ou **Supprimer** une sauvegarde. La galerie illustre quatre instantanés hebdomadaires ; le JSON téléchargeable contient l'état final.

![Bibliothèque des sauvegardes : historique du mois et actions disponibles](docs/demo/images/12-charger.png)

## Télécharger et commencer

| Système | Téléchargement | Démarrage |
| --- | --- | --- |
| **Windows x64** | [EvoFarm.exe](https://github.com/Donj63000/Dofus-Retro-EvoFarm-Dev-By-Donj63000-Coca/releases/latest/download/EvoFarm.exe) ou [ZIP portable](https://github.com/Donj63000/Dofus-Retro-EvoFarm-Dev-By-Donj63000-Coca/releases/latest/download/EvoFarm-Windows-x64.zip) | Lancez `EvoFarm.exe`. |
| **macOS Intel et Apple Silicon** | [ZIP universel](https://github.com/Donj63000/Dofus-Retro-EvoFarm-Dev-By-Donj63000-Coca/releases/latest/download/EvoFarm-macOS-universal.zip) | Extrayez le ZIP, placez `EvoFarm.app` dans Applications et ouvrez-la. |
| **Linux x64** | [Archive TAR.GZ](https://github.com/Donj63000/Dofus-Retro-EvoFarm-Dev-By-Donj63000-Coca/releases/latest/download/EvoFarm-Linux-x64.tar.gz) | Extrayez l'archive puis lancez `./EvoFarm/evofarm`. |

Aucun compte ou abonnement n'est requis. Sélectionnez une activité, renseignez votre session et consultez vos bilans. Le programme fonctionne localement, sans service ni synchronisation cloud.

### Windows

Le programme est portable et intègre son runtime C. Un raccourci vers `EvoFarm.exe` permet de l'épingler au menu Démarrer ou à la barre des tâches, avec le logo de l'application. Les distributions GitHub ne disposent pas d'une signature Authenticode.

### macOS

Le même fichier `EvoFarm.app` contient les versions **Intel x64 et Apple Silicon ARM64**. Le minimum de déploiement est **macOS 11.0** ; les contrôles natifs de la CI sont exécutés sur macOS 15 pour les deux architectures.

Le bundle possède une signature **ad hoc**, sans certificat Developer ID ni notarisation Apple. Si macOS bloque la première ouverture, après avoir essayé de lancer l'app, accédez à **Réglages Système → Confidentialité et sécurité → Ouvrir quand même**, puis confirmez avec **Ouvrir**. Ce parcours est décrit dans l'[assistance Apple](https://support.apple.com/fr-fr/102445).

### Linux

La distribution est construite sur **Ubuntu 22.04 x64**, avec une base **glibc 2.35**. Elle vise les distributions compatibles avec cette base ou une version ultérieure, disposant d'un bureau X11 ou Wayland et d'OpenGL. Les dialogues d'import utilisent XDG Desktop Portal avec un backend GTK, GNOME ou KDE. La présence d'une glibc compatible ne suffit pas si ces bibliothèques de bureau manquent.

Sur Ubuntu 22.04, les composants d'exécution peuvent être installés ainsi :

```bash
sudo apt install libx11-6 libx11-xcb1 libxcb1 libxkbcommon0 libxkbcommon-x11-0 \
  libgl1 libegl1 libwayland-client0 xdg-desktop-portal xdg-desktop-portal-gtk
tar -xzf EvoFarm-Linux-x64.tar.gz
./EvoFarm/evofarm
```

L'archive contient également `logo.png`, `fr.donj63000.evofarm.desktop` et **`install-desktop.sh`**. L'exécution facultative de `./EvoFarm/install-desktop.sh` ajoute le raccourci et son icône au menu des applications de votre utilisateur, sans droits administrateur. **Conservez ensuite le dossier extrait au même emplacement** : le raccourci référence son exécutable. Pour déplacer le programme, relancez ce script depuis son nouvel emplacement. Rien n'est installé automatiquement à l'extraction ou au lancement d'EvoFarm.

### Vérifier un téléchargement

Depuis la version 0.3.0, le fichier [SHA256SUMS](https://github.com/Donj63000/Dofus-Retro-EvoFarm-Dev-By-Donj63000-Coca/releases/latest/download/SHA256SUMS) contient les empreintes des **quatre téléchargements** : l'exécutable Windows et les trois archives. Comparez la valeur calculée avec la ligne correspondant à votre fichier :

```powershell
Get-FileHash .\EvoFarm.exe -Algorithm SHA256
```

```bash
# Linux
sha256sum EvoFarm-Linux-x64.tar.gz
# macOS
shasum -a 256 EvoFarm-macOS-universal.zip
```

## Suivre et comparer vos activités

| Activité | Ce que vous mesurez |
| --- | --- |
| **Zones** | Durée totale, valeur de la session et kamas par heure. |
| **Donjons** | Gains bruts, coût de la clef et bénéfice net par run. |
| **Duo / Trio** | Loot, clefs, pierre de capture et valeur de revente de la capture pleine. |
| **PL arène** | Places vendues, recettes, coût des captures et bénéfice net. |

- Classe, date et heure associées à chaque session.
- Modification, suppression et recherche des entrées.
- Bilans sur 24 h, 7 jours, 30 jours ou l'historique complet.
- Indicateurs, classements, graphiques en barres et courbes.
- Recherche d'activités selon la classe et le temps disponible.
- Sauvegardes nommées : créer, charger, renommer et supprimer vos jeux de données.
- Import JSON, copie de secours et conservation des brouillons.

## Vos données restent sur votre ordinateur

Le dossier de données dépend du système :

| Système | Dossier |
| --- | --- |
| Windows | `%LOCALAPPDATA%\EvoFarm\` |
| macOS | `~/Library/Application Support/EvoFarm/` |
| Linux | `$XDG_DATA_HOME/EvoFarm/`, ou `~/.local/share/EvoFarm/` si cette variable est absente |

Son contenu est le même sur les trois plateformes :

```text
data.json      Données et brouillons courants
data.json.bak  Copie de secours
saves/         Sauvegardes nommées
```

Fermez l'application puis remplacez le programme, le bundle ou le dossier portable pour mettre à jour EvoFarm. Les données sont conservées dans leur dossier séparé.

Les anciens emplacements sont reconnus automatiquement lorsqu'aucune sauvegarde EvoFarm n'existe. Les anciens fichiers restent conservés ; les nouveaux enregistrements utilisent le dossier EvoFarm. Si un fichier principal est invalide, sa copie de secours est essayée. Une erreur est signalée si les deux fichiers prioritaires sont illisibles.

**Importer un JSON remplace les données actuellement chargées.** Créez une sauvegarde nommée si vous souhaitez conserver plusieurs historiques. Pour transférer vos données entre ordinateurs ou systèmes, copiez manuellement `data.json` depuis le dossier de données puis importez cette copie sur l'autre ordinateur. Aucune synchronisation automatique entre appareils n'est fournie.

## Compiler et vérifier le projet

La CI utilise Rust **1.93.0**. Installez **Python 3.11 ou ultérieur** pour les scripts de packaging Unix et leurs tests. Les commandes suivantes vérifient le code sur le système courant :

```text
cargo run --locked
cargo fmt --all -- --check
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo install cargo-audit --version 0.22.1 --locked
cargo audit
```

Tests Python : `python3 -m unittest discover -s scripts -p 'test_*.py'`. Sous Windows, utilisez `py -3.13` ou votre interpréteur Python 3.11+ à la place de `python3`.

### Distribution Windows

Prérequis : cible Rust `x86_64-pc-windows-msvc`, outils C++ de Visual Studio et SDK Windows.

```powershell
rustup target add x86_64-pc-windows-msvc
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-packaging.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
```

Les 21 scénarios PowerShell vérifient le packaging Windows. Le script de construction compile en release avec `Cargo.lock`, puis produit à la racine **EvoFarm.exe**, **EvoFarm-Windows-x64.zip** et **SHA256SUMS**. Ce manifeste local couvre seulement les deux produits Windows. Le ZIP contient le programme et son mode d'emploi.

La signature locale est facultative : les trois variables `SIGNTOOL_EXE`, `SIGN_CERT_PATH` et `SIGN_CERT_PASSWORD` doivent être configurées ensemble. Une erreur de compilation, de signature ou d'archivage interrompt le script.

### Distribution Linux

La construction s'effectue sur Linux x64. Pour une base comparable à la release, utilisez Ubuntu 22.04 avec les outils de compilation et bibliothèques graphiques :

```bash
sudo apt install build-essential pkg-config libx11-dev libx11-xcb-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev \
  libxkbcommon-x11-0 libgl1-mesa-dev libegl1-mesa-dev libwayland-dev
rustup target add x86_64-unknown-linux-gnu
python3 scripts/build-unix-release.py --platform linux
```

Le script produit **dist/EvoFarm-Linux-x64.tar.gz** et **dist/SHA256SUMS-linux**.

### Distribution macOS universelle

La construction s'effectue sur un Mac avec les outils en ligne de commande de Xcode (`xcode-select --install`), Python 3.11+ et les deux cibles Rust :

```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin
python3 scripts/build-unix-release.py --platform macos
```

Le script compile les deux architectures, les réunit avec `lipo`, génère l'icône ICNS à partir de `logo.png`, puis signe et vérifie le bundle. Il produit **dist/EvoFarm-macOS-universal.zip** et **dist/SHA256SUMS-macos**. `MACOSX_DEPLOYMENT_TARGET` vaut `11.0` par défaut ; le relever limite les systèmes pouvant lancer votre compilation locale.

Les scripts de construction respectent `CARGO_TARGET_DIR`, relatif au projet ou absolu. Le script Unix accepte également `--output-dir`. Les archives contiennent uniquement les fichiers nécessaires à la distribution ; les sources, données personnelles, caches et captures de contrôle en sont exclus.

### Contrôle visuel Windows

Sur un bureau Windows disponible, un test complémentaire génère des captures des écrans à plusieurs tailles et facteurs DPI, avec des données fictives :

```powershell
cargo test --locked --release --bin evofarm native_visual_review -- --ignored --test-threads=1 --nocapture
```

Les captures et leur manifeste sont enregistrés dans `target/qa/visual`. Ce contrôle ne charge ni ne modifie les sauvegardes personnelles.

## Publication

La [CI multiplateforme](https://github.com/Donj63000/Dofus-Retro-EvoFarm-Dev-By-Donj63000-Coca/actions/workflows/ci.yml) utilise **Windows 2022 x64, Ubuntu 22.04 x64, macOS 15 ARM64 et macOS 15 Intel**. Elle vérifie le format, Clippy, les tests Rust, les scripts de packaging et les dépendances. Des contrôles de démarrage complètent la validation des programmes Linux et macOS. Chaque architecture macOS exécute ses tests nativement.

Un tag `vX.Y.Z` correspondant à la version de `Cargo.toml` lance cette validation avant publication. La GitHub Release réunit **EvoFarm.exe**, **EvoFarm-Windows-x64.zip**, **EvoFarm-Linux-x64.tar.gz**, **EvoFarm-macOS-universal.zip** et un **SHA256SUMS global de quatre lignes**. Les sources restent disponibles dans le dépôt et les archives source de GitHub ; les binaires ne sont pas stockés dans l'historique courant des sources.

## Crédits

**Dofus Retro EvoFarm [Dev By Donj63000(Coca)]**

Remerciements spéciaux à **Clody**, pour une partie du travail réalisé sur ce logiciel, des idées qui l'ont fait avancer et de la manière dont le projet a été pensé.

EvoFarm n'est pas l'œuvre d'un seul auteur : cette contribution fait pleinement partie de son histoire et de sa conception.
