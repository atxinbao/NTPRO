# nautilus-architect-ax

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-architect-ax)](https://docs.rs/nautilus-architect-ax/latest/nautilus-architect-ax/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-architect-ax.svg)](https://crates.io/crates/nautilus-architect-ax)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

NTPRO adapter for [AX Exchange](https://architect.exchange).

## Overview

[AX Exchange](https://architect.exchange) is the world's first centralized and regulated exchange
for perpetual futures on traditional underlying asset classes (FX, rates, metals, energy, stock
indexes). Designed for institutional and professional traders, it combines innovations from digital
asset perpetual exchanges with the safety and risk management of traditional futures exchanges.
Licensed under the [Bermuda Monetary Authority (BMA)](https://www.bma.bm/).

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

## Documentation

- [Crate docs](https://docs.rs/nautilus-architect-ax)
- [API reference](https://docs.architect.exchange/api-reference/)
- [AX Exchange](https://architect.exchange/)

## Authentication

AX Exchange uses bearer token authentication via HTTP headers:

1. API key and secret obtain a session token via `/authenticate`.
2. The session token is used as a bearer token for subsequent REST and WebSocket requests.

## API endpoints

| Environment | HTTP API (market data)                           | HTTP API (orders)                                   | Market Data WS                                   | Orders WS                                            |
|-------------|--------------------------------------------------|-----------------------------------------------------|--------------------------------------------------|------------------------------------------------------|
| Sandbox     | `https://gateway.sandbox.architect.exchange/api` | `https://gateway.sandbox.architect.exchange/orders` | `wss://gateway.sandbox.architect.exchange/md/ws` | `wss://gateway.sandbox.architect.exchange/orders/ws` |
| Production  | `https://gateway.architect.exchange/api`         | `https://gateway.architect.exchange/orders`         | `wss://gateway.architect.exchange/md/ws`         | `wss://gateway.architect.exchange/orders/ws`         |

## Usage

Run example binaries to test the adapter:

```bash
# HTTP client example
cargo run -p nautilus-architect-ax --bin ax-http-public

# WebSocket data client example
cargo run -p nautilus-architect-ax --bin ax-ws-data

# WebSocket orders client example
cargo run -p nautilus-architect-ax --bin ax-ws-orders
```

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
