# EvoFarm 0.3.0 — correctifs de l'audit de cybersécurité

## Portée et statut

Ce lot est établi sur l'archive `Dofus-Retro-EvoFarm-Dev-By-Donj63000-Coca-master (1)(1).zip`, dont le SHA-256 est `3728edbbf141cdd7c5b1692b27ce90cca188226156a69512bd889641d2023fc9`. Il traite les constats F01 à F05 et l'observation O03 du rapport public noté 37/50. Les changements sont implémentés dans les sources ; leur réception définitive reste subordonnée aux tests Rust, à Clippy et aux essais natifs.

Le patch ne contient pas d'exécutable. Il ne modifie ni le fonctionnement hors ligne de l'application, ni les règles du jeu, ni la version de ses formats : le schéma enregistré reste la version 2. Il n'apporte pas une nouvelle certification ou une nouvelle note d'audit.

## Corrections apportées

### F01 — entrées et calculs extrêmes

Les saisies et imports passent par un contrat commun dans `src/limits.rs`. Les sommes de durée sont vérifiées avant multiplication/addition ; les intervalles de calendrier utilisent les opérations vérifiées de Chrono. Les champs dérivés sont recalculés depuis les entrées autorisées et contrôlés avant sérialisation. Les grands nombres valides ne sont plus affichés comme un entier i64 saturé. Une valeur non finie est refusée à l'enregistrement et n'est pas affichée comme un gain valide.

Budgets retenus : 10 000 sessions par état, nom de 512 octets UTF-8 au maximum, champ de brouillon de 4 096 octets, texte numérique de 128 octets, montant d'entrée de 0 à 1 000 milliards de kamas, quantité de 100 000 au maximum, durée de 1 seconde à 24 h 59 min 59 s, dates de session et de fin entre les années 1970 et 9999. Les caractères de contrôle sont refusés dans les noms. Ces plafonds sont des limites techniques, pas des règles de Dofus.

Les anciens fichiers contenant des données modernes hors de ce contrat sont refusés explicitement et ne sont pas réécrits automatiquement. Les règles existantes de suppression des entrées legacy restent applicables ; le statut de chargement indique leur nombre. Les montants restent au format f32 du projet : ce correctif ne les transforme pas en comptabilité décimale exacte.

### F02 — coût des graphiques

Le mode détaillé utilise un balayage des débuts/fins et une liste de détails partagée par intervalle avec `Arc`, au lieu de recopier les détails pour chaque barre. Il est limité à 128 sessions et 8 192 segments. Au-delà de l'un des budgets, l'affichage bascule sur au plus 256 créneaux temporels, quatre catégories et deux signes : 2 048 segments au maximum. Les gains et pertes sont séparés ; la contribution d'une session est distribuée au prorata de son recouvrement temporel.

L'interface annonce ce mode agrégé. Il représente une répartition temporelle moyenne, pas le moment réel de chaque drop. Les totaux comptables du bilan ne sont pas remplacés par les valeurs du graphique. La précision reste celle des flottants. Les sessions non datées restent dans les totaux applicables mais ne sont pas positionnées artificiellement sur l'axe temporel.

Les barres ne sont pas calculées en mode courbes. Un cache est invalidé par les changements de données, de période, de catégories, de mode et de seconde d'horloge. Les longues infobulles cumulées sont limitées à 16 détails avec indication du nombre restant. L'agrégation coûte O(n + b) après tri ; l'ensemble conserve le coût O(n log n) du tri. Les budgets bornent le mode détaillé, ils ne prétendent pas en faire un algorithme entièrement linéaire.

### F03 — cohérence lecture/écriture et état de l'interface

La limite de 8 Mio est appliquée aux octets JSON effectivement produits, pendant la sérialisation et avant toute écriture ou rotation du secours. Elle couvre les états locaux, exports, sauvegardes nommées et renommages. Un import compact qui dépasse ce budget après indentation ne remplace pas la sauvegarde locale.

Les ajouts, éditions et suppressions automatiques conservent un instantané préalable. Si l'enregistrement échoue, les données et brouillons sont restaurés en mémoire et l'interface affiche une erreur, pas un succès. Les imports restent appliqués en mémoire uniquement après enregistrement réussi. Une erreur de synchronisation après renommage peut laisser un état de disque différent de l'état affiché ; les fichiers de récupération sont alors conservés et le chemin est annoncé. Il ne s'agit pas d'une transaction de base de données multi-processus.

### F04 — accès aux fichiers locaux

