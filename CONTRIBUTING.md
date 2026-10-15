# Contribuer à Bloom

## Branches

- `main` est protégée : on y arrive uniquement par PR, avec la CI verte et une review.
- Une branche par sujet, nommée `<portée>/<sujet>` : `decision/adaptateur-laya`, `surface/observe-mac`.

## Commits

Première ligne : `type(portée): résumé`, au présent, en français, 72 caractères au plus.

| | Valeurs |
|---|---|
| Types | `feat` (fonctionnalité) · `fix` (correction) · `test` · `docs` · `refactor` · `lab` (mesures) · `chore` (outillage, dépendances) |
| Portées | `core` · `decision` · `context` · `surface` · `interface` · `lab` · `ci` · `docs` · `adr` |

- Un commit = un changement cohérent. Le corps du message, facultatif, explique le pourquoi.
- Exemples : `feat(decision): adaptateur Laya en local` · `fix(core): audit écrit même si un bloc plante` · `docs(adr): 0007 backend de décision retenu`

## Pull requests

- Petite : une portée, un sujet, idéalement moins de 300 lignes hors tests.
- Le modèle de PR se remplit en une minute.
- `make check` en local avant de pousser ; la CI relance exactement la même chose.
- Si `src/bloom/core/contracts.py` change : label `contrat`, review par les owners des blocs concernés.

## Review

- Personne ne merge sa propre PR.
- Une review contient au moins un commentaire. Une approbation sans commentaire n'est pas une review.
- On regarde : le contrat est-il respecté ? Un test couvre-t-il le changement ? Est-ce le plus simple possible ? La doc suit-elle ?
- Si la review fait changer la PR : une ligne dans `docs/evidence/reviews.md`.

## CI

`.github/workflows/ci.yml` lance `make check` puis `make bench` à chaque PR et à chaque push sur `main`. Quand elle bloque une PR cassée : une ligne dans `docs/evidence/regressions.md`.
