# nautilus-infrastructure

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-infrastructure)](https://docs.rs/nautilus-infrastructure/latest/nautilus-infrastructure/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-infrastructure.svg)](https://crates.io/crates/nautilus-infrastructure)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

Database and messaging infrastructure for NTPRO.

The `nautilus-infrastructure` crate provides backend database implementations
and message bus adapters that support release-scoped local and service
deployments. This includes configurable data persistence and messaging
capabilities:

- **Redis integration**: Cache database and message bus implementations using Redis.
- **PostgreSQL integration**: SQL-based cache database with full data models.
- **Connection management**: Connection handling with retry logic and health monitoring.
- **Serialization options**: Support for JSON and MessagePack encoding formats.

The crate supports multiple database backends through feature flags, allowing users to choose
the appropriate infrastructure components for their specific deployment requirements and scale.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

- `redis`: Enables the Redis cache database and message bus backing implementations.
- `postgres`: Enables the PostgreSQL SQLx models and cache database backend.

## Documentation

See [the docs](https://docs.rs/nautilus-infrastructure) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
