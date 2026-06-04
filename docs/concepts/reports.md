# Reports

:::warning
This page replaces the upstream Python/pandas report tutorial with the current NTPRO
Rust-only status. Historical Python report APIs such as `ReportProvider`, pandas
DataFrames, and visualization extras are not supported NTPRO product paths.
:::

## NTPRO Rust-only status

NTPRO keeps reporting and performance-analysis terminology as part of the trading
domain, but the current public product surface is Rust-only. Supported release evidence
must come from Rust crates, Cargo validation, Rust examples, and the release notes under
`docs/rust-cutover/`.

Do not treat legacy upstream Python helpers as a current NTPRO install path, API entry,
runtime dependency, or release capability.

## Report concepts retained

The following report categories remain useful domain concepts for future Rust work:

- Orders report: order lifecycle, client order IDs, venue order IDs, status, quantity,
  price, time-in-force, and timestamps.
- Fills report: executed quantity, execution price, commission, liquidity side, and
  event timestamps.
- Positions report: position IDs, opening/closing orders, realized PnL, duration, and
  snapshot status for netting OMS flows.
- Account report: account balances, margin state, currency balances, and venue-reported
  account changes.
- Performance report: drawdown, returns, exposure, PnL, execution quality, and replay
  comparison metrics.

## Unsupported legacy paths

The upstream Python/pandas reporting stack is retained only as migration history outside
this NTPRO Rust-only product contract. In particular:

- Do not install visualization extras through `uv pip` or PyPI for NTPRO.
- Do not import `nautilus_trader.analysis.ReportProvider` as an NTPRO product API.
- Do not depend on pandas DataFrames as the supported NTPRO report output format.
- Do not describe Python report helpers as a live or backtest runtime capability for
  the current release.

## Follow-up work

Future report work should be tracked as a separate Rust product task. A scoped task should
define the Rust report data model, output format, CLI or API entry point, fixture strategy,
and regression evidence before documenting it as supported.
