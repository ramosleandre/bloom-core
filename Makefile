# Bloom : toutes les commandes. `make` seul affiche l'aide.
.DEFAULT_GOAL := help
.PHONY: help setup check fix test bench run quality

b ?=
backends ?= rules
data ?= lab/datasets/situations.sample.jsonl
out ?=
ENV := $(if $(wildcard .env),--env-file .env,)

help: ## Affiche cette aide
	@grep -E '^[a-z]+:.*## ' $(MAKEFILE_LIST) | awk 'BEGIN{FS=":.*## "}{printf "  make %-8s %s\n", $$1, $$2}'

setup: ## Installe tout (une fois, puis à chaque changement de dépendances)
	uv sync

check: ## Ce que la CI exige : format, lint, types, règles d'archi, tests
	uv run ruff format --check .
	uv run ruff check .
	uv run pyright
	uv run lint-imports
	uv run pytest

fix: ## Corrige automatiquement le format et ce que le linter sait corriger
	uv run ruff format .
	uv run ruff check --fix .

test: ## Tests : tous, ou un bloc (make test b=decision)
	uv run pytest tests/$(b)

bench: ## Banc de décision (make bench backends=rules,laya data=... out=lab/results/AAAA-MM-JJ_nom)
	uv run $(ENV) python lab/decision/bench.py --backends $(backends) --data $(data) $(if $(out),--out $(out),)

run: ## Un run de démonstration (blocs simulés)
	uv run $(ENV) python -m bloom

quality: ## Rapport qualité daté, baseline en décembre et final en mai (lab/results/quality/)
	@mkdir -p lab/results/quality
	@f=lab/results/quality/$$(date +%F).txt; \
	{ echo "# Qualité du $$(date +%F), commit $$(git rev-parse --short HEAD 2>/dev/null || echo '?')"; \
	  echo; echo "## ruff (avertissements par règle)"; uv run ruff check . --statistics --exit-zero; \
	  echo; echo "## pyright"; uv run pyright 2>&1 | tail -1; \
	  echo; echo "## couverture des tests"; uv run pytest --cov=bloom --cov-report=term 2>&1 | tail -25; \
	} > $$f; echo "Rapport écrit : $$f"
