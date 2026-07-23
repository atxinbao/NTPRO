# nautilus-polymarket

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg?branch=main)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-polymarket)](https://docs.rs/nautilus-polymarket/latest/nautilus-polymarket/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-polymarket.svg)](https://crates.io/crates/nautilus-polymarket)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

NTPRO adapter for the [Polymarket](https://polymarket.com) prediction market.

The `nautilus-polymarket` crate provides client implementations (HTTP & WebSocket), data
models and parsing for the **Polymarket CLOB API** for trading binary option contracts.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

[High-precision mode](https://github.com/atxinbao/NTPRO/blob/main/docs/getting_started/installation.md#precision-mode) (128-bit value types) is enabled by default.

## API endpoints

The adapter communicates with four Polymarket API surfaces:

| API            | Base URL                                        | Auth                   | Purpose                                     |
|----------------|-------------------------------------------------|------------------------|---------------------------------------------|
| CLOB REST      | `https://clob.polymarket.com`                   | L2 HMAC                | Orders, trades, balances.                   |
| CLOB WebSocket | `wss://ws-subscriptions-clob.polymarket.com/ws` | L2 HMAC (user channel) | Streaming orderbook, trades, order updates. |
| Gamma          | `https://gamma-api.polymarket.com`              | None                   | Market and event discovery, tags, search.   |
| Data           | `https://data-api.polymarket.com`               | None                   | Trade history and user positions.           |

## Authentication

Polymarket uses two-tier authentication:

- **L1 (EIP-712)**: Wallet-level signing for API credential creation and order signing
  via the CTF Exchange contract. Uses `alloy` signer crates.
- **L2 (HMAC-SHA256)**: API key + secret + passphrase for authenticated REST and
  WebSocket requests. Signatures expire after 30 seconds.

## Documentation

See [the docs](https://docs.rs/nautilus-polymarket) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
