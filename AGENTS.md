# AGENTS.md : Bloom

Instructions pour les agents IA (Claude Code, Codex, Cursor…) et pour toute personne qui touche au code.

## En bref

Bloom : sur macOS, un raccourci clavier ouvre un menu en cercle qui propose les trois actions les plus utiles pour ce qui est à l'écran ; l'action choisie s'exécute, avec une validation humaine si elle est risquée. Vue d'ensemble : [`docs/architecture.md`](docs/architecture.md). Avancement : [`docs/roadmap.md`](docs/roadmap.md).

## Commandes

| Commande | Effet |
|---|---|
| `make setup` | installe tout (uv) |
| `make check` | format, lint, types, règles d'architecture, tests : **ce que la CI exige** |
| `make fix` | corrige le format et le lint automatiquement |
| `make test b=decision` | tests d'un bloc (sans `b=` : tous) |
| `make bench` | banc de décision → `lab/results/local/` |
| `make run` | un run de démonstration, blocs simulés |
| `make quality` | rapport qualité daté |

Avant d'annoncer « fini » : `make check` passe.

## Carte du code

| Partie | Dossier | Rôle |
|---|---|---|
| Corps | `src/bloom/core/` | contrats (`contracts.py`), règles de sécurité (`policy.py`), le run (`pipeline.py`), audit |
| A · Décision | `src/bloom/decision/` | classe les actions, évalue le risque ; un fichier par backend de modèle dans `adapters/` |
| B · Contexte | `src/bloom/context/` | assemble le contexte, étiqueté par source, sensibilité et confiance |
| C · Surface | `src/bloom/surface/` | lit l'écran macOS, fournit le catalogue d'actions, les prépare et les exécute |
| D · Interface | `src/bloom/interface/` + `shell/` | pont vers le shell Tauri : raccourci, pie menu, validation |
| Composition | `src/bloom/app.py` | le seul endroit qui branche les blocs entre eux |
| Lab | `lab/` | mesures : modèles de décision, performance |

## Règles

Vérifiées par `make check` quand c'est possible.

1. Les blocs ne s'importent jamais entre eux. Du corps, ils n'importent que `bloom.core.contracts` et `bloom.core.policy`.
2. Le corps n'importe aucun bloc. Seul `bloom/app.py` les branche.
3. Le corps et le bloc contexte n'appellent jamais le réseau ni un SDK de modèle. Les SDK de modèles vivent uniquement dans `src/bloom/decision/adapters/`.
4. Plancher de criticité : un modèle peut monter le niveau de risque d'une action, jamais le descendre. Une action `EXTERNAL` demande toujours une validation.
5. Changer `src/bloom/core/contracts.py` : PR avec le label `contrat`, relue par les owners des blocs concernés.
6. Aucun secret dans le dépôt (`.env` local, voir `.env.example`). Aucune donnée personnelle réelle commitée.

## Rester simple

- Pas d'abstraction générique sans deux usages réels dans le code.
- Pas de nouvelle dépendance sans ADR (`docs/adr/`).
- Une PR = une portée = un sujet. Si la tâche touche deux blocs, c'est probablement un changement de contrat : arrête-toi et demande.
- Pas d'optimisation sans mesure avant et après (`lab/`).

## Définition de « fini »

- [ ] `make check` vert
- [ ] un test couvre le comportement ajouté ou modifié
- [ ] `docs/architecture.md` à jour si la structure ou un contrat a changé
- [ ] la case de `docs/roadmap.md` est cochée
- [ ] la preuve est notée dans `docs/evidence/` si le changement en produit une (tableau ci-dessous)

## Où noter quoi

| Il s'est passé… | Écris dans |
|---|---|
| une décision structurante (techno, découpage, format, abandon) | un nouvel ADR `docs/adr/NNNN-titre.md` |
| un test a attrapé une régression, ou la CI a bloqué une PR | `docs/evidence/regressions.md` |
| une review a fait changer une PR | `docs/evidence/reviews.md` |
| une dette assumée | `docs/evidence/tech-debt.md` |
| une mesure officielle | `lab/results/AAAA-MM-JJ_nom/` puis `docs/evidence/perf.md` |
| une source de veille lue, avec une conclusion | `docs/evidence/research-log.md` |

Commits, PR et review : [`CONTRIBUTING.md`](CONTRIBUTING.md).
