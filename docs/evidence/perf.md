# Performance (PERF)

> Objectif complémentaire D du suivi Epitech : passer de « ça marche » à « ça tient, et voici de combien ». Choisir un axe, mesurer le point de départ, diagnostiquer, changer une chose précise, remesurer, lire l'écart. Une ligne par mesure officielle, y compris les changements qui n'ont rien gagné.

## Note de choix (brouillon, à finaliser avant fin décembre)

**Axe : temps de réponse.** Un pie menu n'a de valeur que s'il apparaît tout de suite. S'il met une ou deux secondes, l'utilisateur va plus vite à la main et Bloom ne sert à rien. C'est aussi notre problème dur ([`../hard-problem.md`](../hard-problem.md)) : ces mesures servent deux fois.

**La qualité des choix est le garde-fou.** Gagner de la vitesse en proposant de moins bonnes actions n'est pas un gain : `top3` ne doit pas baisser d'une boucle à l'autre.

| Indicateur | Formule | Outil | Conditions |
|---|---|---|---|
| `menu_latency` (principal) | heure d'affichage du menu moins heure d'appui sur le raccourci ; p50 et p95 | `RunState.timings_ms` dans l'audit, script de `lab/bloom/` | Mac de référence, 100 runs dont 5 d'échauffement exclus, scénario fixe |
| `decision_latency` | durée d'un appel à `Ranker.rank()` ; p50 et p95 par backend | `make bench` | même jeu, 1 appel d'échauffement |
| `top3` (garde-fou) | part des situations où une action attendue est dans les 3 premières | `make bench` | jeu d'au moins 50 situations, figé |

Suivi en plus, hors cible : `personalization_lift` (gain du top 1 avec la mémoire), à partir de la v1.0.

**Cible (proposée) :** `menu_latency` p95 ≤ 300 ms en mai, sans que `top3` passe sous 80 %. Ce sur quoi on s'appuie : un menu doit sembler instantané (les ordres de grandeur classiques de l'IHM placent l'instantané autour de 0,1 s et la rupture du fil de pensée autour de 1 s), et les modèles de décision hébergés annoncent 70 à 500 ms par appel. Atteindre la cible demandera sans doute de calculer le classement avant l'appui ou de passer en local : c'est une hypothèse à confirmer par le diagnostic, pas une décision.

**Pistes d'optimisation**, à ne choisir qu'après le diagnostic : classement anticipé au changement d'application, backend local plutôt qu'hébergé, cache de l'assemblage du contexte, moins de candidats envoyés au modèle.

## Protocole

Règles et indicateurs : [`lab/README.md`](../../lab/README.md). Mesure de bout en bout : [`lab/bloom/README.md`](../../lab/bloom/README.md).

## Mesures

| Date | Commit | Indicateur | Valeur | Conditions | Fichier de résultats |
|---|---|---|---|---|---|

## Boucles d'optimisation

| Boucle | Hypothèse, écrite avant | Gain attendu | Changement (commit) | Avant | Après | Écart (absolu et %) | Hypothèse vérifiée ? | Compromis accepté | Non-régression OK ? |
|---|---|---|---|---|---|---|---|---|---|
