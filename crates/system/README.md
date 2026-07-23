# nautilus-system

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-system)](https://docs.rs/nautilus-system/latest/nautilus-system/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-system.svg)](https://crates.io/crates/nautilus-system)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

System-level components and orchestration for NTPRO.

The `nautilus-system` crate provides the core system architecture for orchestrating trading systems,
including the kernel that manages all engines, configuration management,
and system-level factories for creating components:

- `NautilusKernel` - Core system orchestrator managing engines and components.
- `NautilusKernelConfig` - Configuration for kernel initialization.
- System builders and factories for component creation.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

- `streaming`: Enables `persistence` dependency for streaming configuration.
- `defi`: Enables DeFi (Decentralized Finance) support.
- `live`: Enables live trading mode dependencies.
- `tracing-bridge`: Enables the `tracing` subscriber bridge for log integration.

## Documentation

See [the docs](https://docs.rs/nautilus-system) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
