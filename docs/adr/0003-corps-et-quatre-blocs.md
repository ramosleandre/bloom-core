# 0003 · Un corps et quatre blocs en étoile, qui n'échangent que des contrats typés

- **Date** : 2026-10-11
- **Statut** : proposé (à accepter par l'équipe en v0.0)
- **Décideurs** : Tous

## Contexte et problème

Quatre personnes à temps partiel doivent développer en parallèle sans s'attendre, et le résultat doit rester testable de bout en bout.

## Options envisagées

1. Des blocs qui s'appellent directement entre eux.
2. Tout faire passer par un bus d'événements.
3. Une étoile : un corps qui orchestre le run et appelle chaque bloc ; les blocs n'échangent que des types définis dans `contracts.py`.

## Option retenue

3.

## Raison

- Chacun code contre les mocks des trois autres blocs dès la première semaine.
- Le run complet se teste en un seul test d'intégration.
- Les règles se vérifient automatiquement (import-linter) : un bloc qui en importe un autre casse la CI.
- Le bus seul rend l'ordre des étapes implicite, donc difficile à suivre et à tester pour une équipe débutante sur le projet.

## Conséquences acceptées

- Le corps est un point de passage obligé, tenu par A.
- Changer un contrat coûte une review par les owners concernés.
- Un peu d'indirection : quand une action doit écrire du texte, le bloc C reçoit le générateur du bloc A par injection, au lieu de l'importer.
