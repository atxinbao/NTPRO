# NTPRO v0.12.0 Production Read-Only Response Shape

Date: 2026-06-20
Executor: Codex
Milestone: `v0.12.0`
Status: IMPLEMENTED CONTRACT

## Summary

V120-003 defines the redacted response-shape evidence for the production
read-only account snapshot path.

Plain Chinese summary: 账户快照返回的是生产账户信息，不能直接保存原文。NTPRO
只记录“形状证据”：字段有没有、类型对不对、数组有几项、是否通过验证。它不会记录
资产名、余额值、权限原文、账户 UID、header、signature、signed query 或 signed URL。

## Account Snapshot Required Shape

The v0.12 account snapshot response shape validator checks:

| Field | Required shape | Stored evidence |
| --- | --- | --- |
| `accountType` | present and string | boolean only |
| `balances` | present and array | boolean plus entry count |
| `balances[].asset` | string | boolean only; value not stored |
| `balances[].free` | string | boolean only; value not stored |
| `balances[].locked` | string | boolean only; value not stored |
| `permissions` | present and array | boolean plus entry count |
| `permissions[]` | string | boolean only; value not stored |
| `canTrade` | present and boolean | boolean only |
| `canWithdraw` | present and boolean | boolean only |
| `canDeposit` | present and boolean | boolean only |

Unknown or incompatible shape fails closed as `response_shape_invalid`.

## Redacted Evidence

The account snapshot report may include:

```json
{
  "response_shape": "binance_account_snapshot_v1",
  "response_shape_validated": true,
  "response_shape_summary": {
    "status": "accepted",
    "account_type_present": true,
    "account_type_is_string": true,
    "balances_present": true,
    "balances_is_array": true,
    "balance_entry_count": 2,
    "balance_entry_shape_validated": true,
    "permissions_present": true,
    "permissions_is_array": true,
    "permission_entry_count": 1,
    "permission_entry_shape_validated": true,
    "can_trade_present": true,
    "can_trade_is_bool": true,
    "can_withdraw_present": true,
    "can_withdraw_is_bool": true,
    "can_deposit_present": true,
    "can_deposit_is_bool": true,
    "raw_account_response_recorded": false,
    "raw_balances_recorded": false,
    "raw_permissions_recorded": false,
    "shape_validated": true,
    "rejection_reason": "none"
  }
}
```

## Forbidden Evidence

The report must not include:

- raw account response body;
- raw balance values;
- asset symbols;
- permission values;
- UID or account identifiers;
- API key or API secret values;
- signature;
- signed query;
- signed URL;
- raw headers.

## Failure Behavior

If the response shape is missing or incompatible:

```text
status=online_account_snapshot_failed
error_code=response_shape_invalid
response_shape_validated=false
shape_validated=false
```

The failure artifact still records only the redacted shape summary.
