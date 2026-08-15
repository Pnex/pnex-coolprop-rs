//! Builds the vendored CoolProp C++ library (pinned at v8.0.0) as a static
//! library and links it into the Rust crate.
//!
//! If `vendor/CoolProp` is missing (it is gitignored), it is cloned from the
//! official repository at the pinned tag so that `cargo build` stays
//! self-contained.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const COOLPROP_TAG: &str = "v8.0.0";
const COOLPROP_REPO: &str = "https://github.com/CoolProp/CoolProp.git";

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("crate must live at <root>/crates/coolprop-sys")
        .to_path_buf();
    let vendor = workspace_root.join("vendor").join("CoolProp");

    ensure_vendored(&vendor);
    apply_patches(&workspace_root, &vendor);

    let dst = cmake::Config::new(&vendor)
        .define("COOLPROP_STATIC_LIBRARY", "ON")
        .define("COOLPROP_RELEASE", "ON")
        .define("COOLPROP_NO_EXAMPLES", "ON")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        // The static-library target does not add this define itself (only the
        // shared-library build does), but it is what makes EXPORT_CODE
        // `extern "C"` in CoolPropLib.h — without it every symbol is
        // C++-mangled and cannot be linked from Rust.
        .define("CMAKE_CXX_FLAGS", "-DCOOLPROP_LIB")
        .build();

    let lib_dir = find_library_dir(&dst).unwrap_or_else(|| {
        panic!(
            "could not locate libCoolProp.a under {} — check the CMake output above",
            dst.display()
        )
    });

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=CoolProp");
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=m");
    println!("cargo:rerun-if-changed={}", vendor.join("src").display());
}

fn ensure_vendored(vendor: &Path) {
    if vendor.join("CMakeLists.txt").exists() {
        return;
    }
    if vendor.exists() {
        std::fs::remove_dir_all(vendor)
            .expect("failed to remove incomplete vendor/CoolProp directory");
    }
    println!(
        "cargo:warning=cloning CoolProp {COOLPROP_TAG} into {} (one-time, ~2 min)",
        vendor.display()
    );
    let status = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--branch",
            COOLPROP_TAG,
            COOLPROP_REPO,
        ])
        .arg(vendor)
        .status()
        .expect("failed to spawn `git` to clone CoolProp");
    if !status.success() {
        panic!(
            "git clone of CoolProp {COOLPROP_TAG} failed; clone it manually:\n\
             git clone --depth 1 --branch {COOLPROP_TAG} {COOLPROP_REPO} {}",
            vendor.display()
        );
    }
}

/// Apply the carried patches to the vendored sources. Upstream CoolProp's
/// deprecated KSI wrappers (`Props1`, `PropsS`, `cair_sat`) do not catch C++
/// exceptions; an escaping exception crosses the `extern "C"` boundary and
/// aborts FFI callers. The patch adds the same try/catch the other exports
/// already have. Idempotent: `git apply --check` fails once applied.
fn apply_patches(workspace_root: &Path, vendor: &Path) {
    let patch = workspace_root.join("patches/coolprop-exception-guards.patch");
    if !patch.exists() {
        panic!(
            "missing required patch {} — restore it from the repository",
            patch.display()
        );
    }
    let patch_arg = patch.to_string_lossy().into_owned();
    let check = Command::new("git")
        .args([
            "-C",
            vendor.to_str().unwrap(),
            "apply",
            "--check",
            &patch_arg,
        ])
        .status()
        .expect("failed to spawn `git` for patch check");
    if check.success() {
        let status = Command::new("git")
            .args(["-C", vendor.to_str().unwrap(), "apply", &patch_arg])
            .status()
            .expect("failed to spawn `git` for patch apply");
        assert!(status.success(), "git apply of {} failed", patch.display());
        println!("cargo:warning=applied exception-guard patch to vendored CoolProp");
    }
}

/// Locate the directory containing libCoolProp.a (or CoolProp.lib on Windows).
fn find_library_dir(root: &Path) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let name = path.file_name()?.to_string_lossy().to_string();
                if name == "libCoolProp.a" || name == "CoolProp.lib" {
                    return path.parent().map(|p| p.to_path_buf());
                }
            }
        }
    }
    None
}
