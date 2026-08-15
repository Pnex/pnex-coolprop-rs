//! High-level property endpoints (`PropsSI`, `Props1SI`, `PhaseSI`, the
//! `*multi` variants, and the deprecated KSI-unit `Props` family).

use axum::Json;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::dto::{ArrayValue, DoubleValue, MatrixValue, StringValue};
use crate::error::{ApiError, ApiResult};
use crate::safe;

#[derive(Debug, Deserialize, ToSchema)]
pub struct PropsSiRequest {
    /// Output parameter name, e.g. `"Dmolar"`, `"Hmolar"`, `"T"`, ...
    pub output: String,
    /// Name of the first input, e.g. `"T"`.
    pub name1: String,
    /// Value of the first input.
    pub prop1: f64,
    /// Name of the second input, e.g. `"P"`.
    pub name2: String,
    /// Value of the second input.
    pub prop2: f64,
    /// Fluid string, e.g. `"Propane[0.5]&Ethane[0.5]"`, `"HEOS::Water"`,
    /// `"REFPROP::R134a"` (backend prefix optional, default `HEOS`).
    pub fluid: String,
}

/// Compute a thermophysical property (SI units) at a single state point.
///
/// Maps to the C function `PropsSI`.
#[utoipa::path(
    post,
    path = "/api/v1/props/si",
    request_body = PropsSiRequest,
    responses(
        (status = 200, description = "Property value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn props_si(Json(req): Json<PropsSiRequest>) -> ApiResult<Json<DoubleValue>> {
    let v = safe::props_si(
        &req.output,
        &req.name1,
        req.prop1,
        &req.name2,
        req.prop2,
        &req.fluid,
    )?;
    Ok(Json(DoubleValue { value: v }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct Props1SiRequest {
    /// Fluid name, e.g. `"Water"` or `"HEOS::R134a"`.
    pub fluid: String,
    /// Output that requires no state inputs, e.g. `"Tcrit"`, `"Pcrit"`,
    /// `"molar_mass"`, `"aliases"`, ...
    pub output: String,
}

/// Compute a parameter that needs no state inputs (critical point data,
/// molar mass, ...) for a fluid.
///
/// Maps to the C function `Props1SI`.
#[utoipa::path(
    post,
    path = "/api/v1/props/1si",
    request_body = Props1SiRequest,
    responses(
        (status = 200, description = "Property value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn props_1si(Json(req): Json<Props1SiRequest>) -> ApiResult<Json<DoubleValue>> {
    let v = safe::props_1si(&req.fluid, &req.output)?;
    Ok(Json(DoubleValue { value: v }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PropsSiMultiRequest {
    /// Output parameter names; one column of the result per output.
    pub outputs: Vec<String>,
    /// Name of the first input variable.
    pub name1: String,
    /// Values of the first input variable.
    pub prop1: Vec<f64>,
    /// Name of the second input variable.
    pub name2: String,
    /// Values of the second input variable (same length as `prop1`).
    pub prop2: Vec<f64>,
    /// Backend (`"HEOS"`, `"REFPROP"`, ...). Defaults to `"HEOS"`.
    #[serde(default)]
    pub backend: Option<String>,
    /// Fluid names (one per component).
    pub fluids: Vec<String>,
    /// Molar fractions, one per fluid. Defaults to an equal split.
    #[serde(default)]
    pub fractions: Vec<f64>,
}

/// Vectorized `PropsSI`: compute one or more outputs over arrays of input
/// values. Returns a matrix with one row per input point and one column per
/// requested output.
///
/// Maps to the C function `PropsSImulti`.
#[utoipa::path(
    post,
    path = "/api/v1/props/si-multi",
    request_body = PropsSiMultiRequest,
    responses(
        (status = 200, description = "Result matrix (rows = input points, columns = outputs)", body = MatrixValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn props_si_multi(Json(req): Json<PropsSiMultiRequest>) -> ApiResult<Json<MatrixValue>> {
    if req.prop1.len() != req.prop2.len() {
        return Err(ApiError::BadInput(format!(
            "prop1 has {} values but prop2 has {}",
            req.prop1.len(),
            req.prop2.len()
        )));
    }
    if req.prop1.is_empty() {
        return Err(ApiError::BadInput("prop1/prop2 must not be empty".into()));
    }
    let fractions = normalize_fractions(&req.fluids, &req.fractions)?;
    let backend = req.backend.clone().unwrap_or_default();
    let rows = safe::props_si_multi(
        &req.outputs,
        &req.name1,
        &req.prop1,
        &req.name2,
        &req.prop2,
        &backend,
        &req.fluids,
        &fractions,
    )?;
    let shape = vec![rows.len(), rows.first().map(|r| r.len()).unwrap_or(0)];
    Ok(Json(MatrixValue {
        shape,
        values: rows,
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct Props1SiMultiRequest {
    /// Output parameter names.
    pub outputs: Vec<String>,
    /// Backend (`"HEOS"`, `"REFPROP"`, ...). Defaults to `"HEOS"`.
    #[serde(default)]
    pub backend: Option<String>,
    /// Fluid names (one per component).
    pub fluids: Vec<String>,
    /// Molar fractions, one per fluid. Defaults to an equal split.
    #[serde(default)]
    pub fractions: Vec<f64>,
}

/// Vectorized `Props1SI` over several fluids. Note that — exactly like the
/// underlying C function — only the first result row is exposed.
///
/// Maps to the C function `Props1SImulti`.
#[utoipa::path(
    post,
    path = "/api/v1/props/1si-multi",
    request_body = Props1SiMultiRequest,
    responses(
        (status = 200, description = "Result values", body = ArrayValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn props_1si_multi(Json(req): Json<Props1SiMultiRequest>) -> ApiResult<Json<ArrayValue>> {
    let fractions = normalize_fractions(&req.fluids, &req.fractions)?;
    let backend = req.backend.clone().unwrap_or_default();
    let values = safe::props_1_si_multi(&req.outputs, &backend, &req.fluids, &fractions)?;
    Ok(Json(ArrayValue { values }))
}

/// Phase at a given state point, e.g. `"liquid"`, `"gas"`, `"twophase"`,
/// `"supercritical"`, ...
///
/// Maps to the C function `PhaseSI`.
#[utoipa::path(
    post,
    path = "/api/v1/props/phase",
    request_body = PropsSiRequest,
    responses(
        (status = 200, description = "Phase name", body = StringValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn phase_si(Json(req): Json<PropsSiRequest>) -> ApiResult<Json<StringValue>> {
    let phase = safe::phase_si(&req.name1, req.prop1, &req.name2, req.prop2, &req.fluid)?;
    Ok(Json(StringValue { value: phase }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PropsLegacyRequest {
    pub output: String,
    /// Single-character input name (e.g. `"T"`) — the legacy API only
    /// accepts one character here.
    pub name1: String,
    pub prop1: f64,
    pub name2: String,
    pub prop2: f64,
    /// Fluid / reference string.
    #[serde(rename = "ref")]
    pub reference: String,
}

/// Deprecated legacy property call, KSI unit system, char input names.
///
/// Maps to the C function `Props`.
#[utoipa::path(
    post,
    path = "/api/v1/props/legacy",
    request_body = PropsLegacyRequest,
    responses(
        (status = 200, description = "Property value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn props_legacy(Json(req): Json<PropsLegacyRequest>) -> ApiResult<Json<DoubleValue>> {
    let n1 = single_char(&req.name1, "name1")?;
    let n2 = single_char(&req.name2, "name2")?;
    let v = safe::props_legacy(&req.output, n1, req.prop1, n2, req.prop2, &req.reference)?;
    Ok(Json(DoubleValue { value: v }))
}

/// Deprecated legacy property call with string input names (KSI units).
///
/// Maps to the C function `PropsS`.
#[utoipa::path(
    post,
    path = "/api/v1/props/legacy-s",
    request_body = PropsLegacyRequest,
    responses(
        (status = 200, description = "Property value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn props_legacy_s(Json(req): Json<PropsLegacyRequest>) -> ApiResult<Json<DoubleValue>> {
    let v = safe::props_s(
        &req.output,
        &req.name1,
        req.prop1,
        &req.name2,
        req.prop2,
        &req.reference,
    )?;
    Ok(Json(DoubleValue { value: v }))
}

/// Deprecated legacy `Props1` (KSI units): single-state output for a fluid.
///
/// Maps to the C function `Props1`.
#[utoipa::path(
    post,
    path = "/api/v1/props/legacy/1",
    request_body = Props1SiRequest,
    responses(
        (status = 200, description = "Property value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn props_legacy_1(Json(req): Json<Props1SiRequest>) -> ApiResult<Json<DoubleValue>> {
    let v = safe::props1_legacy(&req.fluid, &req.output)?;
    Ok(Json(DoubleValue { value: v }))
}

fn single_char(s: &str, field: &str) -> ApiResult<char> {
    let mut chars = s.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Ok(c),
        _ => Err(ApiError::BadInput(format!(
            "{field} must be exactly one character, got {s:?}"
        ))),
    }
}

/// Default empty fractions to an equal split; otherwise require one per fluid.
pub(crate) fn normalize_fractions(fluids: &[String], fractions: &[f64]) -> ApiResult<Vec<f64>> {
    if fractions.is_empty() {
        if fluids.is_empty() {
            return Err(ApiError::BadInput("fluids must not be empty".into()));
        }
        let n = fluids.len() as f64;
        return Ok(vec![1.0 / n; fluids.len()]);
    }
    if fractions.len() != fluids.len() {
        return Err(ApiError::BadInput(format!(
            "got {} fractions for {} fluids",
            fractions.len(),
            fluids.len()
        )));
    }
    Ok(fractions.to_vec())
}
