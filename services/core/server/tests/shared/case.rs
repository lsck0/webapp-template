#![allow(clippy::needless_return)]
//! Three-phase test harness:
//!
//! 1. **Ordered** — run groups in definition order with full assertions.
//! 2. **Shuffled** — replay groups in random order (internal order preserved), assert no 500s.
//! 3. **Fuzz** — generate fake data matching recorded endpoint shapes.
//!
//! ```ignore
//! let mut suite = TestSuite::new().await;
//!
//! let token = test_group!(suite, "auth", {
//!     let resp = suite.case("login")
//!         .post(LOGIN_ENDPOINT)
//!         .body(LoginInfo { name: "admin".into(), password: "pass".into(), otp: None })
//!         .expect(|r| assert_eq!(r.status_code(), 200))
//!         .await;
//!     resp.json::<SessionDTO>().access_token  // type-safe forwarding
//! });
//!
//! test_group!(suite, "user ops", {
//!     suite.case("get user")
//!         .get(USER_ENDPOINT)
//!         .token(&token)
//!         .expect(|r| assert_eq!(r.status_code(), 200))
//!         .await;
//! });
//!
//! suite.run_shuffled(seed).await;   // phase 2
//! suite.fuzz(seed, duration).await; // phase 3
//! ```

use std::net::SocketAddr;
use std::time::Duration;

use axum_test::{TestResponse, TestServer};
use http::{HeaderName, HeaderValue, Method};
use models::DbInitFlags;
use serde::Serialize;
use serde_json::Value;

use super::fuzz::{self, DataKind, EndpointRecord, extract_shape};

// ── Macro ───────────────────────────────────────────────────────────────

/// Register a test group. Cases inside share a group tag for shuffled replay.
/// The block's return value is forwarded — giving type-safe data passing between groups.
#[macro_export]
macro_rules! test_group {
    ($suite:expr, $name:expr, $body:expr) => {{
        $suite.begin_group($name);
        let __result = $body;
        $suite.end_group();
        __result
    }};
}

// ── Stored case snapshot ────────────────────────────────────────────────

#[derive(Clone)]
pub struct StoredCase {
    pub label: String,
    pub group: String,
    pub method: Method,
    pub path: &'static str,
    pub body: Option<Value>,
    pub query: Option<Value>,
    pub token: Option<String>,
}

// ── TestSuite ───────────────────────────────────────────────────────────

pub struct TestSuite {
    pub server: TestServer,
    records: Vec<EndpointRecord>,
    cases: Vec<StoredCase>,
    current_group: String,
}

impl TestSuite {
    pub async fn new() -> Self {
        let server = TestServer::new(
            server::app(DbInitFlags::NUKE)
                .await
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .expect("Error creating test server.");

        Self {
            server,
            records: Vec::new(),
            cases: Vec::new(),
            current_group: String::new(),
        }
    }

    pub fn begin_group(&mut self, name: &str) {
        self.current_group = name.to_string();
    }

    pub fn end_group(&mut self) {
        self.current_group = String::new();
    }

    pub fn case(&mut self, label: &str) -> CaseBuilder<'_> {
        CaseBuilder {
            suite: self,
            label: label.to_string(),
            method: Method::GET,
            path: "",
            body: None,
            query: None,
            token: None,
        }
    }

    fn record_endpoint(&mut self, method: Method, path: &'static str, body: Option<&Value>, query: Option<&Value>, needs_auth: bool) {
        if self.records.iter().any(|r| r.method == method && r.path == path) {
            return;
        }
        let data = if let Some(b) = body {
            Some(DataKind::Body(extract_shape(b)))
        } else {
            query.map(|q| DataKind::Query(extract_shape(q)))
        };
        self.records.push(EndpointRecord {
            method,
            path,
            data,
            needs_auth,
        });
    }

    fn store_case(&mut self, label: String, method: Method, path: &'static str, body: Option<Value>, query: Option<Value>, token: Option<String>) {
        self.cases.push(StoredCase {
            label,
            group: self.current_group.clone(),
            method,
            path,
            body,
            query,
            token,
        });
    }

