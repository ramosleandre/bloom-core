# Dispositif de veille (O1)

> Livrable O1 à décrire **avant fin octobre 2026**, puis à tenir jusqu'en mai.
> Statut : brouillon du 2026-10-15. Le guide exige un dispositif « décrit et démontrable, pas seulement affirmé » : chaque outil coché doit pouvoir être montré en séance.

## Qui suit quoi

| Domaine | Qui | Pourquoi ce domaine |
|---|---|---|
| Modèles de décision System 1, évaluation, calibration | A | c'est le problème dur |
| Mémoire et contexte des agents (mem0, Letta…) | B | son bloc, et la piste de recherche |
| Automatisation macOS, sécurité des agents (injection), MCP | C | son bloc, et les risques |
| UX des menus radiaux, Tauri, latence perçue | D | son bloc, et l'axe PERF |

## Sources

| Source | Type | Qui | Ce qu'elle apporte | Pourquoi on la garde |
|---|---|---|---|---|
| Dépôt `NandhaKishorM/laya` (releases) | dépôt | A | modèle System 1 open source, local | candidat du prototype de décision |
| Dépôt `wfzyx/von` (releases) | dépôt | A | modèle System 1 sur CPU | autre candidat local |
| Liste `unikcc/awesome-system1-decision-models` | liste | A | recensement de la catégorie | ne pas rater une sortie |
| Documentation et blog de TypeSafe (Jev) | doc officielle | A | API, prix, limites | backend hébergé candidat |
| Alertes arXiv (decision model, calibration, agent memory, prompt injection) | publications | A, B, C | méthodes publiées | état de l'art daté |
| Dépôts mem0 et Letta (releases) | dépôts | B | état de l'art de la mémoire d'agents | se comparer à l'existant |
| OWASP Top 10 for LLM Applications | référence sécurité | C | catalogue des attaques, dont l'injection | grille pour le jeu adversarial |
| Documentation Apple Accessibility + pyobjc | doc officielle | C | lecture de l'écran entre applications | risque n°1 de l'architecture |
| Spécification MCP (releases) | spec | C | connecteurs de la V2 | ne pas réinventer les connecteurs |
| Tauri (releases, documentation) | doc officielle | D | le shell desktop | dépendance directe |
| Littérature IHM sur les pie menus et marking menus | publications | D | ce qui rend un menu radial efficace | décisions UX argumentées |

Une source qui n'a produit aucune entrée en deux mois sort de la liste ; le retrait se note dans le journal.

## Outils et automatisations

- [ ] Canal Discord `#veille`, alimenté automatiquement par un bot RSS (flux arXiv, blogs, releases)
- [ ] « Watch → Custom → Releases » sur GitHub pour chaque dépôt suivi
- [ ] Base de notes partagée : [`research-log.md`](research-log.md), dans le dépôt, relue en PR
- [ ] À chaque compte rendu, une capture du canal et des alertes, comme preuve que le dispositif tourne

## Rythme

- 30 minutes par semaine et par personne
- au moins une entrée de journal toutes les deux semaines par personne (une soixantaine sur l'année, pour un minimum de 20)
- une présentation interne par mois dès novembre, à tour de rôle ([`presentations.md`](presentations.md))

## Ce qu'est une entrée valable

Date, source, sujet, ce que ça change ou pourrait changer pour Bloom, et **ce qu'on en a fait, ou pourquoi on n'a rien fait**. Une entrée sans conclusion ne compte pas.
