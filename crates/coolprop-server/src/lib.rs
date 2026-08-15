//! CoolProp REST server: a complete HTTP wrapper around the CoolProp C API,
//! with an OpenAPI spec covering every exported function.
//!
//! Layout:
//! - [`safe`] — safe wrapper over the FFI crate
//! - [`routes`] — one module per API area, each handler annotated with
//!   `#[utoipa::path]` for the OpenAPI spec
//! - [`coverage`] — symbol → route table machine-checked against the
//!   vendored CoolPropLib.h by the `coverage` integration test

pub mod coverage;
pub mod dto;
pub mod error;
pub mod routes;
pub mod safe;
pub mod state;

use std::sync::OnceLock;

use axum::routing::get;
use axum::{Json, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "CoolProp Server",
        version = env!("CARGO_PKG_VERSION"),
        description = "REST API wrapping the complete C API of CoolProp v8.0.0 \
                       (thermophysical properties of pure fluids, mixtures and humid air). \
                       Every one of the 71 functions exported by CoolPropLib.h is exposed \
                       by exactly one endpoint — see the `coverage` module for the mapping."
    ),
    paths(
        routes::props::props_si,
        routes::props::props_1si,
        routes::props::props_si_multi,
        routes::props::props_1si_multi,
        routes::props::phase_si,
        routes::props::props_legacy,
        routes::props::props_legacy_s,
        routes::props::props_legacy_1,
        routes::humid_air::ha_props_si,
        routes::humid_air::ha_props,
        routes::humid_air::cair_sat,
        routes::info::get_global_param_string,
        routes::info::get_parameter_information_string,
        routes::info::get_fluid_param_string,
        routes::info::get_fluid_param_string_len,
        routes::info::get_param_index,
        routes::info::get_input_pair_index,
        routes::info::is_valid_fluid_string,
        routes::info::extract_backend,
        routes::info::add_fluids_as_json,
        routes::info::f2k,
        routes::info::k2f,
        routes::info::saturation_ancillary,
        routes::info::get_debug_level,
        routes::config::set_config_string,
        routes::config::set_config_double,
        routes::config::set_config_bool,
        routes::config::set_departure_functions,
        routes::config::set_reference_state_s,
        routes::config::set_reference_state_d,
        routes::config::set_debug_level,
        routes::config::redirect_stdout,
        routes::fortran::fortran_propssi,
        routes::fortran::fortran_hapropssi,
        routes::fortran::fortran_haprops,
        routes::abstract_state::create,
        routes::abstract_state::free,
        routes::abstract_state::set_fractions,
        routes::abstract_state::get_mole_fractions,
        routes::abstract_state::get_mole_fractions_sat_state,
        routes::abstract_state::get_fugacity,
        routes::abstract_state::get_fugacity_coefficient,
        routes::abstract_state::update,
        routes::abstract_state::specify_phase,
        routes::abstract_state::unspecify_phase,
        routes::abstract_state::keyed_output,
        routes::abstract_state::keyed_output_sat_state,
        routes::abstract_state::first_saturation_deriv,
        routes::abstract_state::first_partial_deriv,
        routes::abstract_state::second_partial_deriv,
        routes::abstract_state::second_two_phase_deriv,
        routes::abstract_state::first_two_phase_deriv,
        routes::abstract_state::first_two_phase_deriv_splined,
        routes::abstract_state::update_and_common_out,
        routes::abstract_state::update_and_1_out,
        routes::abstract_state::update_and_5_out,
        routes::abstract_state::set_binary_interaction,
        routes::abstract_state::set_cubic_alpha_c,
        routes::abstract_state::set_fluid_parameter_double,
        routes::abstract_state::build_phase_envelope,
        routes::abstract_state::get_phase_envelope,
        routes::abstract_state::get_phase_envelope_raw,
        routes::abstract_state::build_spinodal,
        routes::abstract_state::get_spinodal,
        routes::abstract_state::all_critical_points,
        routes::abstract_state::backend_name,
        routes::abstract_state::fluid_names,
        routes::abstract_state::phase,
        routes::abstract_state::fluid_param_string,
        routes::abstract_state::saturated_liquid_keyed_output,
        routes::abstract_state::saturated_vapor_keyed_output,
        health,
    ),
    components(
        schemas(
            dto::Param,
            dto::InputPair,
            dto::DoubleValue,
            dto::StringValue,
            dto::IndexValue,
            dto::LengthValue,
            dto::FlagValue,
            dto::Ack,
            dto::MatrixValue,
            dto::ArrayValue,
            error::ErrorEnvelope,
            error::ErrorDetail,
            routes::props::PropsSiRequest,
            routes::props::Props1SiRequest,
            routes::props::PropsSiMultiRequest,
            routes::props::Props1SiMultiRequest,
            routes::props::PropsLegacyRequest,
            routes::humid_air::HaPropsRequest,
            routes::humid_air::CairSatRequest,
            routes::info::ExtractBackendRequest,
            routes::info::BackendFluid,
            routes::info::AddFluidsRequest,
            routes::config::SetConfigStringRequest,
            routes::config::SetConfigDoubleRequest,
            routes::config::SetConfigBoolRequest,
            routes::config::SetDepartureFunctionsRequest,
            routes::config::SetReferenceStateSRequest,
            routes::config::SetReferenceStateDRequest,
            routes::config::DebugLevelRequest,
            routes::config::RedirectStdoutRequest,
            routes::abstract_state::CreateRequest,
            routes::abstract_state::HandleResponse,
            routes::abstract_state::SetFractionsRequest,
            routes::abstract_state::UpdateRequest,
            routes::abstract_state::PhaseRequest,
            routes::abstract_state::FirstSaturationDerivRequest,
            routes::abstract_state::FirstPartialDerivRequest,
            routes::abstract_state::SecondPartialDerivRequest,
            routes::abstract_state::FirstTwoPhaseDerivRequest,
            routes::abstract_state::UpdateAndOutRequest,
            routes::abstract_state::CommonOutResponse,
            routes::abstract_state::Out1Response,
            routes::abstract_state::Out5Response,
            routes::abstract_state::BinaryInteractionRequest,
            routes::abstract_state::CubicAlphaCRequest,
            routes::abstract_state::FluidParameterDoubleRequest,
            routes::abstract_state::BuildPhaseEnvelopeRequest,
            routes::abstract_state::PhaseEnvelopeResponse,
            routes::abstract_state::SpinodalResponse,
            routes::abstract_state::CriticalPointDto,
            routes::abstract_state::CriticalPointsResponse,
            routes::abstract_state::KeyedOutputSatStateRequest,
            routes::abstract_state::PhaseResponse,
            dto::StringArrayValue,
        ),
    )
)]
struct ApiDoc;

/// The generated OpenAPI document (cached).
pub fn openapi_spec() -> &'static utoipa::openapi::OpenApi {
    static SPEC: OnceLock<utoipa::openapi::OpenApi> = OnceLock::new();
    SPEC.get_or_init(ApiDoc::openapi)
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct HealthResponse {
    status: &'static str,
}

/// Liveness probe.
#[utoipa::path(get, path = "/health", responses((status = 200, description = "Server is up", body = HealthResponse)))]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

/// The full application router: API routes, `/health`, `/openapi.json` and
/// Swagger UI.
pub fn router() -> Router {
    let swagger = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi());
    routes::router()
        .with_state(state::AppState::new())
        .route("/health", get(health))
        .route(
            "/openapi.json",
            get(|| async { Json(openapi_spec().clone()) }),
        )
        .merge(swagger)
}
