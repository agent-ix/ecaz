# Review Request: C1 ADR-030 V2 PqFastScan Runtime Env Surface Rename

## Context

The branch has already removed the old build-time experimental gate and now
lands `PqFastScan` through the per-index `storage_format` reloption.

But the runtime tuning surface in `scan.rs` still leaked the old feasibility
names:

- `TQVECTOR_EXPERIMENTAL_ADR030_V2_*`

That was still visible in:

- runtime env lookups
- user-facing heap-rerank error messages
- the scratch startup script
- the debug runtime-settings view
- pg tests that exercise the runtime knobs

## Problem

This is no longer an ADR030-v2 experiment branch in spirit. Keeping the public
runtime knobs branded as `EXPERIMENTAL_ADR030_V2` leaks old branch history into
the mainline `PqFastScan` surface.

At the same time, dropping the old names immediately would risk breaking local
tooling and test workflows that still set them.

## Planned Slice

One compatibility-focused cleanup checkpoint:

1. introduce canonical `TQVECTOR_PQ_FASTSCAN_*` runtime env names
2. keep the legacy ADR030-v2 names working as fallbacks
3. update the visible runtime/debug/script surface to prefer the canonical
   names

No scan behavior change.

## Implementation

Updated:

- `src/am/scan.rs`
- `src/lib.rs`
- `scripts/restart_adr030_scratch.sh`

### 1. Scan runtime now prefers canonical `PqFastScan` env names

In `src/am/scan.rs`:

- added canonical runtime env constants:
  - `TQVECTOR_PQ_FASTSCAN_SCAN_WINDOW`
  - `TQVECTOR_PQ_FASTSCAN_TRAVERSAL_SCORE_MODE`
  - `TQVECTOR_PQ_FASTSCAN_RERANK_MODE`
  - `TQVECTOR_PQ_FASTSCAN_RERANK_SOURCE_COLUMN`
  - `TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL`
  - `..._SCOPE`
  - `..._LIMIT`
  - `..._STRATEGY`
- kept the legacy `TQVECTOR_EXPERIMENTAL_ADR030_V2_*` names as fallback aliases
- added a centralized helper so lookup semantics are:
  - canonical name first
  - legacy name second

That keeps old local workflows working while moving the intended surface to the
post-ADR naming.

### 2. User-facing rerank errors now reference the canonical source-column env

Also in `src/am/scan.rs`:

- heap-f32 rerank source-column resolution now reports
  `TQVECTOR_PQ_FASTSCAN_RERANK_SOURCE_COLUMN`
  instead of the old experimental ADR030-v2 name

So the user-visible guidance now points at the intended runtime knob.

### 3. Scratch startup and debug settings now expose the canonical names

Updated:

- `scripts/restart_adr030_scratch.sh`
- `tqhnsw_debug_adr030_runtime_settings()` in `src/lib.rs`

The scratch script now exports the canonical `TQVECTOR_PQ_FASTSCAN_*` names,
and the debug settings helper now reports effective values by checking the
canonical name first and the legacy name second.

### 4. Pg coverage now exercises the canonical env surface

In `src/lib.rs`, several existing tests/helpers were moved to the new names,
including the ones that exercise:

- invalid live-window configuration
- exact traversal enable/scope
- heap-f32 rerank mode and source override
- invalid traversal-score mode
- invalid rerank mode
- invalid exact traversal scope / strategy / limit
- live-window simulation overrides

That gives the canonical `TQVECTOR_PQ_FASTSCAN_*` surface direct regression
coverage while legacy aliases remain accepted by runtime lookup.

## Measurements

No benchmark or recall rerun in this slice.

## Validation

Passed:

- `cargo check --tests`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
PostgreSQL linker layer:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed unresolved PostgreSQL symbols remain in the same family, including:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

This does not change scan semantics. It cleans the visible runtime surface:

1. `PqFastScan` runtime knobs no longer need to present themselves as ADR030-v2
   experiment flags
2. old env names still work as compatibility aliases
3. scripts, debug output, and a meaningful part of the pg-test surface now use
   the canonical names

What this slice intentionally does **not** do:

- remove all runtime env tuning knobs from `scan.rs`
- change the default behavior of traversal, rerank, or live-window settings
- delete legacy alias support yet

## Next Slice

The remaining cleanup work is now mostly about deciding how much runtime tuning
surface should remain env-driven at merge time, versus staying as diagnostic
knobs after landing.
