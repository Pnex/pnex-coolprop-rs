//! Machine-checkable mapping between every symbol exported by
//! `vendor/CoolProp/include/CoolProp/CoolPropLib.h` (v8.0.0) and the route
//! that exposes it.
//!
//! The `coverage` integration test parses the vendored header and asserts
//! that every exported C function appears exactly once in this table —
//! this is the guarantee that the OpenAPI spec covers 100% of the CoolProp
//! C API.

/// (C symbol, HTTP method + route) — 71 entries, one per CoolPropLib.h export.
pub const COOLPROP_SYMBOL_TO_ROUTE: &[(&str, &str)] = &[
    // High-level property accessors
    ("Props1SI", "POST /api/v1/props/1si"),
    ("Props1SImulti", "POST /api/v1/props/1si-multi"),
    ("PropsSI", "POST /api/v1/props/si"),
    ("PropsSImulti", "POST /api/v1/props/si-multi"),
    ("PhaseSI", "POST /api/v1/props/phase"),
    // Parameter / fluid information
    (
        "get_global_param_string",
        "GET /api/v1/params/global/{param}",
    ),
    (
        "get_parameter_information_string",
        "GET /api/v1/params/information/{param}",
    ),
    (
        "get_fluid_param_string",
        "GET /api/v1/fluids/{fluid}/param/{param}",
    ),
    (
        "get_fluid_param_string_len",
        "GET /api/v1/fluids/{fluid}/param/{param}/length",
    ),
    // Configuration
    ("set_config_string", "PUT /api/v1/config/string"),
    ("set_config_double", "PUT /api/v1/config/double"),
    ("set_config_bool", "PUT /api/v1/config/bool"),
    (
        "set_departure_functions",
        "POST /api/v1/config/departure-functions",
    ),
    (
        "set_reference_stateS",
        "POST /api/v1/config/reference-state/S",
    ),
    (
        "set_reference_stateD",
        "POST /api/v1/config/reference-state/D",
    ),
    // FORTRAN 77 style wrappers
    ("propssi_", "POST /api/v1/fortran/propssi"),
    ("hapropssi_", "POST /api/v1/fortran/hapropssi"),
    ("haprops_", "POST /api/v1/fortran/haprops"),
    // Misc utilities
    ("F2K", "GET /api/v1/misc/f2k"),
    ("K2F", "GET /api/v1/misc/k2f"),
    ("get_param_index", "GET /api/v1/params/index"),
    ("get_input_pair_index", "GET /api/v1/input-pairs/index"),
    ("redirect_stdout", "POST /api/v1/admin/redirect-stdout"),
    ("get_debug_level", "GET /api/v1/misc/debug-level"),
    ("set_debug_level", "PUT /api/v1/misc/debug-level"),
    (
        "saturation_ancillary",
        "GET /api/v1/misc/saturation-ancillary",
    ),
    // Humid air
    ("HAPropsSI", "POST /api/v1/ha/props-si"),
    ("cair_sat", "POST /api/v1/ha/cair-sat"),
    ("HAProps", "POST /api/v1/ha/props"),
    // AbstractState
    ("AbstractState_factory", "POST /api/v1/abstract-state"),
    (
        "AbstractState_fluid_names",
        "GET /api/v1/abstract-state/{handle}/fluid-names",
    ),
    (
        "AbstractState_free",
        "DELETE /api/v1/abstract-state/{handle}",
    ),
    (
        "AbstractState_set_fractions",
        "POST /api/v1/abstract-state/{handle}/fractions",
    ),
    (
        "AbstractState_get_mole_fractions",
        "GET /api/v1/abstract-state/{handle}/mole-fractions",
    ),
    (
        "AbstractState_get_mole_fractions_satState",
        "GET /api/v1/abstract-state/{handle}/mole-fractions/sat-state",
    ),
    (
        "AbstractState_get_fugacity",
        "GET /api/v1/abstract-state/{handle}/fugacity/{i}",
    ),
    (
        "AbstractState_get_fugacity_coefficient",
        "GET /api/v1/abstract-state/{handle}/fugacity-coefficient/{i}",
    ),
    (
        "AbstractState_update",
        "POST /api/v1/abstract-state/{handle}/update",
    ),
    (
        "AbstractState_specify_phase",
        "POST /api/v1/abstract-state/{handle}/specify-phase",
    ),
    (
        "AbstractState_unspecify_phase",
        "POST /api/v1/abstract-state/{handle}/unspecify-phase",
    ),
    (
        "AbstractState_keyed_output",
        "GET /api/v1/abstract-state/{handle}/keyed-output/{param}",
    ),
    (
        "AbstractState_first_saturation_deriv",
        "POST /api/v1/abstract-state/{handle}/first-saturation-deriv",
    ),
    (
        "AbstractState_first_partial_deriv",
        "POST /api/v1/abstract-state/{handle}/first-partial-deriv",
    ),
    (
        "AbstractState_second_two_phase_deriv",
        "POST /api/v1/abstract-state/{handle}/second-two-phase-deriv",
    ),
    (
        "AbstractState_second_partial_deriv",
        "POST /api/v1/abstract-state/{handle}/second-partial-deriv",
    ),
    (
        "AbstractState_first_two_phase_deriv_splined",
        "POST /api/v1/abstract-state/{handle}/first-two-phase-deriv-splined",
    ),
    (
        "AbstractState_first_two_phase_deriv",
        "POST /api/v1/abstract-state/{handle}/first-two-phase-deriv",
    ),
    (
        "AbstractState_update_and_common_out",
        "POST /api/v1/abstract-state/{handle}/update-and-common-out",
    ),
    (
        "AbstractState_update_and_1_out",
        "POST /api/v1/abstract-state/{handle}/update-and-1-out",
    ),
    (
        "AbstractState_update_and_5_out",
        "POST /api/v1/abstract-state/{handle}/update-and-5-out",
    ),
    (
        "AbstractState_set_binary_interaction_double",
        "POST /api/v1/abstract-state/{handle}/binary-interaction",
    ),
    (
        "AbstractState_set_cubic_alpha_C",
        "POST /api/v1/abstract-state/{handle}/cubic-alpha-c",
    ),
    (
        "AbstractState_set_fluid_parameter_double",
        "POST /api/v1/abstract-state/{handle}/fluid-parameter-double",
    ),
    (
        "AbstractState_build_phase_envelope",
        "POST /api/v1/abstract-state/{handle}/phase-envelope/build",
    ),
    (
        "AbstractState_get_phase_envelope_data",
        "GET /api/v1/abstract-state/{handle}/phase-envelope/raw",
    ),
    (
        "AbstractState_get_phase_envelope_data_checkedMemory",
        "GET /api/v1/abstract-state/{handle}/phase-envelope",
    ),
    (
        "AbstractState_build_spinodal",
        "POST /api/v1/abstract-state/{handle}/spinodal/build",
    ),
    (
        "AbstractState_get_spinodal_data",
        "GET /api/v1/abstract-state/{handle}/spinodal",
    ),
    (
        "AbstractState_all_critical_points",
        "GET /api/v1/abstract-state/{handle}/all-critical-points",
    ),
    (
        "AbstractState_keyed_output_satState",
        "POST /api/v1/abstract-state/{handle}/keyed-output/sat-state",
    ),
    (
        "AbstractState_backend_name",
        "GET /api/v1/abstract-state/{handle}/backend-name",
    ),
    (
        "AbstractState_fluid_param_string",
        "GET /api/v1/abstract-state/{handle}/fluid-param-string",
    ),
    (
        "AbstractState_phase",
        "GET /api/v1/abstract-state/{handle}/phase",
    ),
    (
        "AbstractState_saturated_liquid_keyed_output",
        "GET /api/v1/abstract-state/{handle}/saturated-liquid-output/{param}",
    ),
    (
        "AbstractState_saturated_vapor_keyed_output",
        "GET /api/v1/abstract-state/{handle}/saturated-vapor-output/{param}",
    ),
    // Fluid library management
    ("add_fluids_as_JSON", "POST /api/v1/fluids/add-json"),
    ("C_is_valid_fluid_string", "GET /api/v1/fluids/is-valid"),
    ("C_extract_backend", "POST /api/v1/fluids/extract-backend"),
    // Deprecated KSI-unit accessors
    ("PropsS", "POST /api/v1/props/legacy-s"),
    ("Props", "POST /api/v1/props/legacy"),
    ("Props1", "POST /api/v1/props/legacy/1"),
];

/// Number of exported functions covered (v8.0.0 exports exactly 71).
#[cfg(test)]
mod tests {
    #[test]
    fn table_has_no_duplicates() {
        let mut names: Vec<_> = super::COOLPROP_SYMBOL_TO_ROUTE
            .iter()
            .map(|(s, _)| *s)
            .collect();
        let total = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(total, names.len(), "duplicate symbols in coverage table");
    }
}
