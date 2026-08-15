//! Humid air property endpoints (`HAPropsSI`, `HAProps`, `cair_sat`).

use axum::Json;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::dto::DoubleValue;
use crate::error::ApiResult;
use crate::safe;

#[derive(Debug, Deserialize, ToSchema)]
pub struct HaPropsRequest {
    /// Output, e.g. `"H"`, `"W"`, `"R"`, `"T"`, `"V"`, ...
    pub output: String,
    /// First input name, e.g. `"T"`.
    pub name1: String,
    /// First input value.
    pub prop1: f64,
    /// Second input name, e.g. `"P"`.
    pub name2: String,
    /// Second input value.
    pub prop2: f64,
    /// Third input name, e.g. `"R"` (relative humidity) or `"W"`.
    pub name3: String,
    /// Third input value.
    pub prop3: f64,
}

/// Humid air properties in SI units.
///
/// Maps to the C function `HAPropsSI`.
#[utoipa::path(
    post,
    path = "/api/v1/ha/props-si",
    request_body = HaPropsRequest,
    responses(
        (status = 200, description = "Property value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn ha_props_si(Json(req): Json<HaPropsRequest>) -> ApiResult<Json<DoubleValue>> {
    let v = safe::haprops_si(
        &req.output,
        &req.name1,
        req.prop1,
        &req.name2,
        req.prop2,
        &req.name3,
        req.prop3,
    )?;
    Ok(Json(DoubleValue { value: v }))
}

/// Deprecated humid air properties call (non-SI units).
///
/// Maps to the C function `HAProps`.
#[utoipa::path(
    post,
    path = "/api/v1/ha/props",
    request_body = HaPropsRequest,
    responses(
        (status = 200, description = "Property value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn ha_props(Json(req): Json<HaPropsRequest>) -> ApiResult<Json<DoubleValue>> {
    let v = safe::haprops_legacy(
        &req.output,
        &req.name1,
        req.prop1,
        &req.name2,
        req.prop2,
        &req.name3,
        req.prop3,
    )?;
    Ok(Json(DoubleValue { value: v }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CairSatRequest {
    /// Temperature in K (valid range 250–300 K, no bound checking).
    pub t: f64,
}

/// Humid air saturation specific heat at 1 atmosphere.
///
/// Maps to the C function `cair_sat`.
#[utoipa::path(
    post,
    path = "/api/v1/ha/cair-sat",
    request_body = CairSatRequest,
    responses(
        (status = 200, description = "Specific heat", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn cair_sat(Json(req): Json<CairSatRequest>) -> ApiResult<Json<DoubleValue>> {
    let v = safe::cair_sat(req.t)?;
    Ok(Json(DoubleValue { value: v }))
}
