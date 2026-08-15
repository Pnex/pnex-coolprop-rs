//! Configuration and administration endpoints.
//!
//! Note: these mutate **process-global** CoolProp state and affect every
//! subsequent call on this server.

use axum::Json;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::dto::Ack;
use crate::error::ApiResult;
use crate::safe;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetConfigStringRequest {
    /// Configuration key, e.g. `"VTPR_BACKEND"`.
    pub key: String,
    pub val: String,
}

/// Set a string configuration value (process-global).
///
/// Maps to the C function `set_config_string`.
#[utoipa::path(
    put,
    path = "/api/v1/config/string",
    request_body = SetConfigStringRequest,
    responses(
        (status = 200, description = "Configuration applied", body = Ack),
        (status = 400, description = "Unknown key", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn set_config_string(Json(req): Json<SetConfigStringRequest>) -> ApiResult<Json<Ack>> {
    safe::set_config_string(&req.key, &req.val)?;
    Ok(Json(Ack { success: true }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetConfigDoubleRequest {
    /// Configuration key, e.g. `"RHOL_TO_RHOMASS_MAX_RATIO"`.
    pub key: String,
    pub val: f64,
}

/// Set a numeric configuration value (process-global).
///
/// Maps to the C function `set_config_double`.
#[utoipa::path(
    put,
    path = "/api/v1/config/double",
    request_body = SetConfigDoubleRequest,
    responses(
        (status = 200, description = "Configuration applied", body = Ack),
        (status = 400, description = "Unknown key", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn set_config_double(Json(req): Json<SetConfigDoubleRequest>) -> ApiResult<Json<Ack>> {
    safe::set_config_double(&req.key, req.val)?;
    Ok(Json(Ack { success: true }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetConfigBoolRequest {
    /// Configuration key, e.g. `"SUPERANCILLARY"`.
    pub key: String,
    pub val: bool,
}

/// Set a boolean configuration value (process-global).
///
/// Maps to the C function `set_config_bool`.
#[utoipa::path(
    put,
    path = "/api/v1/config/bool",
    request_body = SetConfigBoolRequest,
    responses(
        (status = 200, description = "Configuration applied", body = Ack),
        (status = 400, description = "Unknown key", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn set_config_bool(Json(req): Json<SetConfigBoolRequest>) -> ApiResult<Json<Ack>> {
    safe::set_config_bool(&req.key, req.val)?;
    Ok(Json(Ack { success: true }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetDepartureFunctionsRequest {
    /// Departure functions as a JSON string, or the contents of a REFPROP
    /// HMX.BNC file.
    pub string_data: String,
}

/// Set the departure functions in the departure-function library
/// (process-global).
///
/// Maps to the C function `set_departure_functions`.
#[utoipa::path(
    post,
    path = "/api/v1/config/departure-functions",
    request_body = SetDepartureFunctionsRequest,
    responses(
        (status = 200, description = "Departure functions set", body = Ack),
        (status = 400, description = "CoolProp rejected the data", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn set_departure_functions(
    Json(req): Json<SetDepartureFunctionsRequest>,
) -> ApiResult<Json<Ack>> {
    safe::set_departure_functions(&req.string_data)?;
    Ok(Json(Ack { success: true }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetReferenceStateSRequest {
    /// Fluid name.
    pub fluid: String,
    /// Named reference state: `"IIR"`, `"ASHRAE"`, `"NBP"`, `"DEF"`, `"RESET"`.
    pub reference_state: String,
}

/// Set the reference state of a fluid by name (process-global).
///
/// Maps to the C function `set_reference_stateS`.
#[utoipa::path(
    post,
    path = "/api/v1/config/reference-state/S",
    request_body = SetReferenceStateSRequest,
    responses(
        (status = 200, description = "Reference state applied", body = Ack),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn set_reference_state_s(
    Json(req): Json<SetReferenceStateSRequest>,
) -> ApiResult<Json<Ack>> {
    safe::set_reference_state_s(&req.fluid, &req.reference_state)?;
    Ok(Json(Ack { success: true }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetReferenceStateDRequest {
    /// Fluid name.
    pub fluid: String,
    /// Temperature [K].
    pub t: f64,
    /// Molar density [mol/m³].
    pub rhomolar: f64,
    /// Molar enthalpy at the new reference state [J/mol].
    pub hmolar0: f64,
    /// Molar entropy at the new reference state [J/mol/K].
    pub smolar0: f64,
}

/// Set the reference state of a fluid to a specified state (process-global).
///
/// Maps to the C function `set_reference_stateD`.
#[utoipa::path(
    post,
    path = "/api/v1/config/reference-state/D",
    request_body = SetReferenceStateDRequest,
    responses(
        (status = 200, description = "Reference state applied", body = Ack),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn set_reference_state_d(
    Json(req): Json<SetReferenceStateDRequest>,
) -> ApiResult<Json<Ack>> {
    safe::set_reference_state_d(&req.fluid, req.t, req.rhomolar, req.hmolar0, req.smolar0)?;
    Ok(Json(Ack { success: true }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DebugLevelRequest {
    /// Verbosity level for CoolProp debugging output (0–10).
    pub level: i32,
}

/// Set the CoolProp debug level (process-global).
///
/// Maps to the C function `set_debug_level`.
#[utoipa::path(
    put,
    path = "/api/v1/misc/debug-level",
    request_body = DebugLevelRequest,
    responses((status = 200, description = "Debug level set", body = Ack))
)]
pub async fn set_debug_level(Json(req): Json<DebugLevelRequest>) -> Json<Ack> {
    safe::set_debug_level(req.level);
    Json(Ack { success: true })
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RedirectStdoutRequest {
    /// File path to append stdout to (process-global). Use `/dev/stdout` on
    /// Linux to restore the original stream.
    pub file: String,
}

/// Redirect CoolProp console output to a file (process-global).
///
/// Maps to the C function `redirect_stdout`.
#[utoipa::path(
    post,
    path = "/api/v1/admin/redirect-stdout",
    request_body = RedirectStdoutRequest,
    responses(
        (status = 200, description = "stdout redirected", body = Ack),
        (status = 400, description = "Redirection failed", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn redirect_stdout(Json(req): Json<RedirectStdoutRequest>) -> ApiResult<Json<Ack>> {
    safe::redirect_stdout(&req.file)?;
    Ok(Json(Ack { success: true }))
}
