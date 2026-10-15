# 0001 · Un nouveau dépôt plutôt que faire évoluer HostaAgent

- **Date** : 2026-09-30
- **Statut** : proposé (à accepter par l'équipe en v0.0)
- **Décideurs** : A

## Contexte et problème

Bloom a besoin d'un socle pour le corps et les blocs. Un projet existant de l'équipe, HostaAgent (framework d'agent ReAct minimal, licence MIT, environ 2 100 lignes), contient des briques réutilisables : événements, permissions, séparation entre le moteur, l'environnement et le mode de lancement.

## Options envisagées

1. Faire évoluer HostaAgent jusqu'à en faire le socle de Bloom.
2. Créer un nouveau dépôt et n'y reprendre que les principes de découpage.

## Option retenue

2.

## Raison

- La thèse de HostaAgent (« la boucle ne grandit jamais ») s'oppose à celle de Bloom (un enchaînement de décisions de natures différentes).
- HostaAgent repose sur une librairie LLM (OpenHosta) qui ne permet pas de brancher un modèle de décision.
- Une grande partie de son code (une interface en ligne de commande de 882 lignes, un registre de configuration) n'a pas d'usage dans Bloom.

## Conséquences acceptées

- On réécrit les tests et on perd l'historique git de ces briques.
- On ne réimplémente les morceaux utiles (événements, permissions) qu'au moment où un premier usage réel apparaît.
