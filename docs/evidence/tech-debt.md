# Dette technique et qualité (O2)

> Ce qu'on sait devoir corriger, pourquoi pas maintenant, et à peu près combien ça coûterait. Les rapports de qualité datés (référence en décembre, final en mai) sont produits par `make quality` dans `lab/results/quality/`.

## Dette

| Date | Bloc | Quoi | Pourquoi pas maintenant | Coût estimé | Statut |
|---|---|---|---|---|---|
| 2026-10-15 | core | quand l'utilisateur modifie une action à la validation (`edit`), les nouveaux arguments ne sont pas revalidés | aucune action n'a encore de schéma d'arguments | 0,5 jour | ouvert |

## Qualité mesurée

| Date | Avertissements ruff | Erreurs pyright | Couverture des tests | Rapport |
|---|---|---|---|---|

## Règles désactivées volontairement

| Règle | Où | Pourquoi |
|---|---|---|
| E501 (longueur de ligne) | partout | le formateur gère le code ; les commentaires en français dépassent souvent |
| RUF001, RUF002, RUF003 (caractères unicode ambigus) | partout | faux positifs sur la typographie française |
| S101 (usage d'`assert`) | `tests/` | `assert` est la façon normale d'écrire un test pytest |
