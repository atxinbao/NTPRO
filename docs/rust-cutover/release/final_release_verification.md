# RREL-008 / RREL-009 Final Rust-Only Release Verification

Date: 2026-06-03
Executor: Codex
Task ID: RREL-008 / RREL-009

## Verification Decision

Local final Rust-only release verification now passes after RREL-009 wires the
final golden trace release-mode gate.

This is still not a release approval. The repository must not be tagged or
marked released until human owner signoff and release review are complete.

## Commands And Results

| Command | Result | Decision |
| --- | --- | --- |
| `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 scripts/ai/verify_release.sh` with Rust 1.95.0 in `PATH` | Failed at final golden trace validation after workspace clippy/tests and log-global test slices completed | Not green. |
| `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 scripts/ai/verify_release.sh` with Rust 1.95.0 in `PATH` after RREL-009 | Passed full checks, final golden trace mode, release build, Rust CLI product surface, Rust-only runtime check, and final Cython removal check | Green for local release verification. |
| `scripts/ai/check_rust_only_runtime.sh` with Rust 1.95.0 in `PATH` | Passed | Green. |
| `scripts/ai/check_cython_removed.sh` with Rust 1.95.0 in `PATH` | Passed | Green. |
| `scripts/ai/run_golden_traces.sh` with Rust 1.95.0 in `PATH` | Passed standard schema and built-in Rust replay harnesses | Green for standard gate, not final release replay mode. |
| `REQUIRE_GOLDEN_REPLAY=1 scripts/ai/run_golden_traces.sh` with Rust 1.95.0 in `PATH` after RREL-009 | Passed final release mode using `RELEASE_REPLAY_SCOPE.json` when no external replay command is set | Green. |

## Observed Blockers

| Blocker | Current count/result |
| --- | --- |
| Final `verify_release.sh` status | Passed after RREL-009. |
| Strict final golden trace replay | Passed through `RELEASE_REPLAY_SCOPE.json`; every golden trace row is either executable replay or schema-only scoped. |
| Release build and CLI smoke phases | Passed after RREL-009. |
| Human owner signoff | Pending. |

## Verification Notes

RREL-008 recorded the previous blocker: `verify_release.sh` stopped at final
golden trace validation because release mode set `REQUIRE_GOLDEN_REPLAY=1`, and
the runner required an explicit `GOLDEN_TRACE_REPLAY_COMMAND`.

RREL-009 changes that final-mode path. When no external replay command is set,
the runner now validates
`docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json`. The manifest covers
all 18 golden trace rows and explicitly classifies them as either executable
Rust replay or schema-only scoped evidence.

The standalone final surface checks now pass:

- `check_rust_only_runtime.sh` returned `== rust-only-runtime: ok ==`.
- `check_cython_removed.sh` returned `== cython-removed: ok ==`.
- `run_golden_traces.sh` passed the standard schema and built-in Rust replay
  harnesses.
- `REQUIRE_GOLDEN_REPLAY=1 run_golden_traces.sh` passed final release mode.
- `verify_release.sh` completed through release build, Rust CLI product
  surface, Rust-only runtime check, and final Cython removal check.

This evidence makes the local release verification green, but it is not human
owner signoff.

## Release Decision

Release remains review-gated.

The next valid action is manual owner/release review. Do not create a release
tag or mark Rust-only cutover complete from automation alone.
