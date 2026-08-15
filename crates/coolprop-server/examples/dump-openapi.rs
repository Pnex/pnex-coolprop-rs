//! Writes the full OpenAPI spec to `openapi.json` and `openapi.yaml` in the
//! repository root.
//!
//! Usage: `cargo run -p coolprop-server --example dump-openapi`
//! (or `task openapi`)

fn main() {
    let spec = coolprop_server::openapi_spec();
    let json_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../openapi.json");
    std::fs::write(json_path, serde_json::to_string_pretty(spec).unwrap())
        .unwrap_or_else(|e| panic!("could not write {json_path}: {e}"));
    let yaml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../openapi.yaml");
    std::fs::write(yaml_path, spec.to_yaml().unwrap())
        .unwrap_or_else(|e| panic!("could not write {yaml_path}: {e}"));
    println!(
        "wrote {json_path} and {yaml_path} ({} paths)",
        spec.paths.paths.len()
    );
}
