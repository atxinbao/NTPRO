# Release Body Hash Semantics

Date: 2026-07-09
Executor: Codex
Task: `V281-004` / GitHub issue `#922`
Status: ACTIVE CONTRACT

## Rule

Release body verification uses normalized SHA-256 as the acceptance rule.
Raw SHA-256 may be recorded for diagnostics, but raw equality is not the gate
condition.

Plain Chinese summary: release body 校验采用 normalized SHA-256，而不是 raw
SHA-256。raw hash 可以记录用于排查，但不能被写成发布通过条件。仅结尾换行或每行尾部空白
造成的差异会被 normalization 忽略；正文内容、段落顺序、必填字段或非尾部内容变化不会被接受。

## Normalization

```text
release_body_hash_semantics = normalized_sha256
release_body_normalization = line_rstrip_and_outer_strip
accepted_trailing_newline_only_drift = true
accepted_per_line_trailing_whitespace_only_drift = true
accepted_content_drift_beyond_normalization = false
raw_sha256_is_acceptance_rule = false
```

The algorithm is:

```python
"\n".join(line.rstrip() for line in text.splitlines()).strip()
```

## Audit Text Requirements

Future release audit text must name the hash type:

```text
release body normalized sha256 = <sha256>
tracked release notes normalized sha256 = <sha256>
normalized release body matches tracked release notes = true
release body raw sha256 = <sha256>
tracked release notes raw sha256 = <sha256>
raw release body matches tracked release notes = <true|false>
raw hash equality is diagnostic, not the acceptance rule
```

Audit text must not use the ambiguous phrase `release body sha256` as the only
proof marker for release body equality.

## Guard Output

`scripts/ai/check_github_release_published.sh` reports:

```text
release_body_hash_semantics=normalized_sha256
release_body_normalization=line_rstrip_and_outer_strip
release_body_normalized_sha256=<sha256>
tracked_release_notes_normalized_sha256=<sha256>
release_body_normalized_sha256_matches_tracked_release_notes=true
release_body_raw_sha256=<sha256>
tracked_release_notes_raw_sha256=<sha256>
release_body_raw_sha256_matches_tracked_release_notes=<true|false>
release_body_raw_sha256_is_acceptance_rule=false
```

When `NTPRO_RELEASE_PUBLICATION_STRICT_BODY=1`, the guard fails unless
`release_body_normalized_sha256_matches_tracked_release_notes=true`.
