# Sense: commands for the context engine. Run from the repository root with `make -C sense <target>`.
# Kept separate from the root Makefile so the Python workflow needs no Rust toolchain.
.DEFAULT_GOAL := help
.PHONY: help check test schema e2e

help: ## Show this help
	@grep -E '^[a-z0-9-]+:.*## ' $(MAKEFILE_LIST) | awk 'BEGIN{FS=":.*## "}{printf "  make -C sense %-7s %s\n", $$1, $$2}'

check: ## What CI requires: format, lints, tests, schema drift, macOS and Windows type-check
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	cargo test
	cargo check --target aarch64-apple-darwin
	cargo check --target x86_64-pc-windows-msvc

test: ## Unit and scenario tests (no desktop needed)
	cargo test

schema: ## Regenerate schema/sense.schema.json from the Rust types
	SENSE_UPDATE_SCHEMA=1 cargo test -p sense-schema

e2e: ## End to end on GNOME, in an isolated headless Shell (needs gnome-shell and zenity)
	cargo build
	dbus-run-session -- tests/e2e-gnome.sh
