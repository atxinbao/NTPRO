# RADP-001 Evidence - Inventory Rust Adapter Gaps For Binance

Date: 2026-05-31
Executor: Codex
Task ID: RADP-001
Risk: medium

## Summary

Inventoried the Binance Rust adapter parser, data, execution, fixture, and
product-scope surfaces for the Rust adapter parity gate.

The inventory records that Spot, USD-M Futures, and COIN-M Futures have Rust
factory paths, parser/data clients, execution clients, fixtures, and Rust tests.
The remaining gaps are product-scope and parity boundaries: Margin and Options
appear in the product model and crate README but are not runtime factory
targets, multi-product configs select only the first product type, custom data is
futures-only and narrow, some book/custom-data limits are intentionally scoped,
Rust docs still point users through Python-heavy configuration guidance, and
optional Python/PyO3 Binance bridge surfaces remain for later removal gates.

## Files Changed

- `docs/rust-cutover/inventory/binance_adapter_gaps.md`
- `docs/rust-cutover/inventory/README.md`
- `docs/rust-cutover/evidence/RADP-001.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-001.json`

## Commands Run

Task setup and scope:

```bash
git status --short --branch
sed -n '1,220p' docs/rust-cutover/tasks/RADP-001.md
python3 -m json.tool .agentflow/state/task_status.json
python3 -m json.tool .agentflow/leases/RADP-001.json
```

Shrimp and planning:

```text
shrimp_task_manager.get_task_detail(87681b7d-c924-418e-9b53-d564e43a77e4)
shrimp_task_manager.process_thought(...)
shrimp_task_manager.analyze_task(...)
shrimp_task_manager.reflect_task(...)
```

Context and inventory:

```bash
find crates/adapters/binance -maxdepth 3 -type f | sort
sed -n '1,220p' crates/adapters/binance/README.md
sed -n '1,240p' crates/adapters/binance/Cargo.toml
sed -n '1,260p' crates/adapters/binance/src/common/enums.rs
sed -n '1,260p' crates/adapters/binance/src/factories.rs
sed -n '35,155p' crates/adapters/binance/src/config.rs
rg -n "TODO|FIXME|not implemented|Unsupported product|unsupported product|bail!\\(|todo!\\(|unimplemented!\\(" crates/adapters/binance/src --glob '!src/python/**' --glob '!src/spot/sbe/generated/**'
rg -n "Python|PyO3|nautilus_pyo3|Rust|Spot margin|Options|Configuration|Rust supports|Python adapter" docs/integrations/binance.md crates/adapters/binance/README.md
sed -n '1,80p' docs/integrations/binance.md
sed -n '680,750p' docs/integrations/binance.md
python3 - <<'PY'
from pathlib import Path
for p in sorted(Path('crates/adapters/binance/tests').rglob('*.rs')):
    text=p.read_text()
    count=sum(text.count(marker) for marker in ('#[rstest]', '#[tokio::test]', '#[test]'))
    print(f'{p}: {count}')
PY
python3 - <<'PY'
from pathlib import Path
for root in ['spot/http_json','spot/user_data_json','spot/user_data_sbe/mainnet','futures/http_json','futures/market_data_json','futures/user_data_json']:
    p=Path('crates/adapters/binance/test_data')/root
    print(f'{root}: {len([x for x in p.rglob("*") if x.is_file()])}')
PY
rg -n "api_key|api_secret|secret|private_key|BEGIN .*PRIVATE KEY|sk_live|AKIA|password" crates/adapters/binance docs/integrations/binance.md --glob '!target/**'
```

Required validation:

```bash
scripts/ai/verify_fast.sh
```

Final local checks:

```bash
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-001.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- `shrimp_task_manager.get_task_detail`: confirmed RADP-001 is the active
  `in_progress` task in the isolated NTPRO Shrimp queue.
- Code-index MCP was requested through tool discovery but was not exposed as a
  callable code-index tool in this session; repository inspection used local
  `rg`, `find`, and direct file reads instead.
- The inventory scan found:
  - `nautilus-binance` default build uses `high-precision`; `python` and
    `extension-module` are optional features.
  - `BinanceProductType` includes `Spot`, `Margin`, `UsdM`, `CoinM`, and
    `Options`.
  - Runtime factories create clients for `Spot`, `UsdM`, and `CoinM`; other
    product types are unsupported at factory creation.
  - Spot data/execution paths exist for SBE stream parsing, HTTP requests,
    WebSocket trading, and user-data/account/order/fill handling.
  - Futures data/execution paths exist for USD-M and COIN-M market data,
    execution, custom data, user data, and stream recovery.
  - The Binance Rust test tree contains 388 annotated Rust tests across spot
    and futures HTTP, data client, execution client, stream, and trading client
    files.
  - Fixture sets exist for spot HTTP JSON, spot user-data JSON, spot user-data
    SBE, futures HTTP JSON, futures market-data JSON, and futures user-data
    JSON.
  - Docs still contain mixed Python/Rust guidance, including Python examples and
    Python-oriented configuration tables.
  - No hardcoded real Binance secret was identified. The matches were
    placeholders, environment variable names, test values, fixture-capture code,
    or redacted credential handling.
- `scripts/ai/verify_fast.sh`: passed with `== verify_fast complete ==`. The
  script ran toolchain lookup and `cargo fmt --check`; cargo check and clippy
  remained skipped by the script defaults.
- JSON validation for `.agentflow/state/task_status.json` and
  `.agentflow/leases/RADP-001.json`: passed.
- `validate_agentflow_roles.py`: passed.
- `git diff --check`: passed.

## Tests Added Or Updated

No Rust tests were added. RADP-001 is inventory-only.

RADP-002 is the follow-up task for Binance Rust adapter fixtures. RADP-003 is
the follow-up task for closing or explicitly scoping the Binance adapter gaps.

## Behavior Impact

No runtime behavior changed. No parser behavior, market data behavior, execution
behavior, exchange protocol behavior, credential handling, public API, Python
API, PyO3 binding, Cython surface, or Cargo feature behavior changed.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR only adds inventory and evidence.

## Gate Status

RADP-001 is medium risk. It documents adapter parity scope, but it does not
change adapter runtime behavior or trading semantics.

Auto-merge is allowed after local validation and required PR checks pass.

## Rollback Plan

Revert the Binance inventory file, the inventory README entry, this evidence
file, and the RADP-001 task state/lease updates. No runtime, persisted data,
adapter, schema, Python, PyO3, Cython, or public API rollback is required.
