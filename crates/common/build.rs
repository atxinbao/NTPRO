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

//! Build script for the `nautilus-common` crate.
//!
//! The script mirrors the responsibilities of the build script in the `core` crate but is scoped
//! to the `common` module.  In summary it:
//!
//! * Emits `cargo:rerun-if-*` directives so that Cargo knows when to re-execute the script.
//! * Generates C header files into Cargo's `OUT_DIR` with
//!   [`cbindgen`](https://github.com/mozilla/cbindgen) whenever the `ffi` feature flag is enabled.
//! * Skips all generation work when it detects the docs.rs build environment because external
//!   file writes are disallowed there.

#[cfg(feature = "ffi")]
use std::env;

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
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=../Cargo.toml");

    #[cfg(feature = "ffi")]
    if env::var("CARGO_FEATURE_FFI").is_ok() {
        extern crate cbindgen;
        use std::path::PathBuf;

        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        // Generate C headers
        let config_c = cbindgen::Config::from_file("cbindgen.toml")
            .expect("unable to find cbindgen.toml configuration file");

        let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
        let c_header_path = out_dir.join("common.h");
        cbindgen::generate_with_config(&crate_dir, config_c)
            .expect("unable to generate bindings")
            .write_to_file(c_header_path);
    }
}
