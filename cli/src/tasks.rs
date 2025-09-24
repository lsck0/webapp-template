use anyhow::{Result, ensure};
use tracing::{error, info, instrument};

use crate::{
    dependencies::is_binary_available,
    macros::{OutputExt, run},
};

#[instrument]
pub fn init() -> Result<()> {
    info!("Setting up pre-commit hooks.");

    if !is_binary_available("pre-commit") {
        error!("pre-commit not found, please install it.");
    }

    ensure!(
        run!("pre-commit install").success(),
        "Failed to install pre-commit hooks."
    );

    return Ok(());
}

#[instrument]
pub fn dev() -> Result<()> {
    run!(
        "GIT_COMMIT=$(git rev-parse HEAD) COMPOSE_BAKE=true docker compose --project-name wat --file \
         ../infrastructure/dev.compose.yml up --build --force-recreate --remove-orphans --abort-on-container-exit \
         --watch"
    );

    return Ok(());
}

#[instrument]
pub fn prod() -> Result<()> {
    run!(
        "GIT_COMMIT=$(git rev-parse HEAD) COMPOSE_BAKE=true docker compose --project-name wat --file \
         ../infrastructure/prod.compose.yml up --build --force-recreate --remove-orphans --abort-on-container-exit"
    );

    return Ok(());
}

#[instrument]
pub fn test() -> Result<()> {
    run!(
        "GIT_COMMIT=$(git rev-parse HEAD) COMPOSE_BAKE=true docker compose --project-name wat --file \
         ../infrastructure/test.compose.yml up --build --force-recreate  --remove-orphans --abort-on-container-exit"
    );

    return Ok(());
}

#[instrument]
pub fn bench() -> Result<()> {
    run!(
        "GIT_COMMIT=$(git rev-parse HEAD) COMPOSE_BAKE=true docker compose --project-name wat --file \
         ../infrastructure/bench.compose.yml up --build --force-recreate  --remove-orphans --abort-on-container-exit"
    );

    return Ok(());
}

#[instrument]
pub fn check() -> Result<()> {
    ensure!(
        run!(
            "cd ../services/core/server; cargo deny check --allow unlicensed --allow license-not-encountered --allow \
             duplicate"
        )
        .success(),
        "cargo deny check failed."
    );

    ensure!(
        run!("cd ../services/core/server; cargo check --workspace").success(),
        "cargo check failed."
    );

    ensure!(
        run!("cd ../services/core/server; cargo clippy --workspace --all-targets --all-features -- -D warnings")
            .success(),
        "cargo clippy failed."
    );

    ensure!(run!("cd ../services/core/client; npm i").success(), "npm i failed.");

    ensure!(run!("cd ../services/core/client; npx -y tsc").success(), "tsc failed.");

    ensure!(
        run!("cd ../services/core/client; npx -y eslint .").success(),
        "eslint failed."
    );

    ensure!(
        run!("cd ../services/core/ai; poetry install").success(),
        "poetry install failed."
    );

    ensure!(
        run!("cd ../services/core/ai; poetry run flake8 src").success(),
        "flake8 failed."
    );

    ensure!(
        run!("cd ../services/core/ai; poetry run mypy src").success(),
        "mypy failed."
    );

    info!("All checks passed.");

    return Ok(());
}

#[instrument]
pub fn bundle() -> Result<()> {
    ensure!(
        run!("cd ..; zip bundle.zip $(git ls-files)").success(),
        "bundle failed."
    );

    return Ok(());
}

#[instrument]
pub fn format() -> Result<()> {
    ensure!(run!("cargo fmt --all").success(), "cargo fmt failed.");

    ensure!(
        run!("cd ../services/core/server; cargo fmt --all").success(),
        "cargo fmt failed."
    );

    ensure!(
        run!("cd ../services/core/client; npx -y prettier . --write > /dev/null").success(),
        "prettier failed."
    );

    ensure!(
        run!("cd ../services/core/ai; black . 2> /dev/null").success(),
        "black failed."
    );
    ensure!(
        run!("cd ../services/core/ai; isort . > /dev/null").success(),
        "isort failed."
    );

    return Ok(());
}

#[instrument]
pub fn diesel(query: String) -> Result<()> {
    ensure!(
        run!(
            "cd ../services/core/server/; diesel --database-url=postgres://admin:password@postgres.localhost/root \
             {query}"
        )
        .success(),
        "diesel command failed: {query}"
    );

    return Ok(());
}

#[instrument]
pub fn export_bindings() -> Result<()> {
    ensure!(
        run!("cd ../services/core/server; cargo test export_bindings --workspace").success(),
        "export_bindings failed."
    );

    run!("rm ../services/core/client/src/api/bindings/*");
    run!(
        r#"find ../services/core/server -type f -regex '.*/bindings/[^/]*\.ts$' -print0 | xargs -0 -I {{}} mv {{}} ../services/core/client/src/api/bindings"#
    );
    run!("find ../services/core/server -type d -name 'bindings' -empty -delete");

    return Ok(());
}

#[instrument]
pub fn generate_pwa_assets() -> Result<()> {
    ensure!(
        run!("cd ../services/core/client; npm run generate-pwa-assets").success(),
        "pwa-assets-generator failed."
    );

    return Ok(());
}

#[instrument]
pub fn server_docs() -> Result<()> {
    ensure!(
        run!("cd ../services/core/server; cargo doc --lib --open --document-private-items").success(),
        "cargo doc failed."
    );

    return Ok(());
}

#[instrument]
pub fn stats() -> Result<()> {
    ensure!(is_binary_available("cloc"), "cloc not found, please install it.");

    run!("cloc .. -vcs git --exclude-content='.lock'");

    return Ok(());
}
