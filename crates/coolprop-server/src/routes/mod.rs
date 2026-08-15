//! Route modules, one per CoolProp API area.

pub mod abstract_state;
pub mod config;
pub mod fortran;
pub mod humid_air;
pub mod info;
pub mod props;

use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // Props family
        .route("/api/v1/props/si", post(props::props_si))
        .route("/api/v1/props/1si", post(props::props_1si))
        .route("/api/v1/props/si-multi", post(props::props_si_multi))
        .route("/api/v1/props/1si-multi", post(props::props_1si_multi))
        .route("/api/v1/props/phase", post(props::phase_si))
        .route("/api/v1/props/legacy", post(props::props_legacy))
        .route("/api/v1/props/legacy-s", post(props::props_legacy_s))
        .route("/api/v1/props/legacy/1", post(props::props_legacy_1))
        // Humid air
        .route("/api/v1/ha/props-si", post(humid_air::ha_props_si))
        .route("/api/v1/ha/props", post(humid_air::ha_props))
        .route("/api/v1/ha/cair-sat", post(humid_air::cair_sat))
        // Info / misc
        .route(
            "/api/v1/params/global/{param}",
            get(info::get_global_param_string),
        )
        .route(
            "/api/v1/params/information/{param}",
            get(info::get_parameter_information_string),
        )
        .route(
            "/api/v1/fluids/{fluid}/param/{param}",
            get(info::get_fluid_param_string),
        )
        .route(
            "/api/v1/fluids/{fluid}/param/{param}/length",
            get(info::get_fluid_param_string_len),
        )
        .route("/api/v1/params/index", get(info::get_param_index))
        .route("/api/v1/input-pairs/index", get(info::get_input_pair_index))
        .route("/api/v1/fluids/is-valid", get(info::is_valid_fluid_string))
        .route(
            "/api/v1/fluids/extract-backend",
            post(info::extract_backend),
        )
        .route("/api/v1/fluids/add-json", post(info::add_fluids_as_json))
        .route("/api/v1/misc/f2k", get(info::f2k))
        .route("/api/v1/misc/k2f", get(info::k2f))
        .route(
            "/api/v1/misc/saturation-ancillary",
            get(info::saturation_ancillary),
        )
        .route("/api/v1/misc/debug-level", get(info::get_debug_level))
        // Config / admin
        .route("/api/v1/config/string", put(config::set_config_string))
        .route("/api/v1/config/double", put(config::set_config_double))
        .route("/api/v1/config/bool", put(config::set_config_bool))
        .route(
            "/api/v1/config/departure-functions",
            post(config::set_departure_functions),
        )
        .route(
            "/api/v1/config/reference-state/S",
            post(config::set_reference_state_s),
        )
        .route(
            "/api/v1/config/reference-state/D",
            post(config::set_reference_state_d),
        )
        .route("/api/v1/misc/debug-level", put(config::set_debug_level))
        .route(
            "/api/v1/admin/redirect-stdout",
            post(config::redirect_stdout),
        )
        // FORTRAN shims
        .route("/api/v1/fortran/propssi", post(fortran::fortran_propssi))
        .route(
            "/api/v1/fortran/hapropssi",
            post(fortran::fortran_hapropssi),
        )
        .route("/api/v1/fortran/haprops", post(fortran::fortran_haprops))
        // AbstractState
        .route("/api/v1/abstract-state", post(abstract_state::create))
        .route(
            "/api/v1/abstract-state/{handle}",
            delete(abstract_state::free),
        )
        .route(
            "/api/v1/abstract-state/{handle}/fractions",
            post(abstract_state::set_fractions),
        )
        .route(
            "/api/v1/abstract-state/{handle}/mole-fractions",
            get(abstract_state::get_mole_fractions),
        )
        .route(
            "/api/v1/abstract-state/{handle}/mole-fractions/sat-state",
            get(abstract_state::get_mole_fractions_sat_state),
        )
        .route(
            "/api/v1/abstract-state/{handle}/fugacity/{i}",
            get(abstract_state::get_fugacity),
        )
        .route(
            "/api/v1/abstract-state/{handle}/fugacity-coefficient/{i}",
            get(abstract_state::get_fugacity_coefficient),
        )
        .route(
            "/api/v1/abstract-state/{handle}/update",
            post(abstract_state::update),
        )
        .route(
            "/api/v1/abstract-state/{handle}/specify-phase",
            post(abstract_state::specify_phase),
        )
        .route(
            "/api/v1/abstract-state/{handle}/unspecify-phase",
            post(abstract_state::unspecify_phase),
        )
        .route(
            "/api/v1/abstract-state/{handle}/keyed-output/{param}",
            get(abstract_state::keyed_output),
        )
        .route(
            "/api/v1/abstract-state/{handle}/keyed-output/sat-state",
            post(abstract_state::keyed_output_sat_state),
        )
        .route(
            "/api/v1/abstract-state/{handle}/first-saturation-deriv",
            post(abstract_state::first_saturation_deriv),
        )
        .route(
            "/api/v1/abstract-state/{handle}/first-partial-deriv",
            post(abstract_state::first_partial_deriv),
        )
        .route(
            "/api/v1/abstract-state/{handle}/second-partial-deriv",
            post(abstract_state::second_partial_deriv),
        )
        .route(
            "/api/v1/abstract-state/{handle}/second-two-phase-deriv",
            post(abstract_state::second_two_phase_deriv),
        )
        .route(
            "/api/v1/abstract-state/{handle}/first-two-phase-deriv",
            post(abstract_state::first_two_phase_deriv),
        )
        .route(
            "/api/v1/abstract-state/{handle}/first-two-phase-deriv-splined",
            post(abstract_state::first_two_phase_deriv_splined),
        )
        .route(
            "/api/v1/abstract-state/{handle}/update-and-common-out",
            post(abstract_state::update_and_common_out),
        )
        .route(
            "/api/v1/abstract-state/{handle}/update-and-1-out",
            post(abstract_state::update_and_1_out),
        )
        .route(
            "/api/v1/abstract-state/{handle}/update-and-5-out",
            post(abstract_state::update_and_5_out),
        )
        .route(
            "/api/v1/abstract-state/{handle}/binary-interaction",
            post(abstract_state::set_binary_interaction),
        )
        .route(
            "/api/v1/abstract-state/{handle}/cubic-alpha-c",
            post(abstract_state::set_cubic_alpha_c),
        )
        .route(
            "/api/v1/abstract-state/{handle}/fluid-parameter-double",
            post(abstract_state::set_fluid_parameter_double),
        )
        .route(
            "/api/v1/abstract-state/{handle}/phase-envelope/build",
            post(abstract_state::build_phase_envelope),
        )
        .route(
            "/api/v1/abstract-state/{handle}/phase-envelope",
            get(abstract_state::get_phase_envelope),
        )
        .route(
            "/api/v1/abstract-state/{handle}/phase-envelope/raw",
            get(abstract_state::get_phase_envelope_raw),
        )
        .route(
            "/api/v1/abstract-state/{handle}/spinodal/build",
            post(abstract_state::build_spinodal),
        )
        .route(
            "/api/v1/abstract-state/{handle}/spinodal",
            get(abstract_state::get_spinodal),
        )
        .route(
            "/api/v1/abstract-state/{handle}/all-critical-points",
            get(abstract_state::all_critical_points),
        )
        .route(
            "/api/v1/abstract-state/{handle}/backend-name",
            get(abstract_state::backend_name),
        )
        .route(
            "/api/v1/abstract-state/{handle}/fluid-names",
            get(abstract_state::fluid_names),
        )
        .route(
            "/api/v1/abstract-state/{handle}/phase",
            get(abstract_state::phase),
        )
        .route(
            "/api/v1/abstract-state/{handle}/fluid-param-string",
            get(abstract_state::fluid_param_string),
        )
        .route(
            "/api/v1/abstract-state/{handle}/saturated-liquid-output/{param}",
            get(abstract_state::saturated_liquid_keyed_output),
        )
        .route(
            "/api/v1/abstract-state/{handle}/saturated-vapor-output/{param}",
            get(abstract_state::saturated_vapor_keyed_output),
        )
}
