# nautilus-binance

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg?branch=main)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-binance)](https://docs.rs/nautilus-binance/latest/nautilus-binance/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-binance.svg)](https://crates.io/crates/nautilus-binance)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

NTPRO adapter for the
[Binance](https://www.binance.com/) cryptocurrency exchange.

The `nautilus-binance` crate provides client bindings (HTTP & WebSocket), data models,
and helper utilities that wrap the official **Binance API**.

Current Rust runtime factory support is scoped to:

- Spot trading (api.binance.com)
- USD-M Futures (fapi.binance.com)
- COIN-M Futures (dapi.binance.com)

The product enum still models Margin and Options for compatibility with the wider
Binance domain, but the Rust runtime factories do not create Margin or Options
data/execution clients yet. Treat those surfaces as deferred until dedicated
factory and runtime support is added.

Configure one Binance runtime client per product target. The current Rust factory
boundary selects a single product type for a created client; it is not a
multi-product client registration path.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Authentication

This crate requires **Ed25519 API keys** for all authenticated endpoints (REST and WebSocket API).
Ed25519 is recommended by Binance for its superior performance and security. HMAC and RSA keys
are not supported.

Generate an Ed25519 keypair and register it with Binance:

```bash
# Generate private key (PKCS#8 PEM format)
openssl genpkey -algorithm ed25519 -out binance_ed25519_private.pem

# Extract public key for Binance registration
openssl pkey -in binance_ed25519_private.pem -pubout -out binance_ed25519_public.pem
```

Set credentials via environment variables:

```bash
export BINANCE_API_KEY="your-api-key-from-binance"
export BINANCE_API_SECRET="$(cat binance_ed25519_private.pem)"
```

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

[High-precision mode](https://github.com/atxinbao/NTPRO/blob/main/docs/getting_started/installation.md#precision-mode) (128-bit value types) is enabled by default.

## Documentation

See [the docs](https://docs.rs/nautilus-binance) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
