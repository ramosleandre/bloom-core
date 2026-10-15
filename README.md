# Bloom

Un raccourci clavier, un menu en cercle, la bonne action. Bloom regarde ce que tu fais sur macOS, propose les trois actions les plus utiles et les exécute, en te demandant ton accord quand c'est risqué.

Projet EIP Epitech (Technical Track, 2026-27). État : **v0.0, socle**. Le corps tourne ; les blocs sont encore simulés. Avancement détaillé : [`docs/roadmap.md`](docs/roadmap.md).

## Prérequis

- macOS pour le produit complet. Linux ou macOS suffit pour le corps, les tests et le banc de mesure.
- [uv](https://docs.astral.sh/uv/) : il installe aussi la bonne version de Python (3.13).
- `make`

## Installation

```bash
make setup     # dépendances
make check     # tout doit être vert
make run       # un run de démonstration
```

## Configuration

Copier `.env.example` en `.env`. Aucune variable n'est obligatoire pour démarrer.

| Variable | Rôle | Défaut |
|---|---|---|
| `BLOOM_DECISION_BACKENDS` | backends de décision, par ordre de préférence | `rules` |
| `BLOOM_AUDIT_PATH` | journal local des runs (données personnelles, jamais commité) | `.bloom/audit.jsonl` |
| `JEV_API_KEY`, `LLM_API_KEY` | clés des modèles hébergés (à partir de la v0.2) | vide |

## Commandes

`make help` les liste toutes : `setup`, `check`, `fix`, `test [b=bloc]`, `bench`, `run`, `quality`.

## Structure

```
src/bloom/core/       le corps : contrats, règles de sécurité, pipeline du run, audit
src/bloom/decision/   bloc A : classement des actions, risque, génération
src/bloom/context/    bloc B : contexte et mémoire
src/bloom/surface/    bloc C : lecture de l'écran, catalogue et exécution des actions
src/bloom/interface/  bloc D : pont vers le shell
src/bloom/app.py      branche les blocs entre eux
shell/                bloc D : app Tauri (à partir de la v0.2)
tests/                un dossier par bloc, plus integration/ (le chemin critique)
lab/                  mesures : modèles de décision, performance
docs/                 architecture, problème dur, roadmap, décisions, preuves
```

## Tests

`make test` lance tout, `make test b=decision` un seul bloc. `tests/integration/` couvre le chemin critique : un run complet, avec ses quatre issues possibles (exécuté, annulé, refusé, échoué).

## Documentation

- [`docs/architecture.md`](docs/architecture.md) : schémas, le corps et les quatre blocs, contrats, règles de sécurité, données, pannes
- [`docs/hard-problem.md`](docs/hard-problem.md) : le problème dur et ses critères de succès chiffrés
- [`docs/roadmap.md`](docs/roadmap.md) : l'avancement, version par version
- [`docs/adr/`](docs/adr/) : les décisions d'architecture
- [`docs/evidence/`](docs/evidence/README.md) : les preuves du suivi Epitech (veille, reviews, régressions, mesures…)
- [`AGENTS.md`](AGENTS.md) : instructions pour les agents IA · [`CONTRIBUTING.md`](CONTRIBUTING.md) : commits, PR, review
