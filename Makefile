# Bloom: all commands. `make` alone shows the help.
.DEFAULT_GOAL := help
.PHONY: help setup check fix test bench run quality ui-setup ui-build ui ui-preview desktop ui-check ui-test gnome-setup gnome-status gnome-remove gnome-ready ui-gnome

b ?=
backends ?= rules
data ?= lab/datasets/situations.sample.jsonl
out ?=
ENV := $(if $(wildcard .env),--env-file .env,)

help: ## Show this help
	@grep -E '^[a-z][a-z-]*:.*## ' $(MAKEFILE_LIST) | awk 'BEGIN{FS=":.*## "}{printf "  make %-10s %s\n", $$1, $$2}'

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

ui-setup: ## Install Python and frontend dependencies for the interactive UI
	uv sync
	npm --prefix shell ci

ui-build: ## Generate shared contracts and build the frontend
	npm --prefix shell run contracts
	npm --prefix shell run build

ui: desktop ## Launch the Tauri desktop frontend

gnome-setup: ## Install the optional GNOME desktop adapter for the current user
	npm --prefix shell run gnome:install

gnome-status: ## Inspect the optional GNOME adapter
	npm --prefix shell run gnome:status

gnome-remove: ## Remove the optional GNOME adapter
	npm --prefix shell run gnome:remove

gnome-ready:
	node shell/scripts/gnome.mjs check

ui-gnome: gnome-ready ui-build ## Check GNOME integration, then launch with an XWayland overlay
	GDK_BACKEND=x11 uv run $(ENV) python -m bloom --desktop

ui-preview: ui-build ## Open an optional browser preview of the UI
	uv run $(ENV) python -m bloom --ui

desktop: ui-build ## Launch the Tauri desktop app (requires native development libraries)
	uv run $(ENV) python -m bloom --desktop

ui-check: ## Check generated contracts and frontend types
	npm --prefix shell run check

ui-test: ui-build ## Browser tests against a real Python core (requires Playwright Chromium)
	npm --prefix shell test

quality: ## Dated quality report, baseline in December and final in May (lab/results/quality/)
	@mkdir -p lab/results/quality
	@f=lab/results/quality/$$(date +%F).txt; \
	{ echo "# Quality on $$(date +%F), commit $$(git rev-parse --short HEAD 2>/dev/null || echo '?')"; \
	  echo; echo "## ruff (warnings per rule)"; uv run ruff check . --statistics --exit-zero; \
	  echo; echo "## pyright"; uv run pyright 2>&1 | tail -1; \
	  echo; echo "## test coverage"; uv run pytest --cov=bloom --cov-report=term 2>&1 | tail -25; \
	} > $$f; echo "Report written: $$f"
