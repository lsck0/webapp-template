default:
    @just --list

# Run the project in development mode
dev:
    @GIT_COMMIT=$(git rev-parse HEAD) LAST_UPDATED=$(git log -1 --format=%cI HEAD) COMPOSE_BAKE=true \
        docker compose \
        --project-name wat \
        --file infrastructure/dev.compose.yml \
        up --build --force-recreate --remove-orphans --abort-on-container-exit --watch

# Run the project in production mode
prod:
    @GIT_COMMIT=$(git rev-parse HEAD) LAST_UPDATED=$(git log -1 --format=%cI HEAD) COMPOSE_BAKE=true \
        docker compose \
        --project-name wat \
        --file infrastructure/prod.compose.yml \
        up --build --force-recreate --remove-orphans --abort-on-container-exit

# Run the project in test mode
test:
    @GIT_COMMIT=$(git rev-parse HEAD) LAST_UPDATED=$(git log -1 --format=%cI HEAD) COMPOSE_BAKE=true \
        docker compose \
        --project-name wat \
        --file infrastructure/test.compose.yml \
        up --build --force-recreate --remove-orphans --abort-on-container-exit

# Run the project in benchmark mode
bench:
    @GIT_COMMIT=$(git rev-parse HEAD) LAST_UPDATED=$(git log -1 --format=%cI HEAD) COMPOSE_BAKE=true \
        docker compose \
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

    cd "$ROOT/services/core/server" && just check
    cd "$ROOT/services/core/client" && just check
    cd "$ROOT/services/core/ai" && just check

# Bundle the project into a zip file
bundle:
    @zip bundle.zip $(git ls-files)
