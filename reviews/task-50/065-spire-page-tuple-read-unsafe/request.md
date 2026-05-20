# Task 50 Review Request: SPIRE Page Tuple Read Unsafe

## Summary

This packet continues the approved comprehensive unsafe burndown from
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.

The code change consolidates SPIRE page-level tuple read regions:

- root/control special-area size, pointer, and slice reads;
- object tuple scan max-offset and per-item visitor calls;
- delete-path item-id and line-pointer reads;
- locked-page tuple lookup, item bounds, tuple pointer, and borrowed slice
  construction.

The page lock/pin and WAL boundaries are unchanged. The cleanup groups the
read-only PostgreSQL page-memory operations behind the local checks that prove
offset and tuple bounds before borrowed tuple slices are exposed to visitors.

## Code

- commit: `eb2d843f Consolidate SPIRE page tuple reads`
- touched file: `src/am/ec_spire/page.rs`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/page.rs` | 35 | 27 | -8 |
| `src/` ledger rows | 2099 | 2091 | -8 |

This does not close Task 50. The packet-local ledger still contains `2091`
direct unsafe rows under `src/`.

## Validation

- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known pre-existing `src/am/mod.rs` unused import warning.
- `artifacts/unsafe-ledger-check.log`: ledger covers `2091` current unsafe rows.
- `artifacts/src-unsafe-block-count-after.log`: after-count evidence for touched and remaining unsafe files.

## Behavioral Risk

Expected behavior is unchanged. This slice preserves root/control decoding,
object tuple scanning, tuple rewrite, and no-compact deletion behavior while
keeping tuple-slice lifetimes tied to the locked page visitor call.

No benchmark was run because this packet does not intentionally alter scoring,
ordering, payload layout, WAL behavior, storage format, corpus loading, or
rerank behavior.

