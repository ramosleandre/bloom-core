# Lab : tout ce qui se mesure

Séparé du produit : le lab importe `bloom`, le produit n'importe jamais le lab. Deux familles de mesures, présentes dès le premier jour parce que deux objectifs de la track reposent dessus.

| Dossier | Sert à | Objectif track |
|---|---|---|
| `decision/` | comparer les backends de décision (règles, Jev, Laya, Von…) sur le même jeu et le même protocole | O1 : prototype de décision, état de l'art |
| `bloom/` | mesurer Bloom de bout en bout : latence raccourci → menu, gain de la personnalisation | PERF |
| `explore/` | prototype d'exploration (créé en février) : hypothèse écrite avant, résultat même négatif | O1 |
| `datasets/` | les situations annotées, leur format et leurs règles | O1, PERF |
| `results/` | les sorties : essais dans `local/` (ignoré par git), mesures officielles commitées | toutes |

## Règles

1. Le protocole s'écrit **avant** la mesure : machine, jeu de données, scénario, nombre de répétitions, moment de la mesure.
2. Ce qu'on compare se mesure dans les mêmes conditions : même jeu, même machine, même protocole.
3. On ne règle pas un backend sur le jeu qui sert à l'évaluer. Les règles et les prompts s'écrivent avant de voir le jeu d'évaluation, sinon le score ne mesure plus rien.
4. Une mesure citée dans un livrable est officielle : `make bench out=lab/results/AAAA-MM-JJ_nom`, puis commit. Les essais restent dans `lab/results/local/`.
5. Chaque mesure officielle donne une ligne dans `docs/evidence/perf.md`, y compris quand elle ne montre aucun gain.

## Commandes

```bash
make bench                                     # règles, sur le jeu d'exemple
make bench backends=rules,laya data=lab/datasets/situations.jsonl out=lab/results/2027-01-20_decision
```

Le banc affiche toujours une ligne **hasard** : le score d'un classement aléatoire sur les mêmes situations. Un backend qui ne la bat pas nettement ne sert à rien.

## Indicateurs

| Nom | Formule | Outil | Conditions |
|---|---|---|---|
| `top1`, `top3` | part des situations où une action attendue est 1re, ou dans les 3 premières | `make bench` | jeu annoté, mêmes actions disponibles |
| `decision_latency` | durée d'un appel à `Ranker.rank()`, p50 et p95 | `make bench` | après 1 appel d'échauffement non mesuré |
| `menu_latency` | heure d'affichage du menu moins heure de l'appui sur le raccourci, p50 et p95 | `RunState.timings_ms` dans l'audit | Mac de référence, protocole de `bloom/README.md` |
| `personalization_lift` | `top1` avec mémoire moins `top1` sans mémoire | `make bench` (v1.0) | situations avec historique utilisateur |

## Mac de référence

À remplir une fois : modèle, puce, RAM, version de macOS, branché sur secteur, applications ouvertes pendant les mesures.
