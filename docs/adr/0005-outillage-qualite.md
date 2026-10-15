# 0005 · Outillage qualité : uv, ruff, pyright, pytest, import-linter, GitHub Actions

- **Date** : 2026-10-15
- **Statut** : proposé (à accepter par l'équipe en v0.0)
- **Décideurs** : A

## Contexte et problème

La track exige une CI qui vérifie le code à chaque changement, des mesures de qualité au début et à la fin de l'année, et des tests qui attrapent de vraies régressions. Il faut une chaîne d'outils simple, identique sur les quatre machines et en CI.

## Options envisagées

1. pip et venv, flake8 et black, mypy.
2. uv (dépendances et Python), ruff (lint et format), pyright (types), pytest, import-linter (règles d'architecture).
3. Comme 2, avec ty (Astral) à la place de pyright.

## Option retenue

2.

## Raison

- Un seul outil rapide pour installer (uv) et un seul pour le lint et le format (ruff). Le fichier `uv.lock` fige les versions : la reproductibilité vient d'office.
- pyright est le moteur de Pylance dans VS Code : les erreurs de l'éditeur et celles de la CI sont les mêmes.
- import-linter transforme nos règles d'architecture en échecs de CI au lieu de vœux pieux.
- ty est encore en bêta (versions 0.0.x en août 2026) : trop jeune pour être le garde-fou de la CI.

## Conséquences acceptées

- pyright a besoin de Node, installé automatiquement par le paquet `pyright[nodejs]` : un téléchargement de plus au premier lancement.
- L'équipe doit apprendre uv.
- Candidate naturelle à une décision revue dans l'année : passer à ty quand sa version 1.0 sortira.
