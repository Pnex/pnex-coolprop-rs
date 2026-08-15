//! FORTRAN 77 style wrapper endpoints (`propssi_`, `hapropssi_`, `haprops_`).
//!
//! These mirror the SI-unit endpoints exactly — they exist so the server
//! covers every symbol exported by `CoolPropLib.h`.

use axum::Json;

use crate::dto::DoubleValue;
use crate::error::ApiResult;
use crate::routes::humid_air::HaPropsRequest;
use crate::routes::props::PropsSiRequest;
use crate::safe;

/// FORTRAN 77 style wrapper of `PropsSI`.
///
/// Maps to the C function `propssi_`.
#[utoipa::path(
    post,
    path = "/api/v1/fortran/propssi",
    request_body = PropsSiRequest,
    responses(
        (status = 200, description = "Property value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn fortran_propssi(Json(req): Json<PropsSiRequest>) -> ApiResult<Json<DoubleValue>> {
    let v = safe::propssi_fortran(
        &req.output,
        &req.name1,
        req.prop1,
        &req.name2,
        req.prop2,
        &req.fluid,
    )?;
    Ok(Json(DoubleValue { value: v }))
}

/// FORTRAN 77 style wrapper of `HAPropsSI`.
///
/// Maps to the C function `hapropssi_`.
#[utoipa::path(
    post,
    path = "/api/v1/fortran/hapropssi",
    request_body = HaPropsRequest,
    responses(
        (status = 200, description = "Property value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn fortran_hapropssi(Json(req): Json<HaPropsRequest>) -> ApiResult<Json<DoubleValue>> {
    let v = safe::hapropssi_fortran(
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

/// FORTRAN 77 style wrapper of the deprecated `HAProps`.
///
/// Maps to the C function `haprops_`.
#[utoipa::path(
    post,
    path = "/api/v1/fortran/haprops",
    request_body = HaPropsRequest,
    responses(
        (status = 200, description = "Property value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn fortran_haprops(Json(req): Json<HaPropsRequest>) -> ApiResult<Json<DoubleValue>> {
    let v = safe::haprops_fortran(
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
