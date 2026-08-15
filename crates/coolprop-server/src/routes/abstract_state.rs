//! Stateful low-level AbstractState endpoints.
//!
//! An AbstractState is created with `POST /api/v1/abstract-state`, which
//! returns an integer handle; every other endpoint in this module addresses
//! the state through that handle. The handle is released with `DELETE`.
//!
//! Handles are kept in server memory (lost on restart).

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::dto::{Ack, ArrayValue, DoubleValue, InputPair, Param, StringValue};
use crate::error::{ApiError, ApiResult};
use crate::safe;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRequest {
    /// Backend: `"HEOS"`, `"REFPROP"`, `"INCOMP"`, `"SRK"`, `"PR"` ...
    pub backend: String,
    /// Fluids of the mixture, one entry per component.
    pub fluids: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HandleResponse {
    /// Handle for all subsequent calls.
    pub handle: i64,
}

/// Create an AbstractState and return its handle.
///
/// Maps to the C function `AbstractState_factory`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state",
    request_body = CreateRequest,
    responses(
        (status = 200, description = "Handle created", body = HandleResponse),
        (status = 400, description = "CoolProp rejected backend/fluids", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn create(
    State(app): State<AppState>,
    Json(req): Json<CreateRequest>,
) -> ApiResult<Json<HandleResponse>> {
    if req.fluids.is_empty() {
        return Err(ApiError::BadInput("fluids must not be empty".into()));
    }
    let handle = safe::abstract_state_factory(&req.backend, &req.fluids.join("&"))?;
    app.insert_handle(handle);
    Ok(Json(HandleResponse { handle }))
}

/// Release an AbstractState. Subsequent calls with this handle return 404.
///
/// Maps to the C function `AbstractState_free`.
#[utoipa::path(
    delete,
    path = "/api/v1/abstract-state/{handle}",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    responses(
        (status = 200, description = "State released", body = Ack),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn free(State(app): State<AppState>, Path(handle): Path<i64>) -> ApiResult<Json<Ack>> {
    app.require_handle(handle)?;
    safe::abstract_state_free(handle)?;
    app.remove_handle(handle);
    Ok(Json(Ack { success: true }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetFractionsRequest {
    /// Fractions (molar by default; mass/volume depending on backend use).
    pub fractions: Vec<f64>,
}

/// Set the fractions of the mixture components.
///
/// Maps to the C function `AbstractState_set_fractions`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/fractions",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    request_body = SetFractionsRequest,
    responses(
        (status = 200, description = "Fractions set", body = Ack),
        (status = 400, description = "CoolProp rejected the fractions", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn set_fractions(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    Json(req): Json<SetFractionsRequest>,
) -> ApiResult<Json<Ack>> {
    app.require_handle(handle)?;
    safe::abstract_state_set_fractions(handle, &req.fractions)?;
    Ok(Json(Ack { success: true }))
}

/// Get the mole fractions of the mixture.
///
/// Maps to the C function `AbstractState_get_mole_fractions`.
#[utoipa::path(
    get,
    path = "/api/v1/abstract-state/{handle}/mole-fractions",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    responses(
        (status = 200, description = "Mole fractions", body = ArrayValue),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn get_mole_fractions(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
) -> ApiResult<Json<ArrayValue>> {
    app.require_handle(handle)?;
    Ok(Json(ArrayValue {
        values: safe::abstract_state_get_mole_fractions(handle)?,
    }))
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SatStateParams {
    /// `"liquid"` or `"gas"`.
    pub saturated_state: String,
}

/// Get the mole fractions in the saturated liquid or vapor phase; only valid
/// in the two-phase region.
///
/// Maps to the C function `AbstractState_get_mole_fractions_satState`.
#[utoipa::path(
    get,
    path = "/api/v1/abstract-state/{handle}/mole-fractions/sat-state",
    params(
        ("handle" = i64, Path, description = "AbstractState handle"),
        SatStateParams,
    ),
    responses(
        (status = 200, description = "Saturated-state mole fractions", body = ArrayValue),
        (status = 400, description = "State is not two-phase", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn get_mole_fractions_sat_state(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    axum::extract::Query(q): axum::extract::Query<SatStateParams>,
) -> ApiResult<Json<ArrayValue>> {
    app.require_handle(handle)?;
    Ok(Json(ArrayValue {
        values: safe::abstract_state_get_mole_fractions_sat_state(handle, &q.saturated_state)?,
    }))
}

/// Fugacity of the i-th mixture component.
///
/// Maps to the C function `AbstractState_get_fugacity`.
#[utoipa::path(
    get,
    path = "/api/v1/abstract-state/{handle}/fugacity/{i}",
    params(
        ("handle" = i64, Path, description = "AbstractState handle"),
        ("i" = i64, Path, description = "Component index"),
    ),
    responses(
        (status = 200, description = "Fugacity [Pa]", body = DoubleValue),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn get_fugacity(
    State(app): State<AppState>,
    Path((handle, i)): Path<(i64, i64)>,
) -> ApiResult<Json<DoubleValue>> {
    app.require_handle(handle)?;
    Ok(Json(DoubleValue {
        value: safe::abstract_state_get_fugacity(handle, i)?,
    }))
}

/// Fugacity coefficient of the i-th mixture component.
///
/// Maps to the C function `AbstractState_get_fugacity_coefficient`.
#[utoipa::path(
    get,
    path = "/api/v1/abstract-state/{handle}/fugacity-coefficient/{i}",
    params(
        ("handle" = i64, Path, description = "AbstractState handle"),
        ("i" = i64, Path, description = "Component index"),
    ),
    responses(
        (status = 200, description = "Fugacity coefficient [-]", body = DoubleValue),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn get_fugacity_coefficient(
    State(app): State<AppState>,
    Path((handle, i)): Path<(i64, i64)>,
) -> ApiResult<Json<DoubleValue>> {
    app.require_handle(handle)?;
    Ok(Json(DoubleValue {
        value: safe::abstract_state_get_fugacity_coefficient(handle, i)?,
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateRequest {
    /// Input pair (name or index), e.g. `"PT_INPUTS"`.
    pub input_pair: InputPair,
    /// First input value (e.g. pressure [Pa] for `PT_INPUTS`).
    pub value1: f64,
    /// Second input value (e.g. temperature [K] for `PT_INPUTS`).
    pub value2: f64,
}

/// Update the state with a pair of inputs.
///
/// Maps to the C function `AbstractState_update`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/update",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    request_body = UpdateRequest,
    responses(
        (status = 200, description = "State updated", body = Ack),
        (status = 400, description = "CoolProp rejected the inputs", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn update(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    Json(req): Json<UpdateRequest>,
) -> ApiResult<Json<Ack>> {
    app.require_handle(handle)?;
    let pair = req.input_pair.resolve()?;
    safe::abstract_state_update(handle, pair, req.value1, req.value2)?;
    Ok(Json(Ack { success: true }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PhaseRequest {
    /// Phase to impose: `"liquid"`, `"gas"`, `"supercritical"`, ...
    pub phase: String,
}

/// Impose the phase for all further calculations.
///
/// Maps to the C function `AbstractState_specify_phase`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/specify-phase",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    request_body = PhaseRequest,
    responses(
        (status = 200, description = "Phase imposed", body = Ack),
        (status = 400, description = "CoolProp rejected the phase", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn specify_phase(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    Json(req): Json<PhaseRequest>,
) -> ApiResult<Json<Ack>> {
    app.require_handle(handle)?;
    safe::abstract_state_specify_phase(handle, &req.phase)?;
    Ok(Json(Ack { success: true }))
}

/// Remove an imposed phase.
///
/// Maps to the C function `AbstractState_unspecify_phase`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/unspecify-phase",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    responses(
        (status = 200, description = "Phase unspecified", body = Ack),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn unspecify_phase(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
) -> ApiResult<Json<Ack>> {
    app.require_handle(handle)?;
    safe::abstract_state_unspecify_phase(handle)?;
    Ok(Json(Ack { success: true }))
}

/// Get an output value by parameter name or index.
///
/// Maps to the C function `AbstractState_keyed_output`.
#[utoipa::path(
    get,
    path = "/api/v1/abstract-state/{handle}/keyed-output/{param}",
    params(
        ("handle" = i64, Path, description = "AbstractState handle"),
        ("param" = String, Path, description = "Parameter name (e.g. `Dmolar`) or numeric index"),
    ),
    responses(
        (status = 200, description = "Output value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the parameter", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn keyed_output(
    State(app): State<AppState>,
    Path((handle, param)): Path<(i64, Param)>,
) -> ApiResult<Json<DoubleValue>> {
    app.require_handle(handle)?;
    let p = param.resolve()?;
    Ok(Json(DoubleValue {
        value: safe::abstract_state_keyed_output(handle, p)?,
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FirstSaturationDerivRequest {
    /// Parameter being differentiated (name or index).
    #[serde(rename = "of")]
    pub of: Param,
    /// Parameter the derivative is taken with respect to.
    pub wrt: Param,
}

/// Saturation derivative `(∂Of/∂Wrt)|sat` — valid in the two-phase region.
///
/// Maps to the C function `AbstractState_first_saturation_deriv`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/first-saturation-deriv",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    request_body = FirstSaturationDerivRequest,
    responses(
        (status = 200, description = "Derivative value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn first_saturation_deriv(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    Json(req): Json<FirstSaturationDerivRequest>,
) -> ApiResult<Json<DoubleValue>> {
    app.require_handle(handle)?;
    let (of, wrt) = (req.of.resolve()?, req.wrt.resolve()?);
    Ok(Json(DoubleValue {
        value: safe::abstract_state_first_saturation_deriv(handle, of, wrt)?,
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FirstPartialDerivRequest {
    #[serde(rename = "of")]
    pub of: Param,
    pub wrt: Param,
    /// Parameter held constant.
    pub constant: Param,
}

/// First partial derivative `(∂Of/∂Wrt)|Constant` in homogeneous phases.
///
/// Maps to the C function `AbstractState_first_partial_deriv`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/first-partial-deriv",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    request_body = FirstPartialDerivRequest,
    responses(
        (status = 200, description = "Derivative value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn first_partial_deriv(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    Json(req): Json<FirstPartialDerivRequest>,
) -> ApiResult<Json<DoubleValue>> {
    app.require_handle(handle)?;
    let (of, wrt, k) = (
        req.of.resolve()?,
        req.wrt.resolve()?,
        req.constant.resolve()?,
    );
    Ok(Json(DoubleValue {
        value: safe::abstract_state_first_partial_deriv(handle, of, wrt, k)?,
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SecondPartialDerivRequest {
    #[serde(rename = "of1")]
    pub of1: Param,
    pub wrt1: Param,
    pub constant1: Param,
    pub wrt2: Param,
    pub constant2: Param,
}

/// Second partial derivative in homogeneous phases.
///
/// Maps to the C function `AbstractState_second_partial_deriv`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/second-partial-deriv",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    request_body = SecondPartialDerivRequest,
    responses(
        (status = 200, description = "Derivative value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn second_partial_deriv(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    Json(req): Json<SecondPartialDerivRequest>,
) -> ApiResult<Json<DoubleValue>> {
    app.require_handle(handle)?;
    let (of1, wrt1, c1, wrt2, c2) = (
        req.of1.resolve()?,
        req.wrt1.resolve()?,
        req.constant1.resolve()?,
        req.wrt2.resolve()?,
        req.constant2.resolve()?,
    );
    Ok(Json(DoubleValue {
        value: safe::abstract_state_second_partial_deriv(handle, of1, wrt1, c1, wrt2, c2)?,
    }))
}

/// Second derivative in the two-phase region.
///
/// Maps to the C function `AbstractState_second_two_phase_deriv`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/second-two-phase-deriv",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    request_body = SecondPartialDerivRequest,
    responses(
        (status = 200, description = "Derivative value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn second_two_phase_deriv(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    Json(req): Json<SecondPartialDerivRequest>,
) -> ApiResult<Json<DoubleValue>> {
    app.require_handle(handle)?;
    let (of1, wrt1, c1, wrt2, c2) = (
        req.of1.resolve()?,
        req.wrt1.resolve()?,
        req.constant1.resolve()?,
        req.wrt2.resolve()?,
        req.constant2.resolve()?,
    );
    Ok(Json(DoubleValue {
        value: safe::abstract_state_second_two_phase_deriv(handle, of1, wrt1, c1, wrt2, c2)?,
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FirstTwoPhaseDerivRequest {
    #[serde(rename = "of")]
    pub of: Param,
    pub wrt: Param,
    pub constant: Param,
    /// Spline end fraction (0..1, usually 0.1) — splined variant only.
    #[serde(default)]
    pub x_end: Option<f64>,
}

/// First derivative in the two-phase region.
///
/// Maps to the C function `AbstractState_first_two_phase_deriv`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/first-two-phase-deriv",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    request_body = FirstTwoPhaseDerivRequest,
    responses(
        (status = 200, description = "Derivative value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn first_two_phase_deriv(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    Json(req): Json<FirstTwoPhaseDerivRequest>,
) -> ApiResult<Json<DoubleValue>> {
    app.require_handle(handle)?;
    let (of, wrt, k) = (
        req.of.resolve()?,
        req.wrt.resolve()?,
        req.constant.resolve()?,
    );
    Ok(Json(DoubleValue {
        value: safe::abstract_state_first_two_phase_deriv(handle, of, wrt, k)?,
    }))
}

/// First two-phase derivative using the spline approach of Quoilin et al.
///
/// Maps to the C function `AbstractState_first_two_phase_deriv_splined`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/first-two-phase-deriv-splined",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    request_body = FirstTwoPhaseDerivRequest,
    responses(
        (status = 200, description = "Derivative value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn first_two_phase_deriv_splined(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    Json(req): Json<FirstTwoPhaseDerivRequest>,
) -> ApiResult<Json<DoubleValue>> {
    app.require_handle(handle)?;
    let x_end = req.x_end.unwrap_or(0.1);
    let (of, wrt, k) = (
        req.of.resolve()?,
        req.wrt.resolve()?,
        req.constant.resolve()?,
    );
    Ok(Json(DoubleValue {
        value: safe::abstract_state_first_two_phase_deriv_splined(handle, of, wrt, k, x_end)?,
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateAndOutRequest {
    pub input_pair: InputPair,
    pub value1: Vec<f64>,
    pub value2: Vec<f64>,
    /// Single output parameter (update-and-1-out variant).
    #[serde(default)]
    pub output: Option<Param>,
    /// Exactly five output parameters (update-and-5-out variant).
    #[serde(default)]
    pub outputs: Option<Vec<Param>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CommonOutResponse {
    pub t: Vec<f64>,
    pub p: Vec<f64>,
    pub rhomolar: Vec<f64>,
    pub hmolar: Vec<f64>,
    pub smolar: Vec<f64>,
}

/// Update the state over arrays of inputs and get the five common outputs
/// (T, p, rhomolar, hmolar, smolar).
///
/// Maps to the C function `AbstractState_update_and_common_out`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/update-and-common-out",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    request_body = UpdateAndOutRequest,
    responses(
        (status = 200, description = "Common outputs", body = CommonOutResponse),
        (status = 400, description = "CoolProp rejected the inputs", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn update_and_common_out(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    Json(req): Json<UpdateAndOutRequest>,
) -> ApiResult<Json<CommonOutResponse>> {
    app.require_handle(handle)?;
    let pair = req.input_pair.resolve()?;
    let out = safe::abstract_state_update_and_common_out(handle, pair, &req.value1, &req.value2)?;
    Ok(Json(CommonOutResponse {
        t: out.t,
        p: out.p,
        rhomolar: out.rhomolar,
        hmolar: out.hmolar,
        smolar: out.smolar,
    }))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Out1Response {
    pub out: Vec<f64>,
}

/// Update the state over arrays of inputs and get one keyed output.
///
/// Maps to the C function `AbstractState_update_and_1_out`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/update-and-1-out",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    request_body = UpdateAndOutRequest,
    responses(
        (status = 200, description = "Output array", body = Out1Response),
        (status = 400, description = "CoolProp rejected the inputs", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn update_and_1_out(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    Json(req): Json<UpdateAndOutRequest>,
) -> ApiResult<Json<Out1Response>> {
    app.require_handle(handle)?;
    let pair = req.input_pair.resolve()?;
    let output = req
        .output
        .ok_or_else(|| ApiError::BadInput("`output` is required for update-and-1-out".into()))?
        .resolve()?;
    let out =
        safe::abstract_state_update_and_1_out(handle, pair, &req.value1, &req.value2, output)?;
    Ok(Json(Out1Response { out }))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Out5Response {
    pub out1: Vec<f64>,
    pub out2: Vec<f64>,
    pub out3: Vec<f64>,
    pub out4: Vec<f64>,
    pub out5: Vec<f64>,
}

/// Update the state over arrays of inputs and get five keyed outputs.
///
/// Maps to the C function `AbstractState_update_and_5_out`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/update-and-5-out",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    request_body = UpdateAndOutRequest,
    responses(
        (status = 200, description = "Five output arrays", body = Out5Response),
        (status = 400, description = "CoolProp rejected the inputs", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn update_and_5_out(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    Json(req): Json<UpdateAndOutRequest>,
) -> ApiResult<Json<Out5Response>> {
    app.require_handle(handle)?;
    let pair = req.input_pair.resolve()?;
    let outputs = req
        .outputs
        .ok_or_else(|| ApiError::BadInput("`outputs` is required for update-and-5-out".into()))?;
    if outputs.len() != 5 {
        return Err(ApiError::BadInput(format!(
            "`outputs` must have exactly 5 entries, got {}",
            outputs.len()
        )));
    }
    let mut resolved = [0i64; 5];
    for (slot, p) in resolved.iter_mut().zip(&outputs) {
        *slot = p.resolve()?;
    }
    let outs =
        safe::abstract_state_update_and_5_out(handle, pair, &req.value1, &req.value2, resolved)?;
    let [o1, o2, o3, o4, o5] = outs;
    Ok(Json(Out5Response {
        out1: o1,
        out2: o2,
        out3: o3,
        out4: o4,
        out5: o5,
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BinaryInteractionRequest {
    /// Index of the first fluid of the binary pair.
    pub i: i64,
    /// Index of the second fluid of the binary pair.
    pub j: i64,
    /// Parameter name, e.g. `"betaT"`, `"gammaT"`.
    pub parameter: String,
    pub value: f64,
}

/// Set a binary interaction parameter of a mixture.
///
/// Maps to the C function `AbstractState_set_binary_interaction_double`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/binary-interaction",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    request_body = BinaryInteractionRequest,
    responses(
        (status = 200, description = "Parameter set", body = Ack),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn set_binary_interaction(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    Json(req): Json<BinaryInteractionRequest>,
) -> ApiResult<Json<Ack>> {
    app.require_handle(handle)?;
    safe::abstract_state_set_binary_interaction(handle, req.i, req.j, &req.parameter, req.value)?;
    Ok(Json(Ack { success: true }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CubicAlphaCRequest {
    /// Index of the fluid the parameters apply to.
    pub i: i64,
    /// Alpha function name, e.g. `"TWU"`.
    pub parameter: String,
    pub c1: f64,
    pub c2: f64,
    pub c3: f64,
}

/// Set cubic alpha function parameters (cubic EOS backends).
///
/// Maps to the C function `AbstractState_set_cubic_alpha_C`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/cubic-alpha-c",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    request_body = CubicAlphaCRequest,
    responses(
        (status = 200, description = "Parameters set", body = Ack),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn set_cubic_alpha_c(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    Json(req): Json<CubicAlphaCRequest>,
) -> ApiResult<Json<Ack>> {
    app.require_handle(handle)?;
    safe::abstract_state_set_cubic_alpha_c(handle, req.i, &req.parameter, req.c1, req.c2, req.c3)?;
    Ok(Json(Ack { success: true }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FluidParameterDoubleRequest {
    /// Index of the fluid the parameter applies to.
    pub i: i64,
    /// Parameter name, e.g. `"cm"` for volume translation.
    pub parameter: String,
    pub value: f64,
}

/// Set a numeric fluid parameter (e.g. volume translation in cubic backends).
///
/// Maps to the C function `AbstractState_set_fluid_parameter_double`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/fluid-parameter-double",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    request_body = FluidParameterDoubleRequest,
    responses(
        (status = 200, description = "Parameter set", body = Ack),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn set_fluid_parameter_double(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    Json(req): Json<FluidParameterDoubleRequest>,
) -> ApiResult<Json<Ack>> {
    app.require_handle(handle)?;
    safe::abstract_state_set_fluid_parameter_double(handle, req.i, &req.parameter, req.value)?;
    Ok(Json(Ack { success: true }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BuildPhaseEnvelopeRequest {
    /// Refinement level: `"none"` (recommended) or a higher level string.
    #[serde(default)]
    pub level: Option<String>,
}

/// Build the phase envelope for the current mixture composition.
///
/// Maps to the C function `AbstractState_build_phase_envelope`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/phase-envelope/build",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    request_body = BuildPhaseEnvelopeRequest,
    responses(
        (status = 200, description = "Envelope built", body = Ack),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn build_phase_envelope(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    Json(req): Json<BuildPhaseEnvelopeRequest>,
) -> ApiResult<Json<Ack>> {
    app.require_handle(handle)?;
    let level = req.level.unwrap_or_else(|| "none".to_string());
    safe::abstract_state_build_phase_envelope(handle, &level)?;
    Ok(Json(Ack { success: true }))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PhaseEnvelopeResponse {
    /// Number of envelope points.
    pub length: usize,
    /// Number of mixture components in `x`/`y`.
    pub components: usize,
    pub t: Vec<f64>,
    pub p: Vec<f64>,
    pub rhomolar_vap: Vec<f64>,
    pub rhomolar_liq: Vec<f64>,
    /// Liquid composition per point: `x[point][component]`.
    pub x: Vec<Vec<f64>>,
    /// Vapor composition per point: `y[point][component]`.
    pub y: Vec<Vec<f64>>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct PhaseEnvelopeParams {
    /// Buffer capacity for points (default 10000).
    #[serde(default)]
    pub max_length: Option<usize>,
    /// Buffer capacity for components (default 20).
    #[serde(default)]
    pub max_components: Option<usize>,
}

/// Get phase envelope data (checked-memory variant: reports actual sizes).
///
/// Maps to the C function `AbstractState_get_phase_envelope_data_checkedMemory`.
#[utoipa::path(
    get,
    path = "/api/v1/abstract-state/{handle}/phase-envelope",
    params(
        ("handle" = i64, Path, description = "AbstractState handle"),
        PhaseEnvelopeParams,
    ),
    responses(
        (status = 200, description = "Phase envelope data", body = PhaseEnvelopeResponse),
        (status = 400, description = "Buffer too small or envelope not built", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn get_phase_envelope(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    axum::extract::Query(q): axum::extract::Query<PhaseEnvelopeParams>,
) -> ApiResult<Json<PhaseEnvelopeResponse>> {
    app.require_handle(handle)?;
    let pe = safe::abstract_state_get_phase_envelope(
        handle,
        q.max_length.unwrap_or(10_000).clamp(1, 1_000_000),
        q.max_components.unwrap_or(20).clamp(1, 1_000),
    )?;
    Ok(Json(PhaseEnvelopeResponse {
        length: pe.actual_length,
        components: pe.actual_components,
        t: pe.t,
        p: pe.p,
        rhomolar_vap: pe.rhomolar_vap,
        rhomolar_liq: pe.rhomolar_liq,
        x: pe.x,
        y: pe.y,
    }))
}

/// Get phase envelope data — raw variant: returns exactly `length` points
/// with no report of the actual size (faithful to the C semantics).
///
/// Maps to the C function `AbstractState_get_phase_envelope_data`.
#[utoipa::path(
    get,
    path = "/api/v1/abstract-state/{handle}/phase-envelope/raw",
    params(
        ("handle" = i64, Path, description = "AbstractState handle"),
        PhaseEnvelopeParams,
    ),
    responses(
        (status = 200, description = "Phase envelope data (fixed length)", body = PhaseEnvelopeResponse),
        (status = 400, description = "Buffer too small or envelope not built", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn get_phase_envelope_raw(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    axum::extract::Query(q): axum::extract::Query<PhaseEnvelopeParams>,
) -> ApiResult<Json<PhaseEnvelopeResponse>> {
    app.require_handle(handle)?;
    let length = q.max_length.unwrap_or(1_000).clamp(1, 100_000);
    let components = q.max_components.unwrap_or(20).clamp(1, 1_000);
    let pe = safe::abstract_state_get_phase_envelope_raw(handle, length, components)?;
    Ok(Json(PhaseEnvelopeResponse {
        length: pe.actual_length,
        components: pe.actual_components,
        t: pe.t,
        p: pe.p,
        rhomolar_vap: pe.rhomolar_vap,
        rhomolar_liq: pe.rhomolar_liq,
        x: pe.x,
        y: pe.y,
    }))
}

/// Build the spinodal curve.
///
/// Maps to the C function `AbstractState_build_spinodal`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/spinodal/build",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    responses(
        (status = 200, description = "Spinodal built", body = Ack),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn build_spinodal(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
) -> ApiResult<Json<Ack>> {
    app.require_handle(handle)?;
    safe::abstract_state_build_spinodal(handle)?;
    Ok(Json(Ack { success: true }))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SpinodalResponse {
    pub tau: Vec<f64>,
    pub delta: Vec<f64>,
    pub m1: Vec<f64>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SpinodalParams {
    /// Buffer capacity for points (default 10000).
    #[serde(default)]
    pub max_length: Option<usize>,
}

/// Get spinodal curve data. The C API reports no actual length; trailing
/// unfilled entries are trimmed.
///
/// Maps to the C function `AbstractState_get_spinodal_data`.
#[utoipa::path(
    get,
    path = "/api/v1/abstract-state/{handle}/spinodal",
    params(
        ("handle" = i64, Path, description = "AbstractState handle"),
        SpinodalParams,
    ),
    responses(
        (status = 200, description = "Spinodal data", body = SpinodalResponse),
        (status = 400, description = "Buffer too small or spinodal not built", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn get_spinodal(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    axum::extract::Query(q): axum::extract::Query<SpinodalParams>,
) -> ApiResult<Json<SpinodalResponse>> {
    app.require_handle(handle)?;
    let sp = safe::abstract_state_get_spinodal(
        handle,
        q.max_length.unwrap_or(10_000).clamp(1, 1_000_000),
    )?;
    Ok(Json(SpinodalResponse {
        tau: sp.tau,
        delta: sp.delta,
        m1: sp.m1,
    }))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CriticalPointDto {
    pub t: f64,
    pub p: f64,
    pub rhomolar: f64,
    pub stable: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CriticalPointsResponse {
    pub points: Vec<CriticalPointDto>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct CriticalPointsParams {
    /// Buffer capacity for points (default 10).
    #[serde(default)]
    pub max_points: Option<usize>,
}

/// All critical points for the current composition. The C API reports no
/// actual count; unfilled entries are trimmed heuristically.
///
/// Maps to the C function `AbstractState_all_critical_points`.
#[utoipa::path(
    get,
    path = "/api/v1/abstract-state/{handle}/all-critical-points",
    params(
        ("handle" = i64, Path, description = "AbstractState handle"),
        CriticalPointsParams,
    ),
    responses(
        (status = 200, description = "Critical points", body = CriticalPointsResponse),
        (status = 400, description = "Buffer too small", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn all_critical_points(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    axum::extract::Query(q): axum::extract::Query<CriticalPointsParams>,
) -> ApiResult<Json<CriticalPointsResponse>> {
    app.require_handle(handle)?;
    let pts =
        safe::abstract_state_all_critical_points(handle, q.max_points.unwrap_or(10).clamp(1, 100))?;
    Ok(Json(CriticalPointsResponse {
        points: pts
            .into_iter()
            .map(|p| CriticalPointDto {
                t: p.t,
                p: p.p,
                rhomolar: p.rhomolar,
                stable: p.stable,
            })
            .collect(),
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct KeyedOutputSatStateRequest {
    /// `"liquid"` or `"gas"`.
    pub saturated_state: String,
    /// Parameter name or index.
    pub param: Param,
}

/// Keyed output for the saturated liquid or vapor state (two-phase region).
///
/// Maps to the C function `AbstractState_keyed_output_satState`.
#[utoipa::path(
    post,
    path = "/api/v1/abstract-state/{handle}/keyed-output/sat-state",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    request_body = KeyedOutputSatStateRequest,
    responses(
        (status = 200, description = "Output value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the input", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn keyed_output_sat_state(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    Json(req): Json<KeyedOutputSatStateRequest>,
) -> ApiResult<Json<DoubleValue>> {
    app.require_handle(handle)?;
    let p = req.param.resolve()?;
    Ok(Json(DoubleValue {
        value: safe::abstract_state_keyed_output_sat_state(handle, &req.saturated_state, p)?,
    }))
}

/// Name of the backend used by this AbstractState.
///
/// Maps to the C function `AbstractState_backend_name`.
#[utoipa::path(
    get,
    path = "/api/v1/abstract-state/{handle}/backend-name",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    responses(
        (status = 200, description = "Backend name", body = StringValue),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn backend_name(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
) -> ApiResult<Json<StringValue>> {
    app.require_handle(handle)?;
    Ok(Json(StringValue {
        value: safe::abstract_state_backend_name(handle)?,
    }))
}

/// Fluid names of this AbstractState.
///
/// Maps to the C function `AbstractState_fluid_names`.
#[utoipa::path(
    get,
    path = "/api/v1/abstract-state/{handle}/fluid-names",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    responses(
        (status = 200, description = "Fluid names", body = crate::dto::StringArrayValue),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn fluid_names(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
) -> ApiResult<Json<crate::dto::StringArrayValue>> {
    app.require_handle(handle)?;
    let names = safe::abstract_state_fluid_names(handle)?;
    Ok(Json(crate::dto::StringArrayValue { values: names }))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PhaseResponse {
    /// Raw phase index from CoolProp (`iphase_*` enum).
    pub phase_index: i32,
    /// Phase name, e.g. `"liquid"`, `"gas"`, `"twophase"`,
    /// `"supercritical"`, ...; `"phase_<index>"` for unmapped values.
    pub phase: String,
}

fn phase_name(index: i32) -> String {
    match index {
        0 => "liquid".into(),
        1 => "supercritical".into(),
        2 => "supercritical_gas".into(),
        3 => "supercritical_liquid".into(),
        4 => "critical_point".into(),
        5 => "gas".into(),
        6 => "twophase".into(),
        7 => "unknown".into(),
        8 => "not_imposed".into(),
        other => format!("phase_{other}"),
    }
}

/// Phase index/name of the current state.
///
/// Maps to the C function `AbstractState_phase`.
#[utoipa::path(
    get,
    path = "/api/v1/abstract-state/{handle}/phase",
    params(("handle" = i64, Path, description = "AbstractState handle")),
    responses(
        (status = 200, description = "Phase", body = PhaseResponse),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn phase(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
) -> ApiResult<Json<PhaseResponse>> {
    app.require_handle(handle)?;
    let idx = safe::abstract_state_phase(handle)?;
    Ok(Json(PhaseResponse {
        phase_index: idx,
        phase: phase_name(idx),
    }))
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct FluidParamStringParams {
    /// Fluid parameter, e.g. `"pure"`, `"CAS"`, `"BibTeX-Key"` ...
    pub param: String,
}

/// String metadata about the fluid(s) of this AbstractState.
///
/// Maps to the C function `AbstractState_fluid_param_string`.
#[utoipa::path(
    get,
    path = "/api/v1/abstract-state/{handle}/fluid-param-string",
    params(
        ("handle" = i64, Path, description = "AbstractState handle"),
        FluidParamStringParams,
    ),
    responses(
        (status = 200, description = "Parameter string", body = StringValue),
        (status = 400, description = "CoolProp rejected the parameter", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn fluid_param_string(
    State(app): State<AppState>,
    Path(handle): Path<i64>,
    axum::extract::Query(q): axum::extract::Query<FluidParamStringParams>,
) -> ApiResult<Json<StringValue>> {
    app.require_handle(handle)?;
    Ok(Json(StringValue {
        value: safe::abstract_state_fluid_param_string(handle, &q.param)?,
    }))
}

/// Keyed output of the saturated liquid state (two-phase region).
///
/// Maps to the C function `AbstractState_saturated_liquid_keyed_output`.
#[utoipa::path(
    get,
    path = "/api/v1/abstract-state/{handle}/saturated-liquid-output/{param}",
    params(
        ("handle" = i64, Path, description = "AbstractState handle"),
        ("param" = String, Path, description = "Parameter name or numeric index"),
    ),
    responses(
        (status = 200, description = "Output value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the parameter", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn saturated_liquid_keyed_output(
    State(app): State<AppState>,
    Path((handle, param)): Path<(i64, Param)>,
) -> ApiResult<Json<DoubleValue>> {
    app.require_handle(handle)?;
    let p = param.resolve()?;
    Ok(Json(DoubleValue {
        value: safe::abstract_state_saturated_liquid_keyed_output(handle, p)?,
    }))
}

/// Keyed output of the saturated vapor state (two-phase region).
///
/// Maps to the C function `AbstractState_saturated_vapor_keyed_output`.
#[utoipa::path(
    get,
    path = "/api/v1/abstract-state/{handle}/saturated-vapor-output/{param}",
    params(
        ("handle" = i64, Path, description = "AbstractState handle"),
        ("param" = String, Path, description = "Parameter name or numeric index"),
    ),
    responses(
        (status = 200, description = "Output value", body = DoubleValue),
        (status = 400, description = "CoolProp rejected the parameter", body = crate::error::ErrorEnvelope),
        (status = 404, description = "Unknown handle", body = crate::error::ErrorEnvelope),
    )
)]
pub async fn saturated_vapor_keyed_output(
    State(app): State<AppState>,
    Path((handle, param)): Path<(i64, Param)>,
) -> ApiResult<Json<DoubleValue>> {
    app.require_handle(handle)?;
    let p = param.resolve()?;
    Ok(Json(DoubleValue {
        value: safe::abstract_state_saturated_vapor_keyed_output(handle, p)?,
    }))
}
