EvoFarm
=======

EvoFarm est une application desktop Rust dediee au suivi et a la comparaison de la rentabilite de sessions de farm sur DOFUS.

Le logiciel sert a enregistrer des donnees reelles de jeu, recalculer automatiquement les gains utiles, puis comparer rapidement les activites les plus interessantes selon votre classe, votre temps disponible et votre historique.

Ce que fait le logiciel
-----------------------

- Enregistre plusieurs types d'activites de farm dans une interface unique.
- Recalcule automatiquement les valeurs derivees pour eviter les erreurs de saisie.
- Trie les activites par rentabilite pour voir tout de suite ce qui rapporte le plus.
- Permet de modifier, supprimer, nettoyer et recharger les donnees en gardant une sauvegarde locale.
- Produit des bilans lisibles avec indicateurs, classements et graphiques.
- Aide a choisir une activite en fonction de la classe jouee et du temps disponible.

Activites gerees
----------------

1. Zones
   Enregistrement d'une session complete avec duree totale, valeur totale de session et calcul des kamas par heure.
2. Donjons
   Calcul du net par run a partir du temps du donjon, du gain brut moyen et du prix de la cle.
3. Duo / Trio
   Prise en compte du mode duo ou trio, du loot, du prix de la pierre de capture, du prix unitaire des clefs et de la valeur de revente de la capture pleine.
4. PL arene
   Calcul du benefice net a partir du temps de ronde, du prix d'une place, du nombre de places vendues, du prix d'une capture et du nombre de captures utilisees.

Fonctionnalites principales
---------------------------

- Saisie par categorie avec formulaires dedies.
- Choix de la classe du personnage pour chaque entree.
- Date et heure d'enregistrement pour alimenter les bilans temporels.
- Calcul automatique des kamas par heure, gains nets, couts et revenus.
- Recherche par nom avec normalisation simple des variantes d'ecriture.
- Edition et suppression d'entrees directement dans les listes.
- Import d'un fichier JSON externe.
- Rechargement de la sauvegarde locale.
- Bilans par periode: 24h, 7 jours, 30 jours ou historique complet.
- Synthese des meilleures sessions, meilleures activites et meilleures classes.
- Recommandations d'activites selon la classe et le temps disponible.

Pourquoi ce logiciel est utile
------------------------------

EvoFarm ne se limite pas a stocker des notes. Il sert a prendre des decisions.

L'idee est de partir de resultats reels observes en jeu, puis de transformer ces sessions en informations comparables:

- quelle zone rapporte vraiment le plus ;
- quel donjon reste rentable une fois le prix de la cle retire ;
- quel run duo / trio devient interessant avec le cout des clefs et de la capture ;
- quel PL arene est rentable ou non apres les depenses.

Les bilans et la recherche d'activite permettent ensuite de repondre rapidement a des questions concretes:

- "Qu'est-ce qui me rapporte le plus sur ma classe ?"
- "Que puis-je lancer si j'ai seulement 30 minutes ou 1 heure ?"
- "Quelle activite est la plus fiable sur mon historique recent ?"

Utilisation du logiciel
-----------------------

Si vous voulez simplement utiliser le logiciel, `EvoFarm.exe` est l'executable directement utilisable.

Si vous voulez modifier le logiciel, adapter son comportement ou travailler sur le code source, le projet peut etre recompile en Rust.

Sauvegarde des donnees
----------------------

Les donnees sont stockees localement au format JSON.

Sous Windows, la sauvegarde locale est ecrite ici:

`%LOCALAPPDATA%\dofus_rentabilite\data.json`

Une sauvegarde de secours peut egalement etre conservee ici:

`%LOCALAPPDATA%\dofus_rentabilite\data.json.bak`

Quand un fichier JSON externe est importe, son contenu remplace les donnees actuellement chargees puis met a jour la sauvegarde locale de l'application.

Lancement en developpement
--------------------------

1. Ouvrir le dossier du projet.
2. Installer Rust et Cargo si necessaire.
3. Executer:

`cargo run`

Build release
-------------

Pour generer l'executable Windows et l'archive de distribution:

1. Executer:

`powershell -ExecutionPolicy Bypass -File .\scripts\build-release.ps1`

2. Recuperer les artefacts generes a la racine du projet:

- `EvoFarm.exe`
- `evofarm.zip`

Structure du projet
-------------------

- `src/main.rs` : demarrage de l'application, etat global, chargement et sauvegarde.
- `src/ui.rs` : interface graphique, formulaires, listes, bilans et recherche d'activite.
- `src/calculations.rs` : calculs de rentabilite, parsing, validation et formatage.
- `src/reports.rs` : agregations, recommandations, series de graphiques et bilans.
- `src/storage.rs` : lecture, ecriture, import JSON et sauvegarde locale atomique.
- `build.rs` : integration de l'icone Windows.
- `scripts/build-release.ps1` : build release et creation de l'archive de distribution.

Remerciements et credits
------------------------

Remerciements speciaux a Clody.

Clody doit etre credite pour une partie du travail realise sur ce logiciel, pour une partie des idees qui l'ont fait avancer, et pour une partie de la maniere dont le projet a ete pense.

EvoFarm n'est donc pas l'oeuvre d'un seul auteur. Ce logiciel n'a pas ete concu ni porte par une seule personne, et cette contribution fait pleinement partie de son histoire et de sa conception.
