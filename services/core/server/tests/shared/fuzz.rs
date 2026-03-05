#![allow(clippy::needless_return)]
//! Shape-based fuzz testing. Records endpoint shapes from test cases,
//! then generates fake data matching those shapes to find crashes.

use std::time::{Duration, Instant};

use axum_test::TestServer;
use config::ServerConfig;
use fake::Fake;
use fake::faker::internet::en::{Password, Username};
use fake::faker::lorem::en::{Sentence, Word};
use fake::faker::name::en::Name;
use fake::rand::rngs::StdRng;
use fake::rand::SeedableRng;
use http::{HeaderName, HeaderValue, Method};
use serde_json::{json, Value};
use uuid::Uuid;

use server::endpoints::auth_endpoints::{LoginInfo, LOGIN_ENDPOINT};

// ── Shape ───────────────────────────────────────────────────────────────

/// Describes the structure of a JSON value for fake data generation.
#[derive(Debug, Clone)]
pub enum Shape {
    Null,
    Bool,
    Number,
    String,
    Uuid,
    Array(Vec<Shape>),
    Object(Vec<(String, Shape)>),
}

/// What kind of data an endpoint accepts.
#[derive(Debug, Clone)]
pub enum DataKind {
    Body(Shape),
    Query(Shape),
}

/// A recorded endpoint from a test case.
#[derive(Debug, Clone)]
pub struct EndpointRecord {
    pub method: Method,
    pub path: &'static str,
    pub data: Option<DataKind>,
    pub needs_auth: bool,
}

pub fn extract_shape(value: &Value) -> Shape {
    match value {
        Value::Null => Shape::Null,
        Value::Bool(_) => Shape::Bool,
        Value::Number(_) => Shape::Number,
        Value::String(s) => {
            if Uuid::parse_str(s).is_ok() {
                Shape::Uuid
            } else {
                Shape::String
            }
        }
        Value::Array(arr) => Shape::Array(arr.iter().map(extract_shape).collect()),
        Value::Object(obj) => {
            Shape::Object(obj.iter().map(|(k, v)| (k.clone(), extract_shape(v))).collect())
        }
    }
}

// ── Fake generation ─────────────────────────────────────────────────────

const EDGE_STRINGS: &[&str] = &[
    "",
    " ",
    "\t\n\r",
    "\0",
    "🎉🔥💀🇺🇸",
    "'; DROP TABLE users; --",
    "<script>alert(1)</script>",
    "null",
    "undefined",
    "true",
    "false",
    "NaN",
    "-1",
    "../../../etc/passwd",
    "{\"nested\": true}",
];

fn fake_from_shape(shape: &Shape, rng: &mut StdRng) -> Value {
    match shape {
        Shape::Null => {
            if (0u8..3u8).fake_with_rng::<u8, _>(rng) == 0 {
                json!(null)
            } else {
                json!(fake_string(rng))
            }
        }
        Shape::Bool => json!((0u8..2u8).fake_with_rng::<u8, _>(rng) == 1),
        Shape::Number => json!((i32::MIN..i32::MAX).fake_with_rng::<i32, _>(rng)),
        Shape::String => json!(fake_string(rng)),
        Shape::Uuid => json!(Uuid::new_v4().to_string()),
        Shape::Array(shapes) => {
            Value::Array(shapes.iter().map(|s| fake_from_shape(s, rng)).collect())
        }
        Shape::Object(fields) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in fields {
                obj.insert(k.clone(), fake_from_shape(v, rng));
            }
            Value::Object(obj)
        }
    }
}

fn edge_from_shape(shape: &Shape, edge: &str) -> Value {
    match shape {
        Shape::Object(fields) => {
            let mut obj = serde_json::Map::new();
            for (k, _) in fields {
                obj.insert(k.clone(), json!(edge));
            }
            Value::Object(obj)
        }
        _ => json!(edge),
    }
}

fn fake_string(rng: &mut StdRng) -> String {
    let choice: u8 = (0u8..6u8).fake_with_rng(rng);
    match choice {
        0 => Name().fake_with_rng(rng),
        1 => Username().fake_with_rng(rng),
        2 => Password(8..40).fake_with_rng(rng),
        3 => Sentence(1..5).fake_with_rng(rng),
        4 => Word().fake_with_rng(rng),
        _ => {
            let idx: usize = (0..EDGE_STRINGS.len()).fake_with_rng(rng);
            EDGE_STRINGS[idx].to_string()
        }
    }
}

// ── Fuzz runner ─────────────────────────────────────────────────────────

pub async fn run_fuzz(
    server: &TestServer,
    records: &[EndpointRecord],
    seed: u64,
    duration: Duration,
) {
    if records.is_empty() {
        return;
    }

    eprintln!(
        "fuzz seed: {seed}, duration: {duration:?}, endpoints: {}",
        records.len()
    );
    let mut rng = StdRng::seed_from_u64(seed);

    let admin_token = login_admin(server).await;
    let deadline = Instant::now() + duration;
    let mut count: u64 = 0;

    // Phase 1: edge cases + malformed payloads (finite)
    for record in records {
        let token = if record.needs_auth { Some(admin_token.as_str()) } else { None };

        // Malformed payloads
        let malformed = [
            json!(null), json!(true), json!(42), json!("x"),
            json!([1, 2, 3]), json!({}),
            json!({"completely": "wrong"}),
        ];
        for body in &malformed {
            no_crash(server, &record.method, record.path, Some(body.clone()), token).await;
        }

        // Edge string payloads
        if let Some(data) = &record.data {
            let shape = match data {
                DataKind::Body(s) | DataKind::Query(s) => s,
            };
            let long = "x".repeat(100_000);
            for edge in EDGE_STRINGS.iter().copied().chain(std::iter::once(long.as_str())) {
                let body = edge_from_shape(shape, edge);
                no_crash(server, &record.method, record.path, Some(body), token).await;
            }
        }
    }

    // Phase 2: random payloads until deadline
    while Instant::now() < deadline {
        let idx = (0..records.len()).fake_with_rng::<usize, _>(&mut rng);
        let record = &records[idx];
        let token = if record.needs_auth { Some(admin_token.as_str()) } else { None };

        let body = record.data.as_ref().map(|data| {
            let shape = match data {
                DataKind::Body(s) | DataKind::Query(s) => s,
            };
            fake_from_shape(shape, &mut rng)
        });

        no_crash(server, &record.method, record.path, body, token).await;
        count += 1;
    }
    eprintln!("fuzz done: {count} random iterations");
}

// ── Helpers ─────────────────────────────────────────────────────────────

pub async fn login_admin(server: &TestServer) -> String {
    let config = ServerConfig::get();
    let resp = server
        .post(LOGIN_ENDPOINT)
        .json(&LoginInfo {
            name: config.default_user_name.clone(),
            password: config.default_user_password.clone(),
            otp: None,
        })
        .await;
    assert_eq!(resp.status_code().as_u16(), 200, "admin login must succeed for fuzz");
    let body: Value = resp.json();
    body["access_token"].as_str().unwrap().to_string()
}

async fn no_crash(
    server: &TestServer,
    method: &Method,
    path: &str,
    body: Option<Value>,
    token: Option<&str>,
) {
    let mut req = server.method(method.clone(), path);
    if let Some(t) = token {
        req = req.add_header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {}", t)).unwrap(),
        );
    }
    let response = match body {
        Some(ref b) => req.json(b).await,
        None => req.await,
    };
    let status = response.status_code().as_u16();
    assert!(
        status < 500,
        "CRASH at {} {}: status {} — body: {:?}",
        method, path, status, body,
    );
}
