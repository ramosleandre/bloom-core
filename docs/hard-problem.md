# Problème dur : note de caractérisation (O1)

> Livrable O1, à faire valider par le mentor **avant fin octobre 2026**.
> Statut : brouillon du 2026-10-15. Les seuils sont des propositions à défendre en séance ; la mesure de l'approche naïve reste à faire.

## Énoncé

Quand l'utilisateur appuie sur le raccourci, Bloom doit afficher presque instantanément les trois actions les plus pertinentes pour ce qu'il a à l'écran, puis décider de façon fiable si l'action choisie exige sa validation. Aujourd'hui, chacune de ces petites décisions passe par un appel à un LLM génératif : c'est trop lent pour un menu, et le résultat n'est ni stable ni chiffré.

## Ce qui le rend dur

- **Le temps.** Un menu doit apparaître en quelques centaines de millisecondes au plus. Au-delà d'environ une seconde, l'utilisateur perd le fil et va plus vite à la main (ordres de grandeur classiques de l'IHM : 0,1 s paraît instantané, 1 s est la limite du fil de pensée).
- **La fiabilité.** Une mauvaise proposition coûte la confiance ; un risque sous-estimé laisse passer une action dangereuse.
- **La confidentialité.** Le contexte (mails, documents) est personnel. L'envoyer à une API hébergée hors de l'UE pose un problème RGPD : il faut pouvoir décider en local.
- **Les ressources.** En local, il faut tenir sur un ordinateur portable, sans carte graphique dédiée.

## Ce que donne l'approche naïve

L'approche évidente : un appel à un LLM génératif hébergé, avec le contexte et la liste des 8 actions, qui renvoie les trois meilleures en JSON.

- **Latence mesurée : à faire avant fin octobre.** 30 appels, même prompt, même machine ; p50 et p95. (Tâche v0.0 [A] de la roadmap.)
- **Ce qui ne va pas, même sans la mesure :** une sortie libre à analyser, pas de probabilités exploitables pour fixer des seuils, un coût à chaque appel, et le contexte envoyé hors de la machine.

## Critères de succès (proposés)

| Critère | Seuil | Mesuré par |
|---|---|---|
| Latence raccourci → menu affiché | p95 ≤ 300 ms sur le Mac de référence | `RunState.timings_ms["menu_latency"]`, protocole de `lab/bloom/` |
| Pertinence | une action voulue dans le top 3 pour au moins 80 % d'un jeu d'au moins 50 situations réelles annotées | `make bench` |
| Sécurité | 100 % des actions `EXTERNAL` ou `DESTRUCTIVE` du jeu adversarial passent par une validation | tests, dans `make check` |
| Confidentialité | 100 % des décisions sur un contexte `LOCAL_ONLY` prises par un backend local | tests, dans `make check` |

## Résultat (à remplir fin avril)

Pour chaque critère : la valeur avant, la valeur après, l'écart, ce que l'intégration a remplacé, les limites connues.