    /// Reset the server with a fresh database for the next phase.
    pub async fn reset(&mut self) {
        self.server = TestServer::new(
            server::app(DbInitFlags::NUKE)
                .await
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .expect("Error creating test server.");
    }

    /// Phase 2: replay groups in random order, cases within each group in original order.
    /// Starts with a fresh database. Re-authenticates as admin so stored authed
    /// requests use a valid token. Asserts no 500s.
    pub async fn run_shuffled(&mut self, seed: u64) {
        use fake::rand::rngs::StdRng;
        use fake::rand::SeedableRng;
        use fake::rand::seq::SliceRandom;

        self.reset().await;

        let admin_token = fuzz::login_admin(&self.server).await;

        // Collect unique group names in insertion order.
        let mut group_order: Vec<&str> = Vec::new();
        for c in &self.cases {
            if !group_order.contains(&c.group.as_str()) {
                group_order.push(&c.group);
            }
        }

        let mut rng = StdRng::seed_from_u64(seed);
        group_order.shuffle(&mut rng);

        eprintln!(
            "── phase 2: shuffled replay ({} groups, {} cases, seed {seed}) ──",
            group_order.len(),
            self.cases.len(),
        );

        for group in &group_order {
            for c in self.cases.iter().filter(|c| c.group == *group) {
                // Use fresh admin token for authed requests since the DB was nuked.
                let token = c.token.as_ref().map(|_| admin_token.as_str());
                let resp = send_request(&self.server, &c.method, c.path, c.body.as_ref(), c.query.as_ref(), token).await;
                assert!(
                    resp.status_code().as_u16() < 500,
                    "[{group}] case {:?} returned {} on {} {}",
                    c.label, resp.status_code(), c.method, c.path,
                );
            }
        }
    }

    /// Phase 3: fuzz all recorded endpoints with fake data for the given duration.
    /// Starts with a fresh database.
    pub async fn fuzz(&mut self, seed: u64, duration: Duration) {
        self.reset().await;
        fuzz::run_fuzz(&self.server, &self.records, seed, duration).await;
    }

    /// Number of collected cases.
    pub fn case_count(&self) -> usize {
        self.cases.len()
    }

    /// Extract collected cases for use in proptest simulation.
    pub fn take_cases(&mut self) -> Vec<StoredCase> {
        self.cases.clone()
    }
}

// ── Request helper ──────────────────────────────────────────────────────

async fn send_request(
    server: &TestServer,
    method: &Method,
    path: &str,
    body: Option<&Value>,
    query: Option<&Value>,
    token: Option<&str>,
) -> TestResponse {
    let mut req = server.method(method.clone(), path);
    if let Some(t) = token {
        req = req.add_header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {t}")).unwrap(),
        );
    }
    if let Some(q) = query {
        req = req.add_query_params(q);
    }
    match body {
        Some(b) => req.json(b).await,
        None => req.await,
    }
}

// ── CaseBuilder ─────────────────────────────────────────────────────────

#[allow(unused)]
pub struct CaseBuilder<'a> {
    suite: &'a mut TestSuite,
    label: String,
    method: Method,
    path: &'static str,
    body: Option<Value>,
    query: Option<Value>,
    token: Option<String>,
}

impl<'a> CaseBuilder<'a> {
    pub fn post(mut self, path: &'static str) -> Self {
        self.method = Method::POST;
        self.path = path;
        self
    }

    pub fn get(mut self, path: &'static str) -> Self {
        self.method = Method::GET;
        self.path = path;
        self
    }

    pub fn put(mut self, path: &'static str) -> Self {
        self.method = Method::PUT;
        self.path = path;
        self
    }

    #[allow(dead_code)]
    pub fn delete(mut self, path: &'static str) -> Self {
        self.method = Method::DELETE;
        self.path = path;
        self
    }

    pub fn body<T: Serialize>(mut self, body: T) -> Self {
        self.body = Some(serde_json::to_value(body).unwrap());
        self
    }

    pub fn query<T: Serialize>(mut self, params: T) -> Self {
        self.query = Some(serde_json::to_value(params).unwrap());
        self
    }

    pub fn token(mut self, token: &str) -> Self {
        self.token = Some(token.to_string());
        self
    }

    /// Send the request, assert with the check lambda, return the response.
    pub async fn expect(self, check: impl FnOnce(&TestResponse)) -> TestResponse {
        let CaseBuilder { suite, label, method, path, body, query, token } = self;

        suite.record_endpoint(method.clone(), path, body.as_ref(), query.as_ref(), token.is_some());
        suite.store_case(label, method.clone(), path, body.clone(), query.clone(), token.clone());

        let response = send_request(&suite.server, &method, path, body.as_ref(), query.as_ref(), token.as_deref()).await;
        check(&response);
        return response;
    }
}

// ── Proptest replay ─────────────────────────────────────────────────────

/// Replay a random sequence of previously-collected cases against a fresh server.
/// Each case that originally had a token gets the admin token substituted.
/// Asserts no 500s — individual cases may legitimately get 4xx when replayed
/// out of their original order.
pub async fn replay_random(cases: &[StoredCase], indices: &[usize]) {
    use std::net::SocketAddr;

    let server = TestServer::new(
        server::app(DbInitFlags::NUKE)
            .await
            .into_make_service_with_connect_info::<SocketAddr>(),
    )
    .expect("Error creating test server.");

    let admin_token = fuzz::login_admin(&server).await;

    for &idx in indices {
        let c = &cases[idx];
        let token = c.token.as_ref().map(|_| admin_token.as_str());
        let resp = send_request(&server, &c.method, c.path, c.body.as_ref(), c.query.as_ref(), token).await;
        assert!(
            resp.status_code().as_u16() < 500,
            "case {:?} returned {} on {} {}",
            c.label,
            resp.status_code(),
            c.method,
            c.path,
        );
    }
}
