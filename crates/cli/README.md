# nautilus-cli

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-cli)](https://docs.rs/nautilus-cli/latest/nautilus-cli/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-cli.svg)](https://crates.io/crates/nautilus-cli)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

Command-line interface and tools for NTPRO.

The `nautilus-cli` crate provides a command-line interface for managing and
operating NTPRO workspaces. It includes tools for database management,
system configuration, and operational utilities:

- Database initialization and management commands.
- PostgreSQL schema setup and maintenance.
- Configuration validation and setup utilities.
- System administration and operational tools.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation,
depending on the intended use case:

- `defi`: Enables blockchain/DeFi commands including block sync, DEX pool sync, and pool analysis.

## Documentation

See [the docs](https://docs.rs/nautilus-cli) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
