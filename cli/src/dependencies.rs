use anyhow::{Result, bail};
use tracing::{info, instrument};

use crate::macros::{OutputExt, run};

const NEEDED_DEPENDENCIES: [&str; 10] = [
    "cargo",
    "cargo-deny",
    "clang",
    "diesel",
    "git",
    "mold",
    "npm",
    "poetry",
    "python",
    "zip",
];

/// This is POSIX only.
pub(crate) fn is_binary_available(name: &str) -> bool {
    return run!("command -v {} > /dev/null", name).success();
}

#[instrument]
pub(crate) fn check_dependencies() -> Result<()> {
    let mut missing_dependencies = vec![];

    for dependency in NEEDED_DEPENDENCIES {
        if !is_binary_available(dependency) {
            missing_dependencies.push(dependency);
        }
    }

    if !missing_dependencies.is_empty() {
        bail!("Missing base dependencies: {}", missing_dependencies.join(", "));
    }

    info!("Found all dependencies.");
    return Ok(());
}
