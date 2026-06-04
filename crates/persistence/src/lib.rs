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

//! Data persistence and storage management for NTPRO.
//!
//! The `nautilus-persistence` crate provides data persistence capabilities for storing and retrieving
//! trading data, state, and configuration.
//!
//! # NTPRO
//!
//! NTPRO is an open-source, production-grade, Rust-native
//! engine for multi-asset, multi-venue trading systems.
//!
//! The system spans research, deterministic simulation, and live execution within a single
//! event-driven architecture, providing research-to-live semantic parity.
//!
//! # Feature Flags
//!
//! This crate provides feature flags to control source code inclusion during compilation.
//!
//! - `cloud`: Enables cloud storage backends (S3, Azure, GCP, HTTP) via `object_store`.
//! - `ffi`: Enables the C foreign function interface (FFI) from [cbindgen](https://github.com/mozilla/cbindgen).
//! - `high-precision`: Enables [high-precision mode](https://github.com/atxinbao/NTPRO/blob/main/docs/getting_started/installation.md#precision-mode) to use 128-bit value types.

#![warn(rustc::all)]
#![deny(nonstandard_style)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(rustdoc::broken_intra_doc_links)]
// #![deny(clippy::missing_errors_doc)]

pub mod backend;
pub mod parquet;
pub mod test_data;
