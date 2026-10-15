# 0004 · Un daemon Python pour le cerveau, un shell Tauri pour l'interface

- **Date** : 2026-10-11 (orientation déjà prise en mars 2026)
- **Statut** : proposé (à accepter par l'équipe en v0.0)
- **Décideurs** : Tous

## Contexte et problème

Bloom doit capter un raccourci global, afficher un menu en surimpression et lire l'écran sur macOS, tout en appelant des modèles d'IA.

## Options envisagées

1. Tout en Python, interface comprise (PyQt, ou une app de barre de menus).
2. Une application native en Swift.
3. Un daemon Python pour le corps et les blocs A, B, C ; un shell Tauri (TypeScript et Rust) pour l'interface ; un pont IPC local entre les deux.

## Option retenue

3.

## Raison

- L'écosystème IA (modèles locaux, SDK, outils de mesure) est en Python.
- Tauri donne une interface légère, rapide à faire évoluer avec des technologies web que l'équipe connaît, et portable plus tard.
- Swift intègre mieux macOS, mais personne dans l'équipe ne le pratique, et le code IA resterait en Python : il faudrait un pont IPC de toute façon.

## Conséquences acceptées

- Deux processus, et un pont IPC à sécuriser : jeton aléatoire, vérification de l'en-tête Origin.
- Incertitude sur la permission Accessibilité de macOS quand Python tourne en sidecar de Tauri. À tester en v0.0 ; plan B : lire l'écran côté Rust et envoyer l'`Observation` au daemon, sans changer le contrat.
- Un empaquetage plus complexe (embarquer Python dans l'app).
