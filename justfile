default:
    @just --list

# Run the project in development mode
dev:
    @GIT_COMMIT=$(git rev-parse HEAD) COMPOSE_BAKE=true docker compose \
        --project-name wat \
        --file infrastructure/dev.compose.yml \
        up --build --force-recreate --remove-orphans --abort-on-container-exit --watch

# Run the project in production mode
prod:
    @GIT_COMMIT=$(git rev-parse HEAD) COMPOSE_BAKE=true docker compose \
        --project-name wat \
        --file infrastructure/prod.compose.yml \
        up --build --force-recreate --remove-orphans --abort-on-container-exit

# Run the project in test mode
test:
    @GIT_COMMIT=$(git rev-parse HEAD) COMPOSE_BAKE=true docker compose \
        --project-name wat \
        --file infrastructure/test.compose.yml \
        up --build --force-recreate --remove-orphans --abort-on-container-exit

# Run the project in benchmark mode
bench:
    @GIT_COMMIT=$(git rev-parse HEAD) COMPOSE_BAKE=true docker compose \
        --project-name wat \
        --file infrastructure/bench.compose.yml \
        up --build --force-recreate --remove-orphans --abort-on-container-exit

# Wrap dev/prod/test/bench with a dashboard when using tmux
tmux rule:
   #!/usr/bin/env bash
   set -euo pipefail

   tmux \
     send-keys 'just {{rule}}' C-m \; \
     split-window -v -p 42 \; \
     split-window -h -p 33 \; \
     select-pane -t 2 \; send-keys 'docker stats' C-m \; \
     select-pane -t 3 \; send-keys 'watch -d -n 1 "nc -z 127.0.0.1 80 >/dev/null && echo UP || echo DOWN"' C-m \; \
     select-pane -t 1

# Run linters and type checks
check:
	#!/usr/bin/env bash
	set -euo pipefail
	ROOT="{{ justfile_directory() }}"

	cd "$ROOT/services/core/server" && cargo deny check --allow unlicensed --allow license-not-encountered --allow duplicate 2>&1 | grep -v "warning\[parse-error\]: error parsing SPDX" | grep -v "warning\[no-license-field\]:" | grep -v "^ ├" | grep -v "^   " | grep -v "^$" || true
	cd "$ROOT/services/core/server" && cargo check --workspace
	cd "$ROOT/services/core/server" && cargo clippy --workspace --all-targets --all-features -- -D warnings
	cd "$ROOT/services/core/client" && npm ci
	cd "$ROOT/services/core/client" && npx -y tsc
	cd "$ROOT/services/core/client" && npx -y eslint .
	cd "$ROOT/services/core/ai" && poetry install
	cd "$ROOT/services/core/ai" && poetry run flake8 src
	cd "$ROOT/services/core/ai" && poetry run mypy src

# Run code formatters
fmt:
    cd services/core/server && cargo fmt --all
    cd services/core/client && npx -y prettier . --write > /dev/null
    cd services/core/ai && black . 2> /dev/null
    cd services/core/ai && isort . > /dev/null

# Bundle the project into a zip file
bundle:
    @zip bundle.zip $(git ls-files)

# Export API bindings to the client
export-bindings:
    @cd services/core/server && cargo test export_bindings --workspace
    @rm -f services/core/client/src/api/bindings/*
    @find services/core/server -type f -regex '.*/bindings/[^/]*\.ts$' -print0 | xargs -0 -I {} mv {} services/core/client/src/api/bindings
    @find services/core/server -type d -name 'bindings' -empty -delete

# Generate PWA assets
generate-pwa-assets:
    @cd services/core/client && npm run generate-pwa-assets

# Open the server documentation
server-docs:
    @cd services/core/server && cargo doc --lib --open --document-private-items

# Run diesel CLI
diesel *args:
    @cd services/core/server && diesel --database-url=postgres://admin:password@postgres.localhost/root {{args}}

# Show project statistics
stats:
    @cloc . -vcs git --exclude-content='.lock'
