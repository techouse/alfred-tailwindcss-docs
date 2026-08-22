SHELL := /bin/bash

.PHONY: help build build-release fmt fmt-check clippy test licenses package version-check ci clean

help:
	@printf '%-20s %s\n' 'Target' 'Description'
	@printf '%-20s %s\n' '------' '-----------'
	@printf '%-20s %s\n' 'build' 'Build the debug Rust executable.'
	@printf '%-20s %s\n' 'build-release' 'Build the release workflow directory.'
	@printf '%-20s %s\n' 'fmt' 'Format Rust source files.'
	@printf '%-20s %s\n' 'fmt-check' 'Check Rust formatting.'
	@printf '%-20s %s\n' 'clippy' 'Run strict Clippy checks.'
	@printf '%-20s %s\n' 'test' 'Run all Rust tests.'
	@printf '%-20s %s\n' 'licenses' 'Generate third-party license notices.'
	@printf '%-20s %s\n' 'package' 'Create a local .alfredworkflow package.'
	@printf '%-20s %s\n' 'version-check' 'Verify Cargo and optional tag versions agree.'
	@printf '%-20s %s\n' 'ci' 'Run all local CI checks.'
	@printf '%-20s %s\n' 'clean' 'Remove Rust and workflow build output.'

build:
	cargo build --locked

build-release:
	@set -euo pipefail; \
	missing_names=(); \
	for variable_name in ALGOLIA_APPLICATION_ID ALGOLIA_SEARCH_ONLY_API_KEY ALGOLIA_SEARCH_INDEX; do \
		if [[ -z "$${!variable_name+x}" ]]; then \
			missing_names+=("$$variable_name"); \
		fi; \
	done; \
	if (( $${#missing_names[@]} > 0 )) && [[ -f ./.env ]]; then \
		dotenv_exports="$$( \
			set -a; \
			source ./.env; \
			dotenv_status=$$?; \
			set +a; \
			if (( dotenv_status != 0 )); then \
				exit "$$dotenv_status"; \
			fi; \
			for variable_name in "$${missing_names[@]}"; do \
				if [[ -n "$${!variable_name:-}" ]]; then \
					printf '%s=%q\n' "$$variable_name" "$${!variable_name}"; \
				fi; \
			done \
		)"; \
		while IFS= read -r assignment; do \
			if [[ -n "$$assignment" ]]; then \
				eval "export $$assignment"; \
			fi; \
		done <<< "$$dotenv_exports"; \
	fi; \
	for variable_name in ALGOLIA_APPLICATION_ID ALGOLIA_SEARCH_ONLY_API_KEY ALGOLIA_SEARCH_INDEX; do \
		if [[ -z "$${!variable_name:-}" ]]; then \
			echo "$$variable_name must be set in the environment or .env file" >&2; \
			exit 1; \
		fi; \
	done; \
	for variable_name in ALGOLIA_APPLICATION_ID ALGOLIA_SEARCH_ONLY_API_KEY ALGOLIA_SEARCH_INDEX; do \
		export "$$variable_name"; \
	done; \
	cargo build --release --locked
	./scripts/package-workflow.sh target/release/alfred_tailwindcss_docs

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

clippy:
	cargo clippy --all-targets --all-features --locked -- -D warnings

test:
	cargo test --all-targets --locked

licenses:
	@mkdir -p build
	cargo-about generate --locked --fail --output-file build/THIRD_PARTY_LICENSES.html about.hbs

package: build-release
	@set -euo pipefail; \
	VERSION="$$(awk '/^\[package\]$$/ { p = 1; next } p && /^\[/ { exit } p && /^version = / { gsub(/["[:space:]]/, "", $$3); print $$3; exit }' Cargo.toml)"; \
	WORKFLOW_NAME="$${WORKFLOW_NAME:-tailwindcss-docs}"; \
	ARCHIVE="build/$${WORKFLOW_NAME}-v$${VERSION}.alfredworkflow"; \
	TEMP_ARCHIVE="$${ARCHIVE}.tmp.zip"; \
	trap 'rm -f "$$TEMP_ARCHIVE"' EXIT; \
	rm -f "$$TEMP_ARCHIVE"; \
	(cd build/dist && zip -qr "../$${WORKFLOW_NAME}-v$${VERSION}.alfredworkflow.tmp.zip" . \
		-x '.env' 'query_cache/*' 'update_cache/*' '*_cache/*' '*_cache_keys.json' 'workflow_intel'); \
	mv -f "$$TEMP_ARCHIVE" "$$ARCHIVE"; \
	trap - EXIT; \
	echo "Created $$ARCHIVE"

version-check:
	./scripts/version-check.sh

ci: fmt-check test clippy version-check licenses

clean:
	cargo clean
	rm -rf build
