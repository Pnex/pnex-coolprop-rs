.PHONY: build test run openapi clean

# Builds everything, including the vendored CoolProp static library
# (auto-cloned at the pinned tag v8.0.0 on first run).
build:
	cargo build --release

# Full test suite: unit, API integration, golden values, and the
# machine-checked 100%-coverage test.
test:
	cargo test

# Run the server (default: http://0.0.0.0:8080, override with ADDR=...)
run:
	cargo run --release

# Dump the OpenAPI spec to ./openapi.json
openapi:
	cargo run -p coolprop-server --example dump-openapi

clean:
	cargo clean
