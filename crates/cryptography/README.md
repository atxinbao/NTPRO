# nautilus-cryptography

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-cryptography)](https://docs.rs/nautilus-cryptography/latest/nautilus-cryptography/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-cryptography.svg)](https://crates.io/crates/nautilus-cryptography)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

Cryptographic utilities and security functions for NTPRO.

The `nautilus-cryptography` crate provides essential cryptographic primitives and security utilities
required for secure communication with trading venues and data providers. This includes
digital signing, TLS configuration, and cryptographic provider management:

- HMAC-based message authentication and signing.
- Digital signatures using RSA and Ed25519 algorithms.
- TLS client configuration with platform certificate verification.
- Cryptographic provider management and initialization.
- Secure encoding and decoding utilities.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

## Documentation

See [the docs](https://docs.rs/nautilus-cryptography) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
