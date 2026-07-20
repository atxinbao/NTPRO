# BPO-004 Checked Interpolation Migration

Date: 2026-07-20
Executor: Codex
Task: BPO-004 / GitHub issue #1123

## Scope

BPO-004 adds checked interpolation and yield-curve APIs. Existing panic-style
functions remain source compatible, but production runtime boundaries should use
the checked variants.

| Compatibility API | Checked API |
| --- | --- |
| `linear_weight` | `try_linear_weight` |
| `quad_polynomial` | `try_quad_polynomial` |
| `quadratic_interpolation` | `try_quadratic_interpolation` |
| `YieldCurveData::get_rate` | `YieldCurveData::try_get_rate` |
| `Cache::yield_curve` | `Cache::try_yield_curve_rate` |

## Required Caller Change

New runtime callers must propagate or explicitly map `InterpolationError`.
They must not convert an error into retry, recovery, fallback trading behavior,
or a silent default. A missing cache key remains `Ok(None)`; malformed cached
data or an invalid expiry is `Err`.

`Cache::add_yield_curve` now validates curve shape, ordering, finite values, and
numeric stability before insertion. Invalid curve data is rejected instead of
being cached for a later panic.

中文说明：公开旧接口没有删除，但生产调用链应迁移到 `try_*` 接口并显式处理错误。
cache 中没有对应曲线仍表示 `Ok(None)`；曲线存在但内容无效，或查询期限无效，则返回
`Err`。不得把该错误转换为自动重试、恢复、交易发送或静默默认值。

## Error Contract

`InterpolationError` distinguishes insufficient points, incompatible lengths,
non-finite input, unsorted or duplicate abscissas, numerically close points, and
non-finite computed output. Callers may add boundary context but should retain
the original error as the source.

## Rollback

Revert BPO-004. No persisted schema, CLI configuration, adapter contract, or
deployment migration is required.
