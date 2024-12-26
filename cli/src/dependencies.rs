use anyhow::{Result, bail, ensure};
use derive_more::derive::Display;
use tracing::{info, instrument};

use crate::macros::{OutputExt, run};

#[derive(Debug, PartialEq, Eq, Display)]
enum Dependency {
    #[display("{_0}")]
    SameName(&'static str),

    #[display("{binary}")]
    DifferentName {
        binary: &'static str,
        package: &'static str,
    },
}
use Dependency::*;

const NEEDED_BASE_DEPENDENCIES: [Dependency; 9] = [
    SameName("cargo"),
    SameName("clang"),
    SameName("docker"),
    // SameName("docker-compose"),
    SameName("git"),
    SameName("mold"),
    SameName("npm"),
    SameName("poetry"),
    SameName("python"),
    SameName("zip"),
];
const NEEDED_CARGO_DEPENDENCIES: [Dependency; 3] = [
    SameName("cargo-binstall"),
    SameName("cargo-deny"),
    DifferentName {
        binary: "diesel",
        package: "diesel_cli",
    },
];

/// This is POSIX only.
pub(crate) fn is_binary_available(name: &str) -> bool {
    return run!("command -v {} > /dev/null", name).success();
}

/// Check if the dependencies are installed.
/// Missing Cargo dependencies will be installed automatically.
#[instrument]
pub(crate) fn check_dependencies() -> Result<()> {
    let mut missing_dependencies = vec![];

    // check for base dependencies
    for dependency in NEEDED_BASE_DEPENDENCIES {
        match dependency {
            SameName(binary) => {
                if !is_binary_available(binary) {
                    missing_dependencies.push(binary);
                }
            }
            DifferentName { binary, package } => {
                if !is_binary_available(binary) {
                    missing_dependencies.push(package);
                }
            }
        }
    }

    if !missing_dependencies.is_empty() {
        bail!("Missing base dependencies: {}", missing_dependencies.join(", "));
    }

    // check for cargo dependencies
    for dependency in NEEDED_CARGO_DEPENDENCIES {
        match dependency {
            SameName(binary) => {
                if !is_binary_available(binary) {
                    missing_dependencies.push(binary);
                }
            }
            DifferentName { binary, package } => {
                if !is_binary_available(binary) {
                    missing_dependencies.push(package);
                }
            }
        }
    }

    if missing_dependencies.is_empty() {
        info!("Found all dependencies.");
        return Ok(());
    }

    info!(
        "Missing cargo dependencies: {}. Installing them.",
        missing_dependencies.join(", ")
    );

    // check first if cargo-binstall is missing and install it if so
    if missing_dependencies.contains(&"cargo-binstall") {
        missing_dependencies.retain(|&dependency| dependency != "cargo-binstall");
        ensure!(
            run!("cargo install cargo-binstall").success(),
            "Failed to install cargo-binstall."
        );
    }

    // install the rest of the missing dependencies
    for dependency in missing_dependencies {
        if !run!("cargo binstall -y {}", dependency).success() {
            info!("Failed to install {dependency} with cargo binstall, trying cargo install.");

            ensure!(
                run!("cargo install {}", dependency).success(),
                "Failed to install dependency: {}",
                dependency
            );
        }
    }

    info!("Installed all missing dependencies.");
    return Ok(());
}
