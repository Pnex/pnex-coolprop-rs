//! Machine-checked guarantee that the API covers **every** function exported
//! by the vendored CoolProp C header:
//!
//! 1. every `EXPORT_CODE ... CONVENTION <name>(` symbol in CoolPropLib.h
//!    appears in the coverage table (`src/coverage.rs`), and vice versa;
//! 2. every route in the coverage table exists in the generated OpenAPI spec
//!    with the right method — so the OpenAPI spec covers 100% of the C API;
//! 3. every path in the OpenAPI spec is actually served by the router.

use std::collections::BTreeSet;

use axum::body::Body;
use axum::http::Request;
use http_body_util::BodyExt;
use tower::ServiceExt;

use coolprop_server::coverage::COOLPROP_SYMBOL_TO_ROUTE;
use coolprop_server::router;

/// Extract `(symbol, route)` pairs from every line of the vendored header
/// that starts a declaration: `EXPORT_CODE <ret> CONVENTION <name>(`.
fn exported_symbols(header: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in header.lines() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("EXPORT_CODE ") else {
            continue;
        };
        let Some(after) = rest.split_once("CONVENTION") else {
            continue;
        };
        let name = after
            .1
            .trim_start()
            .split('(')
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        if !name.is_empty() {
            out.insert(name);
        }
    }
    out
}

#[tokio::test]
async fn every_coolprop_export_is_mapped_to_a_route() {
    let header_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/CoolProp/include/CoolProp/CoolPropLib.h");
    let header = std::fs::read_to_string(&header_path)
        .unwrap_or_else(|e| panic!("could not read {}: {e}", header_path.display()));

    let exported = exported_symbols(&header);
    let mapped: BTreeSet<_> = COOLPROP_SYMBOL_TO_ROUTE
        .iter()
        .map(|(s, _)| s.to_string())
        .collect();

    assert_eq!(
        exported,
        mapped,
        "coverage table and CoolPropLib.h exports differ\n\
         missing from table: {:?}\n\
         extra in table: {:?}",
        exported.difference(&mapped).collect::<Vec<_>>(),
        mapped.difference(&exported).collect::<Vec<_>>(),
    );
}

#[tokio::test]
async fn every_mapped_route_exists_in_the_openapi_spec() {
    let spec = coolprop_server::openapi_spec();
    let paths = spec.paths.clone();

    for (symbol, route) in COOLPROP_SYMBOL_TO_ROUTE {
        let (method, path) = route.split_once(' ').expect("METHOD /path");
        let Some(item) = paths.paths.get(path) else {
            panic!("route {route:?} (for {symbol}) is missing from the OpenAPI spec");
        };
        let op = match method {
            "GET" => item.get.as_ref(),
            "POST" => item.post.as_ref(),
            "PUT" => item.put.as_ref(),
            "DELETE" => item.delete.as_ref(),
            m => panic!("unsupported method {m:?} for {symbol}"),
        };
        assert!(
            op.is_some(),
            "{method} {path} (for {symbol}) missing from spec"
        );
    }
}

#[tokio::test]
async fn every_openapi_path_is_served_by_the_router() {
    let spec = coolprop_server::openapi_spec();
    let paths = &spec.paths.paths;

    assert!(
        paths.len() >= COOLPROP_SYMBOL_TO_ROUTE.len(),
        "spec has {} paths but the coverage table has {} entries",
        paths.len(),
        COOLPROP_SYMBOL_TO_ROUTE.len(),
    );

    for (path, item) in paths {
        // Substitute path parameters with harmless values.
        let concrete = replace_params(path);
        for (method, _) in operations(item) {
            let response = router()
                .oneshot(
                    Request::builder()
                        .method(method.clone())
                        .uri(&concrete)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            // Unmatched routes yield an *empty* 404 body; matched routes
            // always answer with a body (even error handlers). A handler 404
            // (e.g. unknown AbstractState handle) is fine — the route exists.
            let status = response.status();
            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert!(
                !body.is_empty(),
                "{method} {path} is in the spec but the router returned an empty body \
                 (route not registered?) — status {status}",
            );
        }
    }
}

fn replace_params(path: &str) -> String {
    // {handle}/{i} → 1, {param}/{fluid} → T (a valid parameter name).
    path.replace("{handle}", "1")
        .replace("{i}", "0")
        .replace("{param}", "T")
        .replace("{fluid}", "Water")
}

fn operations(item: &utoipa::openapi::PathItem) -> Vec<(axum::http::Method, ())> {
    let mut ops = Vec::new();
    if item.get.is_some() {
        ops.push((axum::http::Method::GET, ()));
    }
    if item.post.is_some() {
        ops.push((axum::http::Method::POST, ()));
    }
    if item.put.is_some() {
        ops.push((axum::http::Method::PUT, ()));
    }
    if item.delete.is_some() {
        ops.push((axum::http::Method::DELETE, ()));
    }
    ops
}
