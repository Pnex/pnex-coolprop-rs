//! Integration tests: every endpoint exercised through the axum router
//! (in-process `tower::ServiceExt::oneshot`, no network).

mod common;

use axum::http::{Method, StatusCode};
use common::{json_req, req, text_req};
use serde_json::json;

// ----------------------------------------------------------------------
// Health / docs
// ----------------------------------------------------------------------

#[tokio::test]
async fn health() {
    let (status, body) = req(Method::GET, "/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn openapi_json_served() {
    let (status, body) = req(Method::GET, "/openapi.json").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["paths"].as_object().unwrap().len() >= 71);
}

#[tokio::test]
async fn swagger_ui_served() {
    let (status, body) = text_req(Method::GET, "/swagger-ui/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("swagger"), "unexpected body: {body}");
}

// ----------------------------------------------------------------------
// Props family
// ----------------------------------------------------------------------

const MIXTURE: &str = "Propane[0.5]&Ethane[0.5]";

async fn dmolar_mixture(fluid: &str) -> f64 {
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/props/si",
        json!({"output": "Dmolar", "name1": "T", "prop1": 298.0,
               "name2": "P", "prop2": 1e5, "fluid": fluid}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    body["value"].as_f64().unwrap()
}

/// The exact call from the user's C++ example.
#[tokio::test]
async fn props_si_mixture_example() {
    let v = dmolar_mixture(MIXTURE).await;
    assert!(v.is_finite() && v > 0.0, "got {v}");
}

#[tokio::test]
async fn props_si_heos_prefix_matches_default_backend() {
    let a = dmolar_mixture(MIXTURE).await;
    let b = dmolar_mixture(&format!("HEOS::{MIXTURE}")).await;
    assert!((a - b).abs() < 1e-9 * a.abs(), "{a} vs {b}");
}

#[tokio::test]
async fn props_si_error_is_400_with_coolprop_message() {
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/props/si",
        json!({"output": "Bogus", "name1": "T", "prop1": 298.0,
               "name2": "P", "prop2": 1e5, "fluid": "Water"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert!(!body["error"]["message"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn props_si_refprop_backend_missing_fails_gracefully() {
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/props/si",
        json!({"output": "Dmolar", "name1": "T", "prop1": 298.0,
               "name2": "P", "prop2": 1e5, "fluid": "REFPROP::Water"}),
    )
    .await;
    // REFPROP is a commercial library; when it is not installed CoolProp
    // must reject the call with a clear message rather than crash.
    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    let msg = body["error"]["message"].as_str().unwrap();
    assert!(!msg.is_empty());
}

#[tokio::test]
async fn props_1si_critical_data() {
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/props/1si",
        json!({"fluid": "Water", "output": "Tcrit"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let t = body["value"].as_f64().unwrap();
    assert!((t - 647.096).abs() < 0.1, "Tcrit = {t}");
}

#[tokio::test]
async fn props_1si_error() {
    let (status, _) = json_req(
        Method::POST,
        "/api/v1/props/1si",
        json!({"fluid": "NotAFluid", "output": "Tcrit"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn props_si_multi_matches_scalar() {
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/props/si-multi",
        json!({
            "outputs": ["Dmolar", "Hmolar"],
            "name1": "T", "prop1": [298.0, 300.0],
            "name2": "P", "prop2": [1e5, 1e5],
            "backend": "HEOS",
            "fluids": ["Propane", "Ethane"],
            "fractions": [0.5, 0.5]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let values = body["values"].as_array().unwrap();
    assert_eq!(values.len(), 2, "one row per input point");
    assert_eq!(values[0].as_array().unwrap().len(), 2, "one col per output");
    let expected = dmolar_mixture(MIXTURE).await;
    let got = values[0][0].as_f64().unwrap();
    assert!(
        (got - expected).abs() < 1e-6 * expected.abs(),
        "multi {got} vs scalar {expected}",
    );
}

#[tokio::test]
async fn props_si_multi_orientation_non_square() {
    // 2 input points x 3 outputs locks the [points][outputs] orientation.
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/props/si-multi",
        json!({
            "outputs": ["Dmolar", "Hmolar", "Smolar"],
            "name1": "T", "prop1": [298.0, 300.0],
            "name2": "P", "prop2": [1e5, 1e5],
            "fluids": ["Propane", "Ethane"],
            "fractions": [0.5, 0.5]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["shape"], json!([2, 3]));
    let values = body["values"].as_array().unwrap();
    assert_eq!(values.len(), 2);
    assert_eq!(values[0].as_array().unwrap().len(), 3);
    // Column 0 is Dmolar: matches the scalar call for the same point.
    let expected = dmolar_mixture(MIXTURE).await;
    let got = values[0][0].as_f64().unwrap();
    assert!((got - expected).abs() < 1e-6 * expected.abs());
}

#[tokio::test]
async fn props_si_multi_default_fractions_and_backend() {
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/props/si-multi",
        json!({
            "outputs": ["Dmolar"],
            "name1": "T", "prop1": [298.0],
            "name2": "P", "prop2": [1e5],
            "fluids": ["Propane", "Ethane"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let expected = dmolar_mixture(MIXTURE).await;
    let got = body["values"][0][0].as_f64().unwrap();
    assert!((got - expected).abs() < 1e-6 * expected.abs());
}

#[tokio::test]
async fn props_si_multi_length_mismatch_is_400() {
    let (status, _) = json_req(
        Method::POST,
        "/api/v1/props/si-multi",
        json!({
            "outputs": ["Dmolar"],
            "name1": "T", "prop1": [298.0, 300.0],
            "name2": "P", "prop2": [1e5],
            "fluids": ["Propane"]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn props_1si_multi() {
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/props/1si-multi",
        json!({
            "outputs": ["Tcrit"],
            "backend": "HEOS",
            "fluids": ["Propane", "Ethane"],
            "fractions": [0.5, 0.5]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let values = body["values"].as_array().unwrap();
    assert!(!values.is_empty());
    assert!(values.iter().all(|v| v.as_f64().unwrap().is_finite()));
}

#[tokio::test]
async fn phase_si_liquid_and_error() {
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/props/phase",
        json!({"output": "", "name1": "T", "prop1": 300.0,
               "name2": "P", "prop2": 101325.0, "fluid": "Water"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["value"], "liquid");

    let (status, _) = json_req(
        Method::POST,
        "/api/v1/props/phase",
        json!({"output": "", "name1": "T", "prop1": 300.0,
               "name2": "P", "prop2": 101325.0, "fluid": "NotAFluid"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn legacy_props_family() {
    // Props (char input names, KSI units)
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/props/legacy",
        json!({"output": "D", "name1": "T", "prop1": 300.0,
               "name2": "P", "prop2": 101.325, "ref": "Water"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["value"].as_f64().unwrap().is_finite());

    // PropsS (string input names)
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/props/legacy-s",
        json!({"output": "D", "name1": "T", "prop1": 300.0,
               "name2": "P", "prop2": 101.325, "ref": "Water"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["value"].as_f64().unwrap().is_finite());

    // Props1: CoolProp v8's kSI conversion rejects every output that
    // Props1SI can produce (deprecated upstream, effectively unusable), so
    // the faithful behavior is a clean 400 with CoolProp's message.
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/props/legacy/1",
        json!({"fluid": "Water", "output": "Tcrit"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert!(!body["error"]["message"].as_str().unwrap().is_empty());

    // Legacy Props rejects multi-char input names.
    let (status, _) = json_req(
        Method::POST,
        "/api/v1/props/legacy",
        json!({"output": "D", "name1": "TT", "prop1": 300.0,
               "name2": "P", "prop2": 101.325, "ref": "Water"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ----------------------------------------------------------------------
// Humid air
// ----------------------------------------------------------------------

#[tokio::test]
async fn ha_props_si_humidity_ratio() {
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/ha/props-si",
        json!({"output": "W", "name1": "T", "prop1": 298.15,
               "name2": "P", "prop2": 101325.0,
               "name3": "R", "prop3": 0.5}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let w = body["value"].as_f64().unwrap();
    assert!(w > 0.0 && w < 0.1, "humidity ratio {w} out of range");
}

#[tokio::test]
async fn ha_props_si_error() {
    let (status, _) = json_req(
        Method::POST,
        "/api/v1/ha/props-si",
        json!({"output": "Bogus", "name1": "T", "prop1": 298.15,
               "name2": "P", "prop2": 101325.0,
               "name3": "R", "prop3": 0.5}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn ha_props_legacy() {
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/ha/props",
        json!({"output": "W", "name1": "T", "prop1": 298.15,
               "name2": "P", "prop2": 101.325,
               "name3": "R", "prop3": 0.5}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let w = body["value"].as_f64().unwrap();
    assert!(w > 0.0 && w < 0.1, "legacy humidity ratio {w} out of range");
}

#[tokio::test]
async fn cair_sat() {
    let (status, body) = json_req(Method::POST, "/api/v1/ha/cair-sat", json!({"t": 300.0})).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    // cair_sat returns kJ/kg/K (EES correlation, steep near 300 K).
    let cp = body["value"].as_f64().unwrap();
    assert!(cp > 4.0 && cp < 5.0, "cair_sat(300) = {cp} kJ/kg/K");
}

// ----------------------------------------------------------------------
// FORTRAN shims
// ----------------------------------------------------------------------

#[tokio::test]
async fn fortran_shims_match_their_counterparts() {
    let si_body = json!({"output": "Dmolar", "name1": "T", "prop1": 298.0,
                         "name2": "P", "prop2": 1e5, "fluid": "Water"});
    let (s1, b1) = json_req(Method::POST, "/api/v1/props/si", si_body.clone()).await;
    let (s2, b2) = json_req(Method::POST, "/api/v1/fortran/propssi", si_body).await;
    assert_eq!(s1, StatusCode::OK);
    assert_eq!(s2, StatusCode::OK, "body: {b2}");
    let (a, b) = (b1["value"].as_f64().unwrap(), b2["value"].as_f64().unwrap());
    assert!((a - b).abs() < 1e-9, "propssi_ {b} vs PropsSI {a}");

    let ha_body = json!({"output": "W", "name1": "T", "prop1": 298.15,
                         "name2": "P", "prop2": 101325.0,
                         "name3": "R", "prop3": 0.5});
    let (s, body) = json_req(Method::POST, "/api/v1/fortran/hapropssi", ha_body.clone()).await;
    assert_eq!(s, StatusCode::OK, "body: {body}");
    assert!(body["value"].as_f64().unwrap() > 0.0);

    let ha_body = json!({"output": "W", "name1": "T", "prop1": 298.15,
                         "name2": "P", "prop2": 101.325,
                         "name3": "R", "prop3": 0.5});
    let (s, body) = json_req(Method::POST, "/api/v1/fortran/haprops", ha_body).await;
    assert_eq!(s, StatusCode::OK, "body: {body}");
    assert!(body["value"].as_f64().unwrap() > 0.0);
}

// ----------------------------------------------------------------------
// Info / misc
// ----------------------------------------------------------------------

#[tokio::test]
async fn global_param_string_version() {
    let (status, body) = req(Method::GET, "/api/v1/params/global/version").await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(!body["value"].as_str().unwrap().is_empty());

    let (status, _) = req(Method::GET, "/api/v1/params/global/bogus_param").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn parameter_information_string() {
    let (status, body) = req(Method::GET, "/api/v1/params/information/T").await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["value"].as_str().unwrap().contains("Temperature"));

    let (status, _) = req(Method::GET, "/api/v1/params/information/Bogus").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn fluids_list_endpoint() {
    let (status, body) = req(Method::GET, "/api/v1/fluids").await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let fluids = body["values"].as_array().unwrap();
    // CoolProp v8 ships >100 EOS fluids.
    assert!(fluids.len() > 100, "got {} fluids", fluids.len());
    assert!(fluids.iter().any(|f| f == "Water"));
    assert!(fluids.iter().any(|f| f == "R134a"));
    // Parsed entries, not the raw comma-joined string.
    assert!(fluids
        .iter()
        .all(|f| !f.as_str().unwrap().is_empty() && !f.as_str().unwrap().contains(',')));
}

#[tokio::test]
async fn fluid_param_string_and_len() {
    let (status, body) = req(Method::GET, "/api/v1/fluids/Water/param/aliases").await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["value"]
        .as_str()
        .unwrap()
        .to_lowercase()
        .contains("water"));

    let (status, body) = req(Method::GET, "/api/v1/fluids/Water/param/aliases/length").await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["length"].as_i64().unwrap() > 0);

    let (status, _) = req(Method::GET, "/api/v1/fluids/NotAFluid/param/aliases").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn param_and_input_pair_indices() {
    let (status, body) = req(Method::GET, "/api/v1/params/index?name=T").await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["index"].as_i64().unwrap() >= 0);

    let (status, _) = req(Method::GET, "/api/v1/params/index?name=Bogus").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, body) = req(Method::GET, "/api/v1/input-pairs/index?name=PT_INPUTS").await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body["index"].as_i64().unwrap() >= 0);

    let (status, _) = req(Method::GET, "/api/v1/input-pairs/index?name=BOGUS_INPUTS").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn fluid_validity_and_backend_extraction() {
    let (status, body) = req(Method::GET, "/api/v1/fluids/is-valid?name=Water").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["value"], true);

    let (status, body) = req(Method::GET, "/api/v1/fluids/is-valid?name=NotAFluidZZZ").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["value"], false);

    let (status, body) = json_req(
        Method::POST,
        "/api/v1/fluids/extract-backend",
        json!({"fluid_string": "REFPROP::Water[0.5]&Ethane[0.5]"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["backend"], "REFPROP");
    assert_eq!(body["fluid"], "Water[0.5]&Ethane[0.5]");

    let (status, body) = json_req(
        Method::POST,
        "/api/v1/fluids/extract-backend",
        json!({"fluid_string": "HEOS::R134a"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["backend"], "HEOS");
    assert_eq!(body["fluid"], "R134a");
}

#[tokio::test]
async fn add_fluids_as_json_rejects_garbage() {
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/fluids/add-json",
        json!({"backend": "HEOS", "fluid_string": "this is not JSON"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert!(!body["error"]["message"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn temperature_conversions() {
    let (status, body) = req(Method::GET, "/api/v1/misc/f2k?t=32").await;
    assert_eq!(status, StatusCode::OK);
    assert!((body["value"].as_f64().unwrap() - 273.15).abs() < 1e-9);

    let (status, body) = req(Method::GET, "/api/v1/misc/k2f?t=273.15").await;
    assert_eq!(status, StatusCode::OK);
    assert!((body["value"].as_f64().unwrap() - 32.0).abs() < 1e-9);
}

#[tokio::test]
async fn saturation_ancillary_water_boiling() {
    let (status, body) = req(
        Method::GET,
        "/api/v1/misc/saturation-ancillary?fluid=Water&output=P&q=1&input=T&value=373.12429584768442",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let p = body["value"].as_f64().unwrap();
    // Ancillary curves are approximations of the full EOS.
    assert!(
        (p - 101325.0).abs() / 101325.0 < 0.01,
        "P_sat(373.124K) = {p}"
    );

    let (status, _) = req(
        Method::GET,
        "/api/v1/misc/saturation-ancillary?fluid=NotAFluid&output=P&q=1&input=T&value=300",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ----------------------------------------------------------------------
// Config / admin
// ----------------------------------------------------------------------

#[tokio::test]
async fn config_setters() {
    // Round-trip a harmless value (the default delimiter is ",").
    let (status, body) = json_req(
        Method::PUT,
        "/api/v1/config/string",
        json!({"key": "LIST_STRING_DELIMITER", "val": ","}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");

    let (status, body) = json_req(
        Method::PUT,
        "/api/v1/config/double",
        json!({"key": "R_U_CODATA", "val": 8.314462618}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");

    let (status, body) = json_req(
        Method::PUT,
        "/api/v1/config/bool",
        json!({"key": "ENABLE_SUPERANCILLARIES", "val": true}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");

    // Unknown keys are rejected with CoolProp's message.
    let (status, body) = json_req(
        Method::PUT,
        "/api/v1/config/bool",
        json!({"key": "NOT_A_REAL_KEY", "val": true}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
}

#[tokio::test]
async fn departure_functions_reject_garbage() {
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/config/departure-functions",
        json!({"string_data": "not valid departure function data"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
}

#[tokio::test]
async fn reference_states() {
    let (status, body) = json_req(
        Method::POST,
        "/api/v1/config/reference-state/S",
        json!({"fluid": "Water", "reference_state": "NBP"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");

    let (status, body) = json_req(
        Method::POST,
        "/api/v1/config/reference-state/S",
        json!({"fluid": "Water", "reference_state": "RESET"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");

    let (status, body) = json_req(
        Method::POST,
        "/api/v1/config/reference-state/D",
        json!({"fluid": "Water", "t": 300.0, "rhomolar": 55.0,
               "hmolar0": 0.0, "smolar0": 0.0}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");

    // Restore the default reference state for the other tests.
    let (status, _) = json_req(
        Method::POST,
        "/api/v1/config/reference-state/S",
        json!({"fluid": "Water", "reference_state": "RESET"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = json_req(
        Method::POST,
        "/api/v1/config/reference-state/S",
        json!({"fluid": "NotAFluid", "reference_state": "NBP"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn debug_level_roundtrip() {
    let (status, body) = req(Method::GET, "/api/v1/misc/debug-level").await;
    assert_eq!(status, StatusCode::OK);
    let original = body["index"].as_i64().unwrap();

    let (status, _) = json_req(Method::PUT, "/api/v1/misc/debug-level", json!({"level": 0})).await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = req(Method::GET, "/api/v1/misc/debug-level").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["index"].as_i64().unwrap(), 0);

    if original != 0 {
        let (status, _) = json_req(
            Method::PUT,
            "/api/v1/misc/debug-level",
            json!({"level": original}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }
}

#[tokio::test]
async fn redirect_stdout_roundtrip() {
    let (status, _) = json_req(
        Method::POST,
        "/api/v1/admin/redirect-stdout",
        json!({"file": "/tmp/coolprop-server-test-stdout.log"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = json_req(
        Method::POST,
        "/api/v1/admin/redirect-stdout",
        json!({"file": "/dev/stdout"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = json_req(
        Method::POST,
        "/api/v1/admin/redirect-stdout",
        json!({"file": "/no/such/dir/f.log"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ----------------------------------------------------------------------
// AbstractState
// ----------------------------------------------------------------------

mod abstract_state {
    use super::*;

    async fn create(backend: &str, fluids: serde_json::Value) -> (StatusCode, serde_json::Value) {
        json_req(
            Method::POST,
            "/api/v1/abstract-state",
            json!({"backend": backend, "fluids": fluids}),
        )
        .await
    }

    async fn new_mixture() -> i64 {
        let (status, body) = create("HEOS", json!(["Propane", "Ethane"])).await;
        assert_eq!(status, StatusCode::OK, "body: {body}");
        body["handle"].as_i64().unwrap()
    }

    async fn update(handle: i64, pair: &str, v1: f64, v2: f64) -> StatusCode {
        let (status, _) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/update"),
            json!({"input_pair": pair, "value1": v1, "value2": v2}),
        )
        .await;
        status
    }

    #[tokio::test]
    async fn unknown_handle_is_404() {
        let (status, body) = req(Method::GET, "/api/v1/abstract-state/999999/keyed-output/T").await;
        assert_eq!(status, StatusCode::NOT_FOUND, "body: {body}");
    }

    #[tokio::test]
    async fn factory_rejects_bad_fluid() {
        let (status, _) = create("HEOS", json!(["NotAFluidZZZ"])).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn empty_fluids_rejected() {
        let (status, body) = create("HEOS", json!([])).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    }

    #[tokio::test]
    async fn lifecycle_mixture() {
        let handle = new_mixture().await;

        // fluid names / backend name
        let (s, b) = req(
            Method::GET,
            &format!("/api/v1/abstract-state/{handle}/fluid-names"),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        // CoolProp canonicalizes "Propane" to "n-Propane".
        let names = b["values"].as_array().unwrap();
        assert_eq!(names.len(), 2);
        assert!(names
            .iter()
            .any(|n| n.as_str().unwrap().contains("Propane")));
        assert!(names.iter().any(|n| n.as_str().unwrap() == "Ethane"));

        let (s, b) = req(
            Method::GET,
            &format!("/api/v1/abstract-state/{handle}/backend-name"),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert!(!b["value"].as_str().unwrap().is_empty());

        // fractions
        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/fractions"),
            json!({"fractions": [0.5, 0.5]}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");

        let (s, b) = req(
            Method::GET,
            &format!("/api/v1/abstract-state/{handle}/mole-fractions"),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        for f in b["values"].as_array().unwrap() {
            assert!((f.as_f64().unwrap() - 0.5).abs() < 1e-12);
        }

        // homogeneous state + outputs
        assert_eq!(
            update(handle, "PT_INPUTS", 101325.0, 298.15).await,
            StatusCode::OK
        );

        let (s, b) = req(
            Method::GET,
            &format!("/api/v1/abstract-state/{handle}/keyed-output/T"),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert!((b["value"].as_f64().unwrap() - 298.15).abs() < 1e-9);

        // numeric index form
        let (s, idx) = req(Method::GET, "/api/v1/params/index?name=T").await;
        assert_eq!(s, StatusCode::OK);
        let t_index = idx["index"].as_i64().unwrap();
        let (s, b) = req(
            Method::GET,
            &format!("/api/v1/abstract-state/{handle}/keyed-output/{t_index}"),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert!((b["value"].as_f64().unwrap() - 298.15).abs() < 1e-9);

        let (s, b) = req(
            Method::GET,
            &format!("/api/v1/abstract-state/{handle}/keyed-output/Bogus"),
        )
        .await;
        assert_eq!(s, StatusCode::BAD_REQUEST, "body: {b}");

        // phase
        let (s, b) = req(
            Method::GET,
            &format!("/api/v1/abstract-state/{handle}/phase"),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert_eq!(b["phase"], "gas");

        // fugacities (mixture, homogeneous state)
        let (s, b) = req(
            Method::GET,
            &format!("/api/v1/abstract-state/{handle}/fugacity/0"),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert!(b["value"].as_f64().unwrap() > 0.0);

        let (s, b) = req(
            Method::GET,
            &format!("/api/v1/abstract-state/{handle}/fugacity-coefficient/1"),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert!(b["value"].as_f64().unwrap() > 0.0);

        // first partial derivative
        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/first-partial-deriv"),
            json!({"of": "Hmolar", "wrt": "P", "constant": "Dmolar"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert!(b["value"].as_f64().unwrap().is_finite());

        // array updates
        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/update-and-common-out"),
            json!({"input_pair": "PT_INPUTS", "value1": [101325.0, 101325.0],
                   "value2": [298.15, 308.15]}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert_eq!(b["t"].as_array().unwrap().len(), 2);
        assert!((b["t"][0].as_f64().unwrap() - 298.15).abs() < 1e-9);

        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/update-and-1-out"),
            json!({"input_pair": "PT_INPUTS", "value1": [101325.0], "value2": [298.15],
                   "output": "Dmolar"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert_eq!(b["out"].as_array().unwrap().len(), 1);
        assert!(b["out"][0].as_f64().unwrap() > 0.0);

        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/update-and-5-out"),
            json!({"input_pair": "PT_INPUTS", "value1": [101325.0], "value2": [298.15],
                   "outputs": ["T", "P", "Dmolar", "Hmolar", "Smolar"]}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        for i in 1..=5 {
            assert!(b[&format!("out{i}")].as_array().unwrap()[0]
                .as_f64()
                .unwrap()
                .is_finite());
        }

        // two-phase state
        assert_eq!(
            update(handle, "PQ_INPUTS", 101325.0, 0.5).await,
            StatusCode::OK
        );

        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/keyed-output/sat-state"),
            json!({"saturated_state": "liquid", "param": "Dmolar"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert!(b["value"].as_f64().unwrap() > 0.0);

        let (s, b) = req(
            Method::GET,
            &format!("/api/v1/abstract-state/{handle}/saturated-liquid-output/Smolar"),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert!(b["value"].as_f64().unwrap().is_finite());

        let (s, b) = req(
            Method::GET,
            &format!("/api/v1/abstract-state/{handle}/saturated-vapor-output/Dmolar"),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert!(b["value"].as_f64().unwrap() > 0.0);

        let (s, b) = req(
            Method::GET,
            &format!(
                "/api/v1/abstract-state/{handle}/mole-fractions/sat-state?saturated_state=liquid"
            ),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert_eq!(b["values"].as_array().unwrap().len(), 2);

        // Saturation derivatives need the bubble/dew line for mixtures.
        assert_eq!(
            update(handle, "PQ_INPUTS", 101325.0, 1.0).await,
            StatusCode::OK
        );
        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/first-saturation-deriv"),
            json!({"of": "P", "wrt": "T"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert!(b["value"].as_f64().unwrap().is_finite());

        // The *two-phase* derivative family is only implemented for pure
        // fluids in CoolProp (mixtures delegate to saturation derivatives
        // that reject the state); a clean 400 is the faithful behavior
        // here. The success paths are covered on a pure fluid below.
        for (path, body) in [
            (
                "first-two-phase-deriv",
                json!({"of": "Dmass", "wrt": "P", "constant": "Hmass"}),
            ),
            (
                "first-two-phase-deriv-splined",
                json!({"of": "Dmass", "wrt": "P", "constant": "Hmass", "x_end": 0.1}),
            ),
            (
                "second-two-phase-deriv",
                json!({"of1": "Dmass", "wrt1": "P", "constant1": "Hmass",
                    "wrt2": "P", "constant2": "Hmass"}),
            ),
        ] {
            let (s, b) = json_req(
                Method::POST,
                &format!("/api/v1/abstract-state/{handle}/{path}"),
                body,
            )
            .await;
            assert!(
                s == StatusCode::OK || s == StatusCode::BAD_REQUEST,
                "{path}: unexpected status {s}, body {b}",
            );
        }

        // binary interaction parameter
        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/binary-interaction"),
            json!({"i": 0, "j": 1, "parameter": "betaT", "value": 1.0}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");

        // phase imposition round trip
        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/specify-phase"),
            json!({"phase": "phase_liquid"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");

        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/unspecify-phase"),
            json!({}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");

        // fluid param string
        let (s, b) = req(
            Method::GET,
            &format!("/api/v1/abstract-state/{handle}/fluid-param-string?param=pure"),
        )
        .await;
        if s == StatusCode::OK {
            assert!(!b["value"].as_str().unwrap().is_empty());
        } else {
            assert_eq!(s, StatusCode::BAD_REQUEST, "body: {b}");
        }

        // phase envelope (mixture)
        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/phase-envelope/build"),
            json!({"level": "none"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");

        let (s, b) = req(
            Method::GET,
            &format!(
                "/api/v1/abstract-state/{handle}/phase-envelope?max_length=10000&max_components=5"
            ),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert!(b["length"].as_u64().unwrap() > 10, "envelope too small");
        assert_eq!(b["components"], 2);
        assert_eq!(
            b["t"].as_array().unwrap().len() as u64,
            b["length"].as_u64().unwrap()
        );

        // raw (unchecked) variant errors cleanly when the buffer is too small
        let (s, b) = req(
            Method::GET,
            &format!(
                "/api/v1/abstract-state/{handle}/phase-envelope/raw?max_length=2&max_components=5"
            ),
        )
        .await;
        assert_eq!(s, StatusCode::BAD_REQUEST, "body: {b}");

        let (s, b) = req(
            Method::GET,
            &format!("/api/v1/abstract-state/{handle}/phase-envelope/raw?max_length=5000&max_components=5"),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");

        // critical points
        let (s, b) = req(
            Method::GET,
            &format!("/api/v1/abstract-state/{handle}/all-critical-points?max_points=10"),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert!(!b["points"].as_array().unwrap().is_empty());
        assert!(b["points"][0]["t"].as_f64().unwrap() > 0.0);

        // free, then 404s
        let (s, _) = req(Method::DELETE, &format!("/api/v1/abstract-state/{handle}")).await;
        assert_eq!(s, StatusCode::OK);
        let (s, _) = req(
            Method::GET,
            &format!("/api/v1/abstract-state/{handle}/keyed-output/T"),
        )
        .await;
        assert_eq!(s, StatusCode::NOT_FOUND);
        let (s, _) = req(Method::DELETE, &format!("/api/v1/abstract-state/{handle}")).await;
        assert_eq!(s, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn pure_fluid_spinodal_and_derivs() {
        let (status, body) = create("HEOS", json!(["Water"])).await;
        assert_eq!(status, StatusCode::OK, "body: {body}");
        let handle = body["handle"].as_i64().unwrap();

        assert_eq!(
            update(handle, "PT_INPUTS", 101325.0, 400.0).await,
            StatusCode::OK
        );

        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/second-partial-deriv"),
            json!({"of1": "Dmolar", "wrt1": "T", "constant1": "P",
                   "wrt2": "T", "constant2": "P"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert!(b["value"].as_f64().unwrap().is_finite());

        // Two-phase derivative family (pure-fluid success paths).
        assert_eq!(
            update(handle, "PQ_INPUTS", 101325.0, 0.5).await,
            StatusCode::OK
        );
        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/first-two-phase-deriv"),
            json!({"of": "Dmass", "wrt": "P", "constant": "Hmass"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert!(b["value"].as_f64().unwrap().is_finite());

        // The splined variant requires 0 <= Q <= x_end.
        assert_eq!(
            update(handle, "PQ_INPUTS", 101325.0, 0.05).await,
            StatusCode::OK
        );
        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/first-two-phase-deriv-splined"),
            json!({"of": "Dmass", "wrt": "P", "constant": "Hmass", "x_end": 0.1}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert!(b["value"].as_f64().unwrap().is_finite());

        // The only implemented second two-phase derivative combination is
        // (Dmolar, Hmolar|P, P|Hmolar).
        assert_eq!(
            update(handle, "PQ_INPUTS", 101325.0, 0.5).await,
            StatusCode::OK
        );
        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/second-two-phase-deriv"),
            json!({"of1": "Dmolar", "wrt1": "Hmolar", "constant1": "P",
                   "wrt2": "P", "constant2": "Hmolar"}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert!(b["value"].as_f64().unwrap().is_finite());

        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/spinodal/build"),
            json!({}),
        )
        .await;
        if s == StatusCode::OK {
            let (s, b) = req(
                Method::GET,
                &format!("/api/v1/abstract-state/{handle}/spinodal?max_length=10000"),
            )
            .await;
            assert_eq!(s, StatusCode::OK, "body: {b}");
            assert!(!b["tau"].as_array().unwrap().is_empty());
        } else {
            // Spinodal construction is not supported for every fluid/state.
            assert_eq!(s, StatusCode::BAD_REQUEST, "body: {b}");
        }

        let (s, _) = req(Method::DELETE, &format!("/api/v1/abstract-state/{handle}")).await;
        assert_eq!(s, StatusCode::OK);
    }

    #[tokio::test]
    async fn cubic_backend_parameters() {
        let (status, body) = create("PR", json!(["Propane"])).await;
        assert_eq!(status, StatusCode::OK, "body: {body}");
        let handle = body["handle"].as_i64().unwrap();

        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/cubic-alpha-c"),
            json!({"i": 0, "parameter": "TWU", "c1": 1.0, "c2": 1.0, "c3": 1.0}),
        )
        .await;
        if s != StatusCode::OK {
            assert_eq!(s, StatusCode::BAD_REQUEST, "body: {b}");
        }

        let (s, b) = json_req(
            Method::POST,
            &format!("/api/v1/abstract-state/{handle}/fluid-parameter-double"),
            json!({"i": 0, "parameter": "c", "value": 0.0}),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");

        assert_eq!(
            update(handle, "PT_INPUTS", 101325.0, 298.15).await,
            StatusCode::OK
        );
        let (s, b) = req(
            Method::GET,
            &format!("/api/v1/abstract-state/{handle}/keyed-output/Dmolar"),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "body: {b}");
        assert!(b["value"].as_f64().unwrap() > 0.0);

        let (s, _) = req(Method::DELETE, &format!("/api/v1/abstract-state/{handle}")).await;
        assert_eq!(s, StatusCode::OK);
    }
}
