# nautilus-kraken

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg?branch=main)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-kraken)](https://docs.rs/nautilus-kraken/latest/nautilus-kraken/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-kraken.svg)](https://crates.io/crates/nautilus-kraken)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

NTPRO adapter for the [Kraken](https://www.kraken.com/) exchange.

The `nautilus-kraken` crate provides client bindings (HTTP & WebSocket), data models,
and helper utilities that wrap the official **Kraken API v2**.

The official Kraken API reference can be found at <https://docs.kraken.com/api/>.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Features

- HTTP REST API clients for market data (Spot v2, Futures v3).
- WebSocket clients for real-time data feeds (Spot v2, Futures).
- Support for both Spot and Futures markets.
- Instrument, ticker, trade, orderbook, and OHLC data.
- Prepared for execution support (orders, positions, balances) - WIP.

## Architecture

This crate provides **separate HTTP and WebSocket clients for Spot and Futures markets**.
This design reflects fundamental differences between the two APIs:

| Aspect         | Spot                  | Futures                      |
|----------------|-----------------------|------------------------------|
| API Version    | REST API v2           | Derivatives API v3           |
| Base URL       | `api.kraken.com`      | `futures.kraken.com`         |
| Auth Headers   | `API-Key`, `API-Sign` | `APIKey`, `Authent`, `Nonce` |
| Request Format | URL-encoded form      | JSON body                    |
| WebSocket      | v2 protocol           | Futures-specific protocol    |

Kraken Futures was originally a separate platform (Crypto Facilities) acquired by Kraken,
which explains why the APIs remain distinct rather than unified.

### Client Types

- **`KrakenSpotHttpClient`** / **`KrakenSpotWebSocketClient`**: For spot trading pairs (e.g., `BTC/USD`, `ETH/EUR`).
- **`KrakenFuturesHttpClient`** / **`KrakenFuturesWebSocketClient`**: For perpetual and fixed-maturity futures (e.g., `PF_XBTUSD`, `PI_ETHUSD`).

### Bitcoin symbol format

| Market  | Format | Example      | Notes                                            |
|---------|--------|--------------|--------------------------------------------------|
| Spot    | `BTC`  | `BTC/USD`    | XBT normalized to BTC (base or quote position).  |
| Futures | `XBT`  | `PI_XBTUSD`  | Uses Kraken's native XBT format.                 |

## Examples

See the `bin/` directory for example usage:

```bash
cargo run --bin kraken-http-spot-raw
cargo run --bin kraken-http-spot-public
cargo run --bin kraken-ws-spot-data
```

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

[High-precision mode](https://github.com/atxinbao/NTPRO/blob/main/docs/getting_started/installation.md#precision-mode) (128-bit value types) is enabled by default.

## Documentation

See [the docs](https://docs.rs/nautilus-kraken) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
