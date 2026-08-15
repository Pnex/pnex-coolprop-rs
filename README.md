# coolprop-rs

[![CI](https://github.com/Pnex/pnex-coolprop-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Pnex/pnex-coolprop-rs/actions/workflows/ci.yml)

A Rust REST server that wraps the **complete C API of
[CoolProp](https://github.com/CoolProp/CoolProp) v8.0.0** — the
thermophysical property library for pure fluids, mixtures and humid air —
with a full OpenAPI spec and Swagger UI.

Every one of the **71 functions exported by `CoolPropLib.h`** is exposed by
exactly one HTTP endpoint. This is not just a claim: the
`coverage` integration test parses the vendored header, extracts every
exported symbol, and verifies that each one maps to a route that exists in
both the router and the generated OpenAPI document.

## Quick start

Prerequisites: Rust (stable), `cmake`, a C++17 compiler, and `git` (to fetch
the pinned CoolProp sources on first build — ~3–5 minutes).

```bash
make test    # full suite: unit + API + golden values + coverage check
make run     # serve on http://0.0.0.0:8080  (ADDR=1.2.3.4:9000 to override)
```

Then:

- Swagger UI: <http://localhost:8080/swagger-ui>
- OpenAPI document: <http://localhost:8080/openapi.json>
- Health: `GET /health`
- Dump the spec to disk: `make openapi` → `openapi.json`

## The example from the CoolProp README, over HTTP

```bash
# PropsSI("Dmolar","T",298,"P",1e5,"Propane[0.5]&Ethane[0.5]")   (default HEOS)
curl -s localhost:8080/api/v1/props/si -H 'content-type: application/json' -d '{
  "output": "Dmolar", "name1": "T", "prop1": 298,
  "name2": "P", "prop2": 1e5, "fluid": "Propane[0.5]&Ethane[0.5]"}'
# -> {"value": 40.7...}

# Same with an explicit backend (HEOS:: / REFPROP::)
curl -s localhost:8080/api/v1/props/si -H 'content-type: application/json' -d '{
  "output": "Dmolar", "name1": "T", "prop1": 298,
  "name2": "P", "prop2": 1e5, "fluid": "HEOS::Propane[0.5]&Ethane[0.5]"}'

# Vectorized PropsSImulti: one row per output, one column per input point
curl -s localhost:8080/api/v1/props/si-multi -H 'content-type: application/json' -d '{
  "outputs": ["Dmolar", "Hmolar"], "name1": "T", "prop1": [298, 300],
  "name2": "P", "prop2": [1e5, 1e5], "fluids": ["Propane", "Ethane"],
  "fractions": [0.5, 0.5]}'

# Low-level stateful AbstractState
h=$(curl -s localhost:8080/api/v1/abstract-state -H 'content-type: application/json' \
    -d '{"backend": "HEOS", "fluids": ["Propane", "Ethane"]}' | jq .handle)
curl -s -X POST localhost:8080/api/v1/abstract-state/$h/fractions \
    -H 'content-type: application/json' -d '{"fractions": [0.5, 0.5]}'
curl -s -X POST localhost:8080/api/v1/abstract-state/$h/update \
    -H 'content-type: application/json' \
    -d '{"input_pair": "PT_INPUTS", "value1": 101325, "value2": 298.15}'
curl -s localhost:8080/api/v1/abstract-state/$h/keyed-output/Dmolar
curl -s -X DELETE localhost:8080/api/v1/abstract-state/$h
```

## API surface

| Group | Endpoints under `/api/v1` | C functions covered |
|---|---|---|
| Props | `POST /props/si`, `/props/1si`, `/props/si-multi`, `/props/1si-multi`, `/props/phase`, `/props/legacy`, `/props/legacy-s`, `/props/legacy/1` | `PropsSI`, `Props1SI`, `PropsSImulti`, `Props1SImulti`, `PhaseSI`, `Props`, `PropsS`, `Props1` |
| Humid air | `POST /ha/props-si`, `/ha/props`, `/ha/cair-sat` | `HAPropsSI`, `HAProps`, `cair_sat` |
| Info / misc | `/params/global/{p}`, `/params/information/{p}`, `/fluids/{f}/param/{p}[/length]`, `/params/index`, `/input-pairs/index`, `/fluids/is-valid`, `POST /fluids/extract-backend`, `POST /fluids/add-json`, `/misc/f2k`, `/misc/k2f`, `/misc/saturation-ancillary` | `get_global_param_string`, `get_parameter_information_string`, `get_fluid_param_string(_len)`, `get_param_index`, `get_input_pair_index`, `C_is_valid_fluid_string`, `C_extract_backend`, `add_fluids_as_JSON`, `F2K`, `K2F`, `saturation_ancillary` |
| Config | `PUT /config/{string,double,bool}`, `POST /config/departure-functions`, `POST /config/reference-state/{S,D}`, `GET|PUT /misc/debug-level`, `POST /admin/redirect-stdout` | `set_config_*`, `set_departure_functions`, `set_reference_stateS/D`, `get/set_debug_level`, `redirect_stdout` |
| FORTRAN shims | `POST /fortran/{propssi,hapropssi,haprops}` | `propssi_`, `hapropssi_`, `haprops_` |
| AbstractState | `POST /abstract-state`, `DELETE /abstract-state/{h}`, and ~35 sub-endpoints (fractions, update, keyed outputs, derivatives, phase envelope, spinodal, critical points, ...) | all 38 `AbstractState_*` functions |

The complete machine-checked symbol → route table lives in
`crates/coolprop-server/src/coverage.rs`.

## Error model

Failures return `{"error": {"code": <http>, "message": "<CoolProp message>"}}`
— 400 for inputs CoolProp rejects, 404 for unknown AbstractState handles.
Errors are extracted from CoolProp's errstring / errcode conventions; the
`PhaseSI` "unknown: ..." spelling and the `_HUGE` sentinel are both handled.

## Layout

```
crates/coolprop-sys      FFI: build.rs compiles vendored CoolProp (static, extern "C"),
                         hand-written declarations for all 71 exports
crates/coolprop-server   safe wrapper, axum routes, utoipa OpenAPI, tests
patches/                 exception-guard patch applied to the vendored sources
vendor/CoolProp/         pinned clone of https://github.com/CoolProp/CoolProp (v8.0.0)
```

## Notes & caveats

- **REFPROP** is commercial software. `REFPROP::`-prefixed calls work only
  when `librefprop.so` is installed and findable; otherwise the server
  returns CoolProp's error (HTTP 400) — verified by the test suite.
- **Global state**: config, reference states, debug level and stdout
  redirection are process-global in CoolProp (and serialized behind a mutex).
- **Two-phase derivatives** (`first/second_two_phase_deriv*`) are only
  implemented for pure fluids in CoolProp v8; mixture calls return a clean
  400. The deprecated `Props1` kSI conversion rejects every modern output —
  also surfaced as a 400. Both are upstream behaviors, faithfully surfaced.
- AbstractState handles live in the server process and are lost on restart.
