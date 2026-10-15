# Bloom: all commands. `make` alone shows the help.
.DEFAULT_GOAL := help
.PHONY: help setup check fix test bench run quality

b ?=
backends ?= rules
data ?= lab/datasets/situations.sample.jsonl
out ?=
ENV := $(if $(wildcard .env),--env-file .env,)

help: ## Show this help
	@grep -E '^[a-z]+:.*## ' $(MAKEFILE_LIST) | awk 'BEGIN{FS=":.*## "}{printf "  make %-8s %s\n", $$1, $$2}'

setup: ## Install everything (once, then whenever dependencies change)
	uv sync

check: ## What CI requires: format, lint, types, architecture rules, tests
	uv run ruff format --check .
	uv run ruff check .
	uv run pyright
	uv run lint-imports
	uv run pytest

fix: ## Auto-fix formatting and whatever the linter can fix
	uv run ruff format .
	uv run ruff check --fix .

test: ## Tests: all, or one block (make test b=decision)
	uv run pytest tests/$(b)

bench: ## Decision bench (make bench backends=rules,laya data=... out=lab/results/YYYY-MM-DD_name)
	uv run $(ENV) python lab/decision/bench.py --backends $(backends) --data $(data) $(if $(out),--out $(out),)

run: ## A demo run (mocked blocks)
	uv run $(ENV) python -m bloom

quality: ## Dated quality report, baseline in December and final in May (lab/results/quality/)
	@mkdir -p lab/results/quality
	@f=lab/results/quality/$$(date +%F).txt; \
	{ echo "# Quality on $$(date +%F), commit $$(git rev-parse --short HEAD 2>/dev/null || echo '?')"; \
	  echo; echo "## ruff (warnings per rule)"; uv run ruff check . --statistics --exit-zero; \
	  echo; echo "## pyright"; uv run pyright 2>&1 | tail -1; \
	  echo; echo "## test coverage"; uv run pytest --cov=bloom --cov-report=term 2>&1 | tail -25; \
	} > $$f; echo "Report written: $$f"