Les lectures contrôlent le fichier réellement ouvert, refusent les liens symboliques/points de réanalyse en dernier composant et exigent un fichier ordinaire. Sur Unix, `O_NOFOLLOW` et `O_NONBLOCK` évitent le suivi final des liens et le blocage sur un FIFO. Sur Windows, le point de réanalyse est ouvert puis refusé par contrôle de ses attributs.

Les nouveaux fichiers sont créés exclusivement dans un dossier temporaire réservé, sans `fs::copy` vers un `.bak` prévisible. Les nouvelles créations sont privées sur Unix (dossier 0700, fichier 0600, sous réserve du umask). Le répertoire parent immédiat est contrôlé puis résolu. L'application refuse un répertoire de données utilisateur indisponible au lieu de choisir silencieusement le répertoire courant. Les catalogues de sauvegardes ont des budgets de nombre et de lecture.

Ce durcissement n'isole pas l'application d'un autre processus déjà exécuté sous le même compte, ne garantit pas l'absence de toute course sur les répertoires ancêtres et ne met pas en place des ACL Windows personnalisées. Deux instances ne doivent pas modifier le même état simultanément. Les permissions du profil et du dossier d'export restent importantes.

### F05 — conservation des récupérations

Les copies de remplacement et de retour arrière vivent dans un dossier `.evofarm-recovery-*` unique. Elles ne sont supprimées qu'après finalisation réussie. Un échec d'installation ET de restauration laisse les anciennes et nouvelles versions disponibles, avec leurs chemins dans l'erreur. Un fichier de récupération préexistant n'est jamais supprimé pour libérer son nom.

Un principal corrompu, non UTF-8 ou trop volumineux ne remplace pas un secours valide lors d'une réparation. Un `.json.bak` de sauvegarde nommée dont le principal est absent reste visible dans le catalogue et chargeable.

Après une panne, fermer l'application, copier le dossier de récupération complet ailleurs, puis examiner les fichiers avant toute suppression. `ancienne-sauvegarde.json` contient l'ancien principal ; `nouvelle-sauvegarde.json` contient le nouvel état si son installation n'a pas abouti. `ancien-secours.json` et `nouveau-secours.json` concernent la rotation du secours. Tous ne sont pas nécessairement présents selon la phase de l'échec. Importer la version souhaitée depuis une copie, puis la vérifier dans l'interface.

### O03 — format et identité

Seules les versions absente, 1 et 2 sont acceptées. Une version future, négative, fractionnaire ou tronquée par conversion ne peut plus être interprétée comme la version 2. L'identifiant d'une sauvegarde nommée doit correspondre au nom du fichier, y compris lors d'une lecture de secours.

Les index des éditions en cours sont remappés après suppression d'entrées legacy puis après tri, en suivant la position d'origine, même pour deux lignes identiques. Un brouillon dont la ligne a réellement disparu est écarté comme auparavant. Le patch ne migre pas les entrées vers des UUID persistants.

## Observations partiellement traitées

**O01 — maintenance des dépendances :** ajout d'un audit RustSec hebdomadaire et à la demande, conservation du rapport JSON dans les artefacts CI, propositions de mise à jour Dependabot pour Cargo et les actions. Les codes d'erreur de `cargo audit` ne sont pas masqués. Les avertissements de non-maintenance ne doivent pas être confondus avec des vulnérabilités exploitables ; ils restent à traiter même si l'audit n'échoue pas. Le patch ne migre pas à l'aveugle la pile graphique et ne déclare pas résolus les avis concernant notamment ttf-parser, paste, instant et derivative. Les versions transitives du verrou restent inchangées ; libc, déjà verrouillé, devient une dépendance directe Unix pour les drapeaux d'ouverture sûrs.

**O02 — provenance et signature :** la publication ajoute une attestation GitHub des quatre distributions, avec une action épinglée au commit `977bb373ede98d70efdf65b84cb5f73e068dcc2a`. Les permissions OIDC et d'attestation sont limitées au job de publication. Le contrat existant de cinq fichiers de release reste inchangé. Cette attestation ne remplace ni une signature Authenticode, ni une identité Developer ID/notarisation Apple. Ces opérations demandent des certificats et une configuration de l'éditeur, non fournis par ce patch. Le workflow ne signifie pas qu'une attestation a déjà été produite : il devra réellement s'exécuter dans un dépôt éligible.

## Tests et réception avant distribution

