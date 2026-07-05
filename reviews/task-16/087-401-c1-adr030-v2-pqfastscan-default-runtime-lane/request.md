# Review Request: C1 ADR-030 V2 PqFastScan Default Runtime Lane

## Context

Packets `378` through `400` moved `PqFastScan` from an ADR030 feasibility
branch surface toward a first-class format:

- per-index `storage_format = 'pq_fastscan'`
- scan / insert / vacuum parity for built indexes
- canonical `PqFastScan` naming across the runtime and docs
- real-corpus harness support for explicit storage formats

But one important thing was still wrong for a mainline landing:

- the runtime still defaulted to the old exploratory lane
  - live rerank window defaulted to `4`
  - traversal score mode defaulted to grouped-PQ
- the stronger operating point from packets `359` / `360` / `361` / `362`
  still required process env tuning:
  - `TQVECTOR_PQ_FASTSCAN_SCAN_WINDOW=64`
  - `TQVECTOR_PQ_FASTSCAN_TRAVERSAL_SCORE_MODE=binary`

That is not acceptable for a "first-class on main" format. A format whose
default runtime lane is substantially weaker than its already-proven operating
point is still effectively experimental.

## Problem

`PqFastScan` had two contradictory truths at once:

1. the branch already had credible real-corpus evidence for the tuned lane
   - packet `361`: canonical `50k`, `m=8`, `queries_50`, `ef=128` reached
     `0.910 Recall@10` at `window=64`, binary traversal
   - packet `362`: canonical `50k`, `m=16`, `queries_50`, `ef=128` reached
     `0.936 Recall@10` on the same tuned lane
2. the code still defaulted to the weaker untuned lane whenever those envs
   were absent

That meant a clean install could build a valid `PqFastScan` index and then scan
it through a runtime shape we already knew was not the intended operating
point.

## Planned Slice

Promote the proven runtime lane into the code defaults without deleting the
tuning surface:

1. raise the default live rerank window to `64`
2. make binary traversal the default `PqFastScan` traversal score mode
3. keep an automatic fallback to grouped-PQ when a `PqFastScan` layout lacks
   the persisted binary sidecar
4. make the debug runtime-settings helpers report effective defaults instead of
   only surfacing explicitly-set env vars
5. align the scratch restart helper defaults with the code defaults

This slice intentionally does not:

- remove the runtime env knobs
- move the runtime knobs to reloptions or GUCs
- change rerank mode defaults (`quantized` stays the default)
- claim a fresh benchmark rerun on the newly-built binary in this packet

## Implementation

Updated:

- `src/am/scan.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `scripts/restart_adr030_scratch.sh`

Concrete changes:

1. promoted `PQ_FASTSCAN_DEFAULT_LIVE_RERANK_WINDOW` from `4` to `64`
2. changed the default traversal score mode from grouped-PQ to binary
3. made the env-less traversal-score fallback layout-aware:
   - `PqFastScan` with a persisted binary sidecar defaults to binary traversal
   - `PqFastScan` without the sidecar falls back to grouped-PQ
4. re-exported the runtime-default constants for the pg test/debug helper
   surface
5. changed `tqhnsw_debug_pq_fastscan_runtime_settings()` /
   `tqhnsw_debug_adr030_runtime_settings()` to report effective defaults for:
   - scan window
   - traversal score mode
   - rerank mode
6. added pg coverage for those effective defaults
7. updated `scripts/restart_adr030_scratch.sh` so its no-flag startup path
   matches the new code defaults:
   - `window=64`
   - `grouped_score_mode=binary`

## Validation

Passed:

- `cargo check --tests`
- `cargo check --tests --no-default-features --features 'pg17 pg_test'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `bash -n scripts/restart_adr030_scratch.sh`

Required full-test commands were run and still hit the same workstation linker
boundary as the rest of this branch:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed unresolved PostgreSQL symbols remain in the same family:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

Operational note:

- `cargo fmt --all` is currently blocked by an unrelated parse failure in
  `src/quant/prod.rs` on `rng.gen()` under this toolchain. I did not touch that
  unrelated file in this slice.

## Outcome

This packet makes the mainline `PqFastScan` story more honest:

1. the default runtime lane now matches the already-proven binary/window-64
   direction instead of the old weak exploratory defaults
2. the debug runtime-settings helpers now surface what an operator will
   actually get by default
3. the scratch restart helper no longer defaults to a weaker lane than the
   code it is meant to exercise

What this packet still does **not** solve:

- the runtime tuning surface is still env-driven
- the scratch wrappers can still silently target the wrong cluster if socket
  fallback is ambiguous

## Next Slice

Harden the scratch wrappers so "scratch" commands do not silently fall back
from `/tmp/tqvector_pgrx_home` to `~/.pgrx` unless the caller makes that
choice explicitly. That is the operational guardrail this branch needs after
the 2026-04-16 real-corpus mismeasurement.
