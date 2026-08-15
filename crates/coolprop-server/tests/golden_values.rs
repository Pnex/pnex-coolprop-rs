//! Golden-value tests: known physics, cross-checked between independent
//! endpoints (scalar vs multi vs FORTRAN shim vs AbstractState).

mod common;

use axum::http::{Method, StatusCode};
use common::json_req;
use serde_json::json;

/// Water's normal boiling point from the IAPWS-95 EOS as used by CoolProp.
const WATER_NBP_K: f64 = 373.12429584768442;

#[tokio::test]
async fn water_normal_boiling_point() {
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/props/si",
        json!({"output": "T", "name1": "P", "prop1": 101325.0,
               "name2": "Q", "prop2": 0.0, "fluid": "Water"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let t = body["value"].as_f64().unwrap();
    assert!((t - WATER_NBP_K).abs() < 1e-6, "NBP = {t}");
}

#[tokio::test]
async fn water_molar_mass() {
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/props/1si",
        json!({"fluid": "Water", "output": "molar_mass"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let m = body["value"].as_f64().unwrap();
    assert!((m - 0.01801528).abs() < 1e-7, "molar mass = {m}");
}

/// The user's example call, consistent across every code path that can
/// compute it: PropsSI (default and HEOS-prefixed), PropsSImulti, and the
/// FORTRAN shim.
#[tokio::test]
async fn mixture_example_consistent_across_endpoints() {
    let body = json!({"output": "Dmolar", "name1": "T", "prop1": 298.0,
                      "name2": "P", "prop2": 1e5, "fluid": "Propane[0.5]&Ethane[0.5]"});

    let (s, b) = json_req(Method::POST, "/api/v1/props/si", body.clone()).await;
    assert_eq!(s, StatusCode::OK);
    let scalar = b["value"].as_f64().unwrap();
    assert!(scalar.is_finite() && scalar > 0.0);

    let mut heos_body = body.clone();
    heos_body["fluid"] = json!("HEOS::Propane[0.5]&Ethane[0.5]");
    let (s, b) = json_req(Method::POST, "/api/v1/props/si", heos_body).await;
    assert_eq!(s, StatusCode::OK);
    assert!((b["value"].as_f64().unwrap() - scalar).abs() < 1e-9);

    let (s, b) = json_req(Method::POST, "/api/v1/fortran/propssi", body).await;
    assert_eq!(s, StatusCode::OK);
    assert!((b["value"].as_f64().unwrap() - scalar).abs() < 1e-9);

    let (s, b) = json_req(
        Method::POST,
        "/api/v1/props/si-multi",
        json!({"outputs": ["Dmolar"], "name1": "T", "prop1": [298.0],
               "name2": "P", "prop2": [1e5], "fluids": ["Propane", "Ethane"],
               "fractions": [0.5, 0.5]}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!((b["values"][0][0].as_f64().unwrap() - scalar).abs() < 1e-6 * scalar.abs());
}

/// AbstractState PT update must agree with PropsSI for the same state.
#[tokio::test]
async fn abstract_state_agrees_with_props_si() {
    let (s, b) = json_req(
        Method::POST,
        "/api/v1/abstract-state",
        json!({"backend": "HEOS", "fluids": ["Water"]}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "body: {b}");
    let handle = b["handle"].as_i64().unwrap();

    let (s, b) = json_req(
        Method::POST,
        &format!("/api/v1/abstract-state/{handle}/update"),
        json!({"input_pair": "PT_INPUTS", "value1": 101325.0, "value2": 400.0}),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "body: {b}");

    let (s, b) = req_get(&format!(
        "/api/v1/abstract-state/{handle}/keyed-output/Dmolar"
    ))
    .await;
    assert_eq!(s, StatusCode::OK);
    let as_value = b["value"].as_f64().unwrap();

    let (s, b) = json_req(
        Method::POST,
        "/api/v1/props/si",
        json!({"output": "Dmolar", "name1": "T", "prop1": 400.0,
               "name2": "P", "prop2": 101325.0, "fluid": "Water"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let props_value = b["value"].as_f64().unwrap();

    assert!(
        (as_value - props_value).abs() < 1e-6 * props_value.abs(),
        "AbstractState {as_value} vs PropsSI {props_value}",
    );

    let (s, _) = req_delete(&format!("/api/v1/abstract-state/{handle}")).await;
    assert_eq!(s, StatusCode::OK);
}

/// Saturation ancillary approximates the EOS boiling pressure.
#[tokio::test]
async fn saturation_ancillary_close_to_eos() {
    let (s, b) = req_get(
        "/api/v1/misc/saturation-ancillary?fluid=Water&output=P&q=0&input=T&value=373.12429584768442",
    )
    .await;
    assert_eq!(s, StatusCode::OK, "body: {b}");
    let ancillary = b["value"].as_f64().unwrap();

    let (s, b) = json_req(
        Method::POST,
        "/api/v1/props/si",
        json!({"output": "P", "name1": "T", "prop1": WATER_NBP_K,
               "name2": "Q", "prop2": 0.0, "fluid": "Water"}),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let eos = b["value"].as_f64().unwrap();

    assert!(
        (ancillary - eos).abs() / eos < 0.01,
        "ancillary {ancillary} vs EOS {eos}",
    );
}

/// get_parameter_information_string describes known parameters.
#[tokio::test]
async fn parameter_info_descriptions() {
    for (param, fragment) in [
        ("T", "Temperature"),
        ("P", "Pressure"),
        ("Dmolar", "density"),
    ] {
        let (s, b) = req_get(&format!("/api/v1/params/information/{param}")).await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        let desc = b["value"].as_str().unwrap().to_lowercase();
        assert!(desc.contains(&fragment.to_lowercase()), "{param}: {desc}");
    }
}

async fn req_get(uri: &str) -> (StatusCode, serde_json::Value) {
    common::req(Method::GET, uri).await
}

async fn req_delete(uri: &str) -> (StatusCode, serde_json::Value) {
    common::req(Method::DELETE, uri).await
}
