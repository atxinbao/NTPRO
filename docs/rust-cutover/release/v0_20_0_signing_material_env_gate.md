# NTPRO v0.20.0 Signing Material Env-Only Gate

Date: 2026-06-29
Executor: Codex
Milestone: `v0.20.0`
Task: `V200-004`
Status: IMPLEMENTED LOCAL GATE

## Summary

V200-004 adds a typed Rust env-only signing material gate in
`crates/risk/src/v20_signing_material_gate.rs`. The gate verifies that required
production credential material is present only through declared environment
variables, blocks missing or mismatched material, and emits redacted evidence
with non-secret fingerprints only.

Plain Chinese summary: 这次实现 production submit 的签名材料 env-only gate。
真实 venue 访问材料只能来自明确的运行环境变量；缺失、空值、环境不匹配、非 env 来源
都会 blocked。evidence 只记录 env var 名、material kind 和 fingerprint，不写 key、
secret、token、signature 明文，也不打开 Dashboard credential 输入或输出。

## Runtime Entry

```text
crate = nautilus-risk
module = nautilus_risk::v20_signing_material_gate
schema_version = ntpro.v200_signing_material_env_gate.v1
contract_id = ntpro.v200_order_lifecycle_safety_contract.v1
evaluation = evaluate_signing_material_env_gate(policy, snapshot)
fingerprint = signing_material_fingerprint(value)
artifact = SigningMaterialGateEvidence::redacted_artifact()
```

## Required Fields

```text
gate_id
lifecycle_id
expected_environment
observed_environment
required env_var names
material_kind
declared material source
redacted fingerprint
raw_value_recorded = false
```

## Decisions

```text
ready = all required material is present, non-empty, and sourced from env in the expected environment
blocked = environment mismatch, missing env material, empty material, or non-env material source
```

All evidence keeps:

```text
env_only_gate_required = true
raw_key_persisted = false
raw_secret_persisted = false
raw_token_persisted = false
raw_signature_material_persisted = false
stdout_stderr_contains_secret = false
diagnostics_contains_secret = false
dashboard_credential_output_enabled = false
dashboard_credential_input_enabled = false
remote_secret_manager_used = false
```

## Stable Codes

```text
v200_signing_material_ready
v200_signing_material_environment_mismatch
v200_signing_material_missing
v200_signing_material_empty
v200_signing_material_source_not_env
```

## Coverage

The integration test `crates/risk/tests/v20_signing_material_gate.rs` covers:

```text
ready env-only material
missing env material
environment mismatch
non-env source
empty material
stable non-raw fingerprint output
redacted JSON and artifact output
```

## Non-Goals

V200-004 does not introduce a secret storage component, remote key manager,
Dashboard credential UI, adapter submit call, signed request builder, response
redaction, readback, cancel, golden traces, or release gates. Those remain
assigned to later V200 issues.
