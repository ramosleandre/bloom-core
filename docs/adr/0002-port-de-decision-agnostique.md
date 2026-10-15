# 0002 · Une interface interne pour les modèles de décision, un adaptateur par fournisseur

- **Date** : 2026-09-30
- **Statut** : proposé (à accepter par l'équipe en v0.0)
- **Décideurs** : A

## Contexte et problème

Les modèles de décision « System 1 » (Jev, sorti le 15 septembre 2026, puis Laya, Von et d'autres) répondent exactement à notre problème dur : choisir vite parmi des options données, avec des probabilités. Mais la catégorie a moins d'un mois, les chiffres sont annoncés par les éditeurs, et Jev a suspendu ses inscriptions une semaine après sa sortie.

## Options envisagées

1. Appeler directement le SDK de Jev depuis le code de Bloom.
2. Définir une interface interne (`DecisionPort` : une question de type choix, score ou oui/non, une réponse avec probabilités), avec un adaptateur par backend : hébergé, local, règles.

## Option retenue

2.

## Raison

- Ne pas dépendre d'un fournisseur en accès anticipé, sans engagement de service.
- Des alternatives locales existent déjà : elles évitent d'envoyer le contexte personnel hors de la machine (RGPD).
- Le prototype de décision exigé par la track compare plusieurs backends sur le même protocole : il faut qu'ils soient interchangeables.

## Conséquences acceptées

- Une couche de plus, et une interface au plus petit dénominateur commun : on renonce aux fonctions propres à un fournisseur.
- Un adaptateur à maintenir par backend.
- Règle vérifiée par la CI : aucun SDK de modèle n'est importé ailleurs que dans `src/bloom/decision/adapters/`.
