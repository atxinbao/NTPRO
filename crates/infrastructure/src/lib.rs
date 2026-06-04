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

//! Database and messaging infrastructure for NTPRO.
//!
//! The `nautilus-infrastructure` crate provides backend database implementations and message bus adapters
//! that enable NTPRO to scale from development to production deployments. This includes
//! enterprise-grade data persistence and messaging capabilities:
//!
//! - **Redis integration**: Cache database and message bus implementations using Redis.
//! - **PostgreSQL integration**: SQL-based cache database with full data models.
//! - **Connection management**: Connection handling with retry logic and health monitoring.
//! - **Serialization options**: Support for JSON and MessagePack encoding formats.
//!
//! The crate supports multiple database backends through feature flags, allowing users to choose
//! the appropriate infrastructure components for their specific deployment requirements and scale.
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
//! - `redis`: Enables the Redis cache database and message bus backing implementations.
//! - `postgres`: Enables the PostgreSQL SQLx models and cache database backend.

#![warn(rustc::all)]
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(nonstandard_style)]
#![deny(missing_debug_implementations)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
#![deny(rustdoc::broken_intra_doc_links)]

#[cfg(feature = "redis")]
pub mod redis;

#[cfg(feature = "postgres")]
pub mod sql;
