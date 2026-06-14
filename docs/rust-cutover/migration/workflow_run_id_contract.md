# Workflow run_id contract

Date: 2026-06-14
Executor: Codex

## Summary

NTPRO workflow artifacts use one effective `run_id` as the public identity for a
workflow run.

For the Binance testnet workflow:

```text
effective_run_id = CLI --run-id if provided, otherwise config.run.id
```

The effective run id is written to:

- CLI output;
- `manifest.json.run_id`;
- `manifest.json.summary.run_id`;
- `summary.json.run_id`;
- `testnet/config.json.run_id`;
- every `events.jsonl` event `run_id`;
- Dashboard workflow artifact rows.

## Audit Field

`testnet/config.json` also includes:

```text
config_declared_run_id
```

This field preserves the original `config.run.id` value for audit when the CLI
overrides the run id. It is not the primary workflow identity.

## User Impact

Scripts and dashboards should key a workflow run by the primary `run_id` field.
They should not infer workflow identity from the output directory name or from
the original config `run.id` when CLI `--run-id` was supplied.

This migration does not connect to Binance testnet, submit orders, read secret
values, or change trading semantics.
