# 0006 · Pas de moteur de workflow générique en V1

- **Date** : 2026-10-15
- **Statut** : proposé (à accepter par l'équipe en v0.0). Revient sur l'orientation envisagée en mars 2026.
- **Décideurs** : A

## Contexte et problème

En mars 2026, l'architecture cible envisagée pour Bloom comprenait un orchestrateur de workflows en graphe (DAG), écrit en TypeScript sur Node, pour exécuter des branches en parallèle. Depuis, on a fixé la V1 : un seul enchaînement, du raccourci à l'action.

## Options envisagées

1. Construire dès maintenant un moteur de workflow générique (nœuds typés, graphe, exécution parallèle), en Node ou en Python.
2. Écrire l'enchaînement en dur dans une fonction (`pipeline.handle_trigger`), et n'extraire une abstraction qu'au moment où un deuxième cas d'usage réel l'impose.

## Option retenue

2.

## Raison

- La V1 n'a qu'un seul chemin : un moteur générique n'aurait aucun deuxième usage pour valider ses abstractions.
- Un framework s'extrait de cas réels ; inventé d'avance, il se trompe sur une bonne partie de ses détails.
- Un seul langage dans le daemon (Python) : moins de ponts, moins de choses à apprendre.

## Ce qui a changé d'avis

Le passage d'un projet de démonstration à une V1 bornée (8 actions, un déclencheur), et le constat que le risque principal est la qualité de la décision, pas l'orchestration.

## Conséquences acceptées

- Le jour où Bloom aura des déclencheurs automatiques ou une deuxième surface, il faudra refactoriser le pipeline ; c'est prévu en Tech 5.
- Pas d'exécution parallèle de branches en V1. Si une mesure montre qu'il en faut, ce sera une boucle d'optimisation PERF, documentée.
