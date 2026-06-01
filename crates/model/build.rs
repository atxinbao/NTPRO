// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! Build script for the `nautilus-model` crate.
//!
//! In addition to the common tasks performed by the other build scripts (header generation,
//! rerun-tracking, docs.rs early-exit) this script also toggles *high-precision* mode for the
//! generated bindings based on either:
//!
//! 1. The `HIGH_PRECISION` environment variable, **or**
//! 2. The compile-time `high-precision` cargo feature.
//!
//! When enabled the flag is forwarded to the generated C header in Cargo's `OUT_DIR` so C ABI
//! consumers compile with the same precision configuration.

#[cfg(feature = "ffi")]
use std::env;

#[allow(
    clippy::expect_used,
    reason = "Build script may panic on misconfiguration; .expect() calls are behind #[cfg(feature = \"ffi\")]"
)]
#[allow(
    unused_assignments,
    reason = "Conditional compilation creates unused assignments"
)]
#[allow(unused_mut)]
fn main() {
    // Skip file generation if we're in the docs.rs environment
    if std::env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=Running in docs.rs environment, skipping file generation");
        return;
    }

    // Ensure the build script runs on changes
    println!("cargo:rerun-if-env-changed=HIGH_PRECISION");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_HIGH_PRECISION");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");
    println!("cargo:rerun-if-changed=../Cargo.toml");

    #[cfg(feature = "ffi")]
    if env::var("CARGO_FEATURE_FFI").is_ok() {
        extern crate cbindgen;
        use std::path::PathBuf;

        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        // Generate C headers
        let mut config_c = cbindgen::Config::from_file("cbindgen.toml")
            .expect("unable to find cbindgen.toml configuration file");

        // Check HIGH_PRECISION environment variable for C header too
        let high_precision_c = env::var("HIGH_PRECISION").map_or_else(
            |_| {
                #[cfg(feature = "high-precision")]
                {
                    true
                }
                #[cfg(not(feature = "high-precision"))]
                {
                    false
                }
            },
            |v| v.to_lowercase() == "true" || v == "1",
        );

        if high_precision_c && let Some(mut includes) = config_c.after_includes {
            includes.insert_str(0, "\n#define HIGH_PRECISION\n");
            config_c.after_includes = Some(includes);
        }

        let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
        let c_header_path = out_dir.join("model.h");
        cbindgen::generate_with_config(&crate_dir, config_c)
            .expect("unable to generate bindings")
            .write_to_file(&c_header_path);
    }
}
