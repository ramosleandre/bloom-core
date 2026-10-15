# Mesures de bout en bout (PERF)

État : à construire en v0.3 (protocole écrit avant fin janvier). L'outillage de base existe déjà : le corps chronomètre chaque étape d'un run et l'écrit dans `RunState.timings_ms`, donc dans chaque ligne de `.bloom/audit.jsonl` :

`observe`, `assemble`, `rank`, `menu_latency`, `resolve`, `assess`, `execute`

C'est ce qui permettra le diagnostic exigé par PERF (« où ça bloque ? ») sans intuition.

## Protocole (à compléter avant la première mesure officielle)

- **Machine** : le Mac de référence (`lab/README.md`)
- **Scénario** : <les situations jouées, par exemple 10 situations × 10 répétitions>
- **Répétitions** : 100 runs ; les 5 premiers exclus (échauffement)
- **Backend de décision** : <nom et version>
- **Moment** : <par exemple après redémarrage, seules les applications du scénario ouvertes>
- **Calcul** : p50 et p95 de `menu_latency` sur les runs valides ; même calcul par étape pour le diagnostic

`menu_latency` compare deux horloges : l'appui (pris par le shell) et l'affichage (pris par le shell aussi). Les deux viennent du même processus, donc la mesure ne dépend pas du décalage entre le shell et le daemon.

## Script

`latency.py` (à écrire en v0.3) : lit l'audit, garde la fenêtre de mesure, calcule p50 et p95 par étape, écrit le résultat dans `lab/results/AAAA-MM-JJ_latency/`.