Le lot ajoute **43 tests Rust** : 11 calculs, 21 stockage, 9 rapports et 2 transactions d'interface. Ils couvrent notamment les bornes, les index, les liens Unix, le partage des détails, les 10 000 sessions et les pannes de renommage injectées. Les essais de liens Unix ne remplacent pas des essais Windows de jonctions/points de réanalyse.

La suite Python compte **96 tests**, dont les 81 préexistants et 15 nouveaux. Les nouveaux tests comprennent des oracles mathématiques indépendants et les tests réels du lanceur de validation. Ils ne constituent pas une exécution du moteur Rust.

Dans l'environnement de préparation, les 96 tests Python ont réussi. Rust, Cargo, Clippy et rustfmt n'étaient pas disponibles : ni la compilation, ni les 43 nouveaux tests Rust, ni le formatage rustfmt, ni un audit RustSec complet du verrou n'ont été exécutés. Les sources doivent donc être formatées et validées avant commit/publication, sans contourner les gates existantes.

Sur le poste de développement, avec la chaîne Rust 1.93.0 utilisée par la CI, rustfmt, Clippy et Python 3.11+ installés :

```powershell
cargo install cargo-audit --version 0.22.1 --locked
python scripts/validate-security.py --format
```

L'option `--format` applique rustfmt, puis le script vérifie le formatage, Clippy avec avertissements interdits, les tests Rust, les tests Python et cargo-audit. Il n'annonce pas de réussite globale si une étape manque ou échoue. Les journaux et le résultat sont placés dans `target/security-validation/`. Une étape échouée interrompt la séquence : corriger puis relancer. Relire les avertissements de dépendances même avec un retour global réussi.

Faire ensuite passer les quatre cibles CI existantes et vérifier manuellement sur Windows : ouverture sans réseau, ajout/édition/suppression, conservation d'une saisie après erreur, sauvegarde/rechargement, import refusé sans perte, récupération `.bak`, brouillon après tri et passage courbes/bâtons sur un gros historique. Ne pas diffuser un nouvel exécutable sur la seule base des contrôles Python.

## Réception dans EvoFarm 0.3.1

Le patch fourni a été appliqué sans conflit : les 20 fichiers de l'index Git correspondaient exactement aux empreintes de blobs du patch avant intégration. La version 0.3.1 applique ensuite rustfmt et deux adaptations nécessaires à la validation : la mutabilité de `DirBuilder` est limitée à Unix pour satisfaire Clippy sous Windows ; le message d'échec d'ouverture conserve le préfixe « Impossible de lire » attendu par le test de compatibilité existant. Les contrôles de sécurité ne sont pas désactivés.

Un test Windows supplémentaire crée une véritable jonction dans un dossier temporaire et vérifie le refus de lecture du point de réanalyse, de catalogage et d'écriture via ce dossier, sans modifier sa cible. Les tests Windows et Unix s'exécutent sur leurs systèmes respectifs dans la CI. Le binaire Windows distribué passe également le test de démarrage sur le runner temporaire, comme les binaires Linux et macOS.

La réception locale sous Windows avec Rust 1.93.0 a validé rustfmt, Clippy sans avertissement, les tests Rust et Python, les 21 scénarios de packaging et cargo-audit. Les deux essais natifs `native_visual_review` et `native_demo_review` ont produit respectivement 45 et 12 captures ; les essais utilisent des états isolés du profil personnel. L'audit ne signale aucune vulnérabilité connue ; les quatre avertissements de maintenance restent présents.

Les nombres de tests indiqués plus haut décrivent l'environnement de préparation du patch. Le dépôt a depuis reçu les contrôles du README vidéo ; le poste de développement contient aussi des tests de tournage locaux non inclus dans cette publication. Les résultats de la CI du tag `v0.3.1` font foi pour les sources distribuées. Le workflow de release vérifie les quatre cibles, les archives et les empreintes avant de publier les binaires et leurs attestations.

## Références techniques

- Rust, `std::fs::OpenOptions` et création exclusive : https://doc.rust-lang.org/std/fs/struct.OpenOptions.html
- Rust, précautions liées aux courses de chemins : https://doc.rust-lang.org/std/fs/index.html
- Chrono, opérations de calendrier vérifiées : https://docs.rs/chrono/latest/chrono/struct.NaiveDateTime.html
- GitHub, attestations d'artefacts : https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations/using-artifact-attestations-to-establish-provenance-for-builds
- Action épinglée : https://github.com/actions/attest-build-provenance/blob/977bb373ede98d70efdf65b84cb5f73e068dcc2a/action.yml
