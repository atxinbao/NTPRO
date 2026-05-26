# Target Architecture

```text
crates/
  core/                  foundational types, time, UUID, correctness
  model/                 domain model, instruments, orders, events, value types
  common/                components, clock, cache, message bus, actor machinery
  data/                  data engine and data routing
  execution/             order lifecycle and execution routing
  risk/                  pre-trade checks and risk limits
  portfolio/             positions, accounts, PnL
  backtest/              deterministic backtest runtime
  live/                  live node and client lifecycle
  trading/               Rust actors and strategies
  adapters/*             venue/data-provider adapters
  cli/                   Rust-only operational command surface
```

Rust examples and Rust documentation are part of the product surface. There is no retained Python, PyO3, or Cython product runtime in the target architecture.

## Desired Dependency Direction

```text
cli/examples -> live/backtest -> execution/risk/data/portfolio -> common -> model -> core
adapters -> live/backtest -> execution/risk/data/portfolio -> common -> model -> core
```

## Anti-Goals

- Do not create a second trading engine.
- Do not maintain duplicate semantics across Python/PyO3/Cython and Rust.
- Do not keep v1 Cython as a legacy runtime.
- Do not keep PyO3 as a final product API.
- Do not use Python as the runtime source of truth.
