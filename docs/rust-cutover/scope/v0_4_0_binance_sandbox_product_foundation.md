# NTPRO v0.4.0 Scope - Binance Sandbox Product Foundation

Date: 2026-06-13
Executor: Codex
Status: proposed product boundary

## Decision

`v0.4.0` is scoped as the first Binance sandbox product foundation milestone.
It is not a production trading release and must not be described as real-funds
venue connectivity.

Release claim for `v0.4.0`:

```text
Binance Sandbox Product Foundation
```

Plain language:

```text
v0.4.0 proves a local, deterministic Binance sandbox path for product demos and
release evidence. It does not connect to real accounts, submit real orders, or
claim production Binance trading readiness.
```

## Product Boundary

In scope:

- Binance Spot sandbox product boundary documentation.
- Binance USDT-M classification as deferred unless a later task records
  explicit sandbox evidence.
- Checked-in fixture replay for deterministic market-data evidence.
- Mock order lifecycle evidence for submit, accept, fill, cancel, and scoped
  reject behavior.
- Built-in EMA and RSI sandbox strategy contracts and smokes.
- Deterministic risk rejection evidence.
- Local Dashboard panels that display exchange, strategy, order, and risk state
  from the sandbox evidence path.

Out of scope:

- real funds;
- production trading;
- production exchange connectivity;
- real account connectivity;
- real order submission;
- production venue behavior;
- production Binance parity claims;
- arbitrary user strategy loading;
- manual order entry;
- remote or multi-user Dashboard operation;
- Docker or prebuilt binary delivery as a `v0.4.0` release requirement.

## Validation Boundary

The default validation path for `v0.4.0` is:

```text
checked-in fixture -> local replay -> mock order lifecycle -> scoped risk
evidence -> strategy smoke -> local dashboard state
```

Allowed evidence types:

- checked-in Binance-like fixtures;
- local deterministic replay;
- mock execution;
- sandbox-only risk rejection;
- local CLI and Dashboard smokes;
- GitHub release-gate output after local evidence is complete.

Not allowed as release evidence:

- real API keys;
- real account balances;
- live production Binance connectivity;
- production order acknowledgements;
- network-only behavior that cannot be replayed locally.

## Binance Spot Boundary

`v0.4.0` may claim Binance Spot only as a sandbox product foundation when the
queue proves:

- fixture-backed market-data replay;
- built-in EMA and RSI strategy smokes;
- mock order lifecycle evidence;
- deterministic risk rejection evidence;
- Dashboard visibility for the sandbox state.

`v0.4.0` must not claim production Binance Spot trading support.

## Binance USDT-M Boundary

Binance USDT-M is deferred for production or real venue behavior in `v0.4.0`.
It may be listed in the capability matrix only as `deferred`, `fixture-only`, or
`sandbox-only` if later evidence exists.

No task may turn Binance USDT-M into a production trading claim without a new
scope decision and release gatekeeper approval.

## Release Surface Wording

README and release documents should describe `v0.4.0` with this wording:

```text
v0.4.0 is the Binance Sandbox Product Foundation. It proves local fixture
replay, built-in EMA/RSI strategy smokes, mock order lifecycle, deterministic
risk rejection, and local Dashboard state for sandbox evidence only. It is not
a production trading release and does not support real funds or real orders.
```

Short label:

```text
Binance sandbox-only; fixture/testnet/mock first; no real funds.
```

## Task Sequence

```text
V04-001 Binance product boundary contract
  -> V04-002 Binance capability matrix
  -> V04-003 EMA / RSI strategy contracts
  -> V04-004 strategy config DTO
  -> V04-005 Binance fixture market data replay
  -> V04-008 mock order lifecycle
  -> V04-009 risk rejection smoke
  -> V04-006 EMA smoke
  -> V04-007 RSI smoke
  -> V04-010 Dashboard exchange / strategy / order / risk panels
  -> V04-011 ignored tests closure batch 2
  -> V04-012 v0.4 readiness report
```

## Release Decision

Do not publish a `v0.4.0` release until `V04-001` through `V04-012` have
evidence and the final readiness report records strict PASS.
