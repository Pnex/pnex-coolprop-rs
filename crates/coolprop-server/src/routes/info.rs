//! Parameter/fluid information and miscellaneous utility endpoints.

use axum::extract::{Path, Query};
use axum::Json;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::dto::{DoubleValue, FlagValue, IndexValue, LengthValue, StringValue};
use crate::error::ApiResult;
use crate::safe;

#[derive(Debug, Deserialize, IntoParams)]
pub struct NamedParam {
    /// Parameter name, e.g. `"T"`, `"Dmolar"`, `"PT_INPUTS"`, ...
    pub name: String,
}

/// Query a global parameter string: `version`, `gitrevision`, `fluids_list`,
/// `errstring`, `warnstring`, `FluidsList`, ...
///
/// Maps to the C function `get_global_param_string`.
#[utoipa::path(
    get,
    path = "/api/v1/params/global/{param}",
    params(("param" = String, Path, description = "Global parameter name, e.g. `version`")),
    responses(
        (status = 200, description = "Parameter value", body = StringValue),
        (status = 400, description = "Unknown parameter", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn get_global_param_string(Path(param): Path<String>) -> ApiResult<Json<StringValue>> {
    Ok(Json(StringValue {
        value: safe::get_global_param_string(&param)?,
    }))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct InfoKind {
    /// Kind of information: `"long"` (description, default), `"short"`,
    /// `"units"`, or `"IO"`.
    #[serde(default)]
    pub info: Option<String>,
}

/// Long description, units, or IO role of a CoolProp parameter, e.g. for
/// `"T"` + `long` → `"Temperature [K]"`.
///
/// Maps to the C function `get_parameter_information_string`.
#[utoipa::path(
    get,
    path = "/api/v1/params/information/{param}",
    params(
        ("param" = String, Path, description = "Parameter name, e.g. `Dmolar`"),
        InfoKind,
    ),
    responses(
        (status = 200, description = "Human-readable parameter information", body = StringValue),
        (status = 400, description = "Unknown parameter", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn get_parameter_information_string(
    Path(param): Path<String>,
    Query(kind): Query<InfoKind>,
) -> ApiResult<Json<StringValue>> {
    let info = kind.info.unwrap_or_else(|| "long".to_string());
    Ok(Json(StringValue {
        value: safe::get_parameter_information_string(&param, &info)?,
    }))
}

/// Fluid metadata as a string: `"aliases"`, `"CAS"`, `"HASH"`, `"BibTeX"` ...
///
/// Maps to the C function `get_fluid_param_string`.
#[utoipa::path(
    get,
    path = "/api/v1/fluids/{fluid}/param/{param}",
    params(
        ("fluid" = String, Path, description = "Fluid name, e.g. `Water`"),
        ("param" = String, Path, description = "Fluid parameter, e.g. `aliases`"),
    ),
    responses(
        (status = 200, description = "Fluid parameter value", body = StringValue),
        (status = 400, description = "Unknown fluid/parameter", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn get_fluid_param_string(
    Path((fluid, param)): Path<(String, String)>,
) -> ApiResult<Json<StringValue>> {
    Ok(Json(StringValue {
        value: safe::get_fluid_param_string(&fluid, &param)?,
    }))
}

/// Length of the string that [`get_fluid_param_string`] returns.
///
/// Maps to the C function `get_fluid_param_string_len`.
#[utoipa::path(
    get,
    path = "/api/v1/fluids/{fluid}/param/{param}/length",
    params(
        ("fluid" = String, Path, description = "Fluid name"),
        ("param" = String, Path, description = "Fluid parameter"),
    ),
    responses(
        (status = 200, description = "String length", body = LengthValue),
        (status = 400, description = "Unknown fluid/parameter", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn get_fluid_param_string_len(
    Path((fluid, param)): Path<(String, String)>,
) -> ApiResult<Json<LengthValue>> {
    Ok(Json(LengthValue {
        length: safe::get_fluid_param_string_len(&fluid, &param)?,
    }))
}

/// Integer index for a CoolProp parameter name (for keyed outputs).
///
/// Maps to the C function `get_param_index`.
#[utoipa::path(
    get,
    path = "/api/v1/params/index",
    params(NamedParam),
    responses(
        (status = 200, description = "Parameter index", body = IndexValue),
        (status = 400, description = "Unknown parameter", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn get_param_index(Query(q): Query<NamedParam>) -> ApiResult<Json<IndexValue>> {
    Ok(Json(IndexValue {
        index: safe::get_param_index(&q.name)?,
    }))
}

/// Integer index for an AbstractState input pair name.
///
/// Maps to the C function `get_input_pair_index`.
#[utoipa::path(
    get,
    path = "/api/v1/input-pairs/index",
    params(NamedParam),
    responses(
        (status = 200, description = "Input pair index", body = IndexValue),
        (status = 400, description = "Unknown input pair", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn get_input_pair_index(Query(q): Query<NamedParam>) -> ApiResult<Json<IndexValue>> {
    Ok(Json(IndexValue {
        index: safe::get_input_pair_index(&q.name)?,
    }))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct FluidName {
    /// Fluid string to validate, e.g. `"HEOS::Water[0.5]&Ethane[0.5]"`.
    pub name: String,
}

/// Check whether a fluid string is valid.
///
/// Maps to the C function `C_is_valid_fluid_string`.
#[utoipa::path(
    get,
    path = "/api/v1/fluids/is-valid",
    params(FluidName),
    responses(
        (status = 200, description = "Validity flag", body = FlagValue),
    )
)]
pub async fn is_valid_fluid_string(Query(q): Query<FluidName>) -> ApiResult<Json<FlagValue>> {
    Ok(Json(FlagValue {
        value: safe::is_valid_fluid_string(&q.name),
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExtractBackendRequest {
    /// Fluid string, e.g. `"REFPROP::Water[0.5]&Ethane[0.5]"`.
    pub fluid_string: String,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct BackendFluid {
    /// Backend part (`"HEOS"`, `"REFPROP"`, ...; empty when not prefixed).
    pub backend: String,
    /// Fluid part, e.g. `"Water[0.5]&Ethane[0.5]"`.
    pub fluid: String,
}

/// Split a fluid string into its backend and fluid parts.
///
/// Maps to the C function `C_extract_backend`.
#[utoipa::path(
    post,
    path = "/api/v1/fluids/extract-backend",
    request_body = ExtractBackendRequest,
    responses(
        (status = 200, description = "Backend and fluid parts", body = BackendFluid),
        (status = 400, description = "Extraction failed", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn extract_backend(
    Json(req): Json<ExtractBackendRequest>,
) -> ApiResult<Json<BackendFluid>> {
    let (backend, fluid) = safe::extract_backend(&req.fluid_string)?;
    Ok(Json(BackendFluid { backend, fluid }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddFluidsRequest {
    /// Backend to add the fluids to, e.g. `"HEOS"`, `"SRK"`, `"PR"`.
    pub backend: String,
    /// JSON-formatted fluid definition string.
    pub fluid_string: String,
}

/// Add fluids defined by a JSON string to a backend's fluid library.
///
/// Maps to the C function `add_fluids_as_JSON`.
#[utoipa::path(
    post,
    path = "/api/v1/fluids/add-json",
    request_body = AddFluidsRequest,
    responses(
        (status = 200, description = "Fluids added", body = crate::dto::Ack),
        (status = 400, description = "CoolProp rejected the fluid definitions", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn add_fluids_as_json(
    Json(req): Json<AddFluidsRequest>,
) -> ApiResult<Json<crate::dto::Ack>> {
    safe::add_fluids_as_json(&req.backend, &req.fluid_string)?;
    Ok(Json(crate::dto::Ack { success: true }))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct Temperature {
    /// Temperature.
    pub t: f64,
}

/// Convert degrees Fahrenheit to Kelvin.
///
/// Maps to the C function `F2K`.
#[utoipa::path(
    get,
    path = "/api/v1/misc/f2k",
    params(Temperature),
    responses((status = 200, description = "Temperature in K", body = DoubleValue))
)]
pub async fn f2k(Query(q): Query<Temperature>) -> Json<DoubleValue> {
    Json(DoubleValue {
        value: safe::f2k(q.t),
    })
}

/// Convert Kelvin to degrees Fahrenheit.
///
/// Maps to the C function `K2F`.
#[utoipa::path(
    get,
    path = "/api/v1/misc/k2f",
    params(Temperature),
    responses((status = 200, description = "Temperature in °F", body = DoubleValue))
)]
pub async fn k2f(Query(q): Query<Temperature>) -> Json<DoubleValue> {
    Json(DoubleValue {
        value: safe::k2f(q.t),
    })
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct SaturationAncillaryParams {
    /// Fluid name (Helmholtz-EOS backend only).
    pub fluid: String,
    /// Desired output of the ancillary, e.g. `"P"`.
    pub output: String,
    /// Quality: 0 (saturated liquid) or 1 (saturated vapor).
    pub q: i32,
    /// Ancillary input variable, e.g. `"T"`.
    pub input: String,
    /// Input value.
    pub value: f64,
}

/// Evaluate a saturation ancillary curve (fast, but less accurate than the
/// full EOS).
///
/// Maps to the C function `saturation_ancillary`.
#[utoipa::path(
    get,
    path = "/api/v1/misc/saturation-ancillary",
    params(SaturationAncillaryParams),
    responses(
        (status = 200, description = "Ancillary value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn saturation_ancillary(
    Query(q): Query<SaturationAncillaryParams>,
) -> ApiResult<Json<DoubleValue>> {
    let v = safe::saturation_ancillary(&q.fluid, &q.output, q.q, &q.input, q.value)?;
    Ok(Json(DoubleValue { value: v }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DebugLevel {
    /// Verbosity level for CoolProp debugging output (0–10).
    pub level: i32,
}

/// Current CoolProp debug level.
///
/// Maps to the C function `get_debug_level`.
#[utoipa::path(
    get,
    path = "/api/v1/misc/debug-level",
    responses((status = 200, description = "Debug level", body = crate::dto::IndexValue))
)]
pub async fn get_debug_level() -> Json<crate::dto::IndexValue> {
    Json(crate::dto::IndexValue {
        index: safe::get_debug_level() as i64,
    })
}
