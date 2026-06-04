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

//! Portfolio analysis and performance metrics for NTPRO.
//!
//! The `nautilus-analysis` crate provides portfolio analysis tools and performance
//! statistics for evaluating trading strategies and portfolios. This includes return-based metrics,
//! PnL-based statistics, and risk measurements commonly used in quantitative finance:
//!
//! - Portfolio analyzer for tracking account states and positions.
//! - Extensive collection of performance statistics and risk metrics.
//! - Flexible statistic calculation framework supporting different data sources.
//! - Support for multi-currency portfolios and unrealized PnL calculations.
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
//! This crate currently builds as a Rust-only library and does not expose optional
//! product features.

#![warn(rustc::all)]
#![warn(clippy::pedantic)]
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(nonstandard_style)]
#![deny(missing_debug_implementations)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
#![deny(rustdoc::broken_intra_doc_links)]
#![allow(
    clippy::similar_names,
    reason = "domain terms such as returns/realized and pnl/pnls are intentionally parallel"
)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    reason = "analysis math casts between usize/i32/i64/f64 with values bounded by sample counts"
)]
#![cfg_attr(
    test,
    allow(
        clippy::float_cmp,
        clippy::unreadable_literal,
        reason = "analysis tests assert exact float outputs and reference statistic constants"
    )
)]

pub mod analyzer;
pub mod statistic;
pub mod statistics;

use std::collections::BTreeMap;

use nautilus_core::UnixNanos;

/// Type alias for time-indexed returns data used in portfolio analysis.
///
/// Maps timestamps to return values for time-series analysis of portfolio performance.
pub type Returns = BTreeMap<UnixNanos, f64>;
