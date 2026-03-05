#![allow(clippy::needless_return)]
//! # Simulation Testing System
//!
//! Three-phase testing:
//! 1. **Ordered** — run groups in definition order with full assertions.
//! 2. **Shuffled** — replay groups in random order (internal order preserved), assert no 500s.
//! 3. **Fuzz** — generate fake data matching recorded endpoint shapes, assert no 500s.
//!
//! Additionally, proptest-driven replay tests generate random operation
//! sequences from the collected cases and verify no 500s occur.

#[path = "../shared/mod.rs"]
#[macro_use]
mod shared;

mod auth_cases;
mod session_cases;
mod user_cases;

use proptest::prelude::*;
use proptest::test_runner::{Config, TestRunner};
use serial_test::serial;

use shared::case::{self, TestSuite};

// ── Test cases + fuzz ───────────────────────────────────────────────────

#[test]
#[serial]
fn test_cases_and_fuzz() {
    let seed = std::env::var("FUZZ_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64
        });
    let secs = std::env::var("FUZZ_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30u64);

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut suite = TestSuite::new().await;

        // Phase 1: ordered execution
        let auth = auth_cases::register(&mut suite).await;
        session_cases::register(&mut suite, &auth.alice_token).await;
        user_cases::register(&mut suite, &auth.alice_token).await;

        // Phase 2: shuffled group replay
        suite.run_shuffled(seed).await;

        // Phase 3: fuzz recorded endpoints
        suite.fuzz(seed, std::time::Duration::from_secs(secs)).await;
    });
}

// ── Proptest replay simulation ──────────────────────────────────────────

/// Collect test cases from ordered execution, then replay random sequences
/// via proptest. Cases are the single source of truth for endpoint definitions.
fn collect_cases(rt: &tokio::runtime::Runtime) -> Vec<case::StoredCase> {
    rt.block_on(async {
        let mut suite = TestSuite::new().await;
        let auth = auth_cases::register(&mut suite).await;
        session_cases::register(&mut suite, &auth.alice_token).await;
        user_cases::register(&mut suite, &auth.alice_token).await;
        suite.take_cases()
    })
}

#[test]
#[serial]
fn simulation_replay() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let cases = collect_cases(&rt);
    let n = cases.len();
    assert!(n > 0, "no cases collected");

    let config = Config {
        cases: 15,
        max_shrink_iters: 100,
        ..Config::default()
    };
    let mut test_runner = TestRunner::new(config);

    test_runner
        .run(
            &prop::collection::vec(0..n, 20..60),
            |indices| {
                rt.block_on(async {
                    case::replay_random(&cases, &indices).await;
                });
                Ok(())
            },
        )
        .unwrap();
}

/// Shorter variant for quick smoke testing.
#[test]
#[serial]
fn simulation_replay_smoke() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let cases = collect_cases(&rt);
    let n = cases.len();
    assert!(n > 0, "no cases collected");

    let config = Config {
        cases: 5,
        max_shrink_iters: 50,
        ..Config::default()
    };
    let mut test_runner = TestRunner::new(config);

    test_runner
        .run(
            &prop::collection::vec(0..n, 5..15),
            |indices| {
                rt.block_on(async {
                    case::replay_random(&cases, &indices).await;
                });
                Ok(())
            },
        )
        .unwrap();
}
