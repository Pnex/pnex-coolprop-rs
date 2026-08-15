# syntax=docker/dockerfile:1

# ---------------------------------------------------------------------------
# Build stage — latest stable Rust on Debian 13 (trixie).
#
# Why trixie and not alpine: the server links the vendored CoolProp C++
# library against libstdc++/glibc (see crates/coolprop-sys/build.rs), so a
# musl/alpine build would require patching the link setup for no real gain —
# the runtime stage below is distroless anyway. Trixie ships GCC 14 (modern
# C++ toolchain), glibc 2.41 and matches the distroless cc-debian13 runtime
# exactly. `git` is preinstalled (build.rs clones the pinned CoolProp v8.0.0
# and applies patches/coolprop-exception-guards.patch via git apply).
# ---------------------------------------------------------------------------
FROM rust:1-trixie AS builder

# cmake drives the CoolProp C++ build through the `cmake` crate; it is not
# part of the base image. ninja is picked up automatically by the cmake crate
# and compiles CoolProp much faster than the default Make generator.
RUN apt-get update \
 && apt-get install -y --no-install-recommends cmake ninja-build \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Everything needed to build: workspace manifests, both crates, and the
# CoolProp patch. vendor/ and target/ are excluded via .dockerignore.
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY patches ./patches

# Cache mounts keep the cargo registry, the compiled target dir (including
# the built libCoolProp.a) and the cloned CoolProp sources warm across
# builds. On a cold cache, build.rs clones CoolProp at the pinned tag v8.0.0.
# The cache mounts shadow /app/target and /app/vendor only for this RUN, so
# the finished binary is copied out to a path that persists in the layer.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/app/vendor \
    cargo build --release -p coolprop-server \
 && cp target/release/coolprop-server /tmp/coolprop-server

# ---------------------------------------------------------------------------
# Runtime stage — distroless, non-root, no shell, no package manager.
#
# cc-debian13 carries glibc, libm, libgcc_s and libstdc++ (the old `cxx`
# variant was folded into `cc`), which is exactly what the binary needs:
# libCoolProp.a is linked statically; only the C++ runtime is dynamic.
# ---------------------------------------------------------------------------
FROM gcr.io/distroless/cc-debian13:nonroot AS runtime

LABEL org.opencontainers.image.title="coolprop-rs" \
      org.opencontainers.image.description="REST server exposing the complete CoolProp C API with OpenAPI docs" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.base.name="gcr.io/distroless/cc-debian13:nonroot"

ENV COOLPROP_SERVER_ADDR=0.0.0.0:8080

COPY --from=builder /tmp/coolprop-server /usr/local/bin/coolprop-server

# Bind metrics/docs port (server listens on 8080, unprivileged).
EXPOSE 8080

# The :nonroot tag already runs as uid 65532 (nonroot); keep it explicit.
USER nonroot:nonroot

ENTRYPOINT ["/usr/local/bin/coolprop-server"]
