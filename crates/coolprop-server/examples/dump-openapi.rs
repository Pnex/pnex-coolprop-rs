//! Writes the full OpenAPI spec to `openapi.json` in the repository root.
//!
//! Usage: `cargo run -p coolprop-server --example dump-openapi`

fn main() {
    let spec = coolprop_server::openapi_spec();
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../openapi.json");
    std::fs::write(path, serde_json::to_string_pretty(spec).unwrap())
        .unwrap_or_else(|e| panic!("could not write {path}: {e}"));
    println!("wrote {path} ({} paths)", spec.paths.paths.len());
}
