# PAR-007 Production Mutation Executor Retirement

Date: 2026-07-29
Executor: Codex
Task: PAR-007 / GitHub issue #1175

## Change

The following legacy commands remain available only for offline historical
artifact evaluation:

```text
nautilus live production-mutation-guarded-send
nautilus live production-mutation-actual-cancel-single-shot
```

They no longer accept:

```text
--manual-online
--api-key-env
--api-secret-env
```

The current CLI no longer contains their production POST/DELETE executors,
signed request construction, credential environment readers, or approval
consumption path. Timestamp and recv-window inputs remain only as historical
artifact-shape metadata and are never signed.

中文说明：这不是把真实下单或撤单隐藏得更深，而是从当前产品二进制中删除
对应网络执行能力。命令仍可读取旧版本工件并输出脱敏、离线、零网络计数的审计
结果；任何历史 owner approval、release manifest 或成功 attempt 工件都不能转化
为当前执行权限。

## Operator Action

Remove the three retired options from retained scripts. Treat output fields
such as `manual_online_requested=false`, `single_shot_*_allowed=false`,
`production_signing_material_env_read=false`, and
`not_attempted_executor_retired` as the current authoritative boundary.

Historical actual-attempt artifacts remain valid audit inputs for downstream
readback and failure-evidence readers. They are historical facts, not commands
and not authorization.

## Future Capability

Restoring production mutation requires a separately scoped release with new
current-version authorization, risk, rollback, telemetry, venue, credential,
negative-test, and publication evidence. It must not reuse the retired code or
inherit authority from v0.16-v0.19, v0.32.0, or v0.33.0.

## Rollback

Reverting PAR-007 would reintroduce compiled production mutation executors and
is not an acceptable operational rollback. Any restoration requires a new
capability decision and independent implementation review.
