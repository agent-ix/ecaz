# Task 53 / 003 — source.rs Consumer Migration · Artifact Manifest

Packet path: `reviews/task-53/003-source-rs-consumer-migration/`
Branch: `task-53`

## Surfaces

- `src/am/ec_hnsw/source.rs` only.

## Per-file before/after `unsafe { ... }` blocks

| File | Pre | Post | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/source.rs` | 29 | **13** | **-16** |
| `src/am/common/datum.rs` | 15 | 15 | 0 |
| `src/am/common/detoast.rs` | 5 | 5 | 0 |

Diff stat: `+51 / -234` (183 net lines deleted from source.rs as the
HNSW-local wrappers retire in favor of `common/datum.rs`).

**Task 53 §Exit Criterion #2 (`source.rs` ≤ 14) satisfied with 1
block of margin.**

## Task 53 cumulative arc (slices 001-003)

| Surface | Pre-Task-53 | Now | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/source.rs` | **29** | **13** | **-16 (-55.2%)** |
| `src/am/common/datum.rs` (new) | — | 15 | +15 |
| `src/am/common/detoast.rs` | 4 | 5 | +1 |
| **`src/` total** | 960 | 960 | **0 net** |

Wrapper-side +16; consumer-side -16. Net zero on the `src/` total —
the wrappers fully absorb the surface, no inflation.

## Artifacts

This slice's evidence is the diff and the count grep. No standalone
JSON / log artifact needed.

- Head SHA: parent of packet commit.
- Lane / fixture / storage / rerank: N/A (compile-only).
- Isolation: N/A.
- Command (validation):
  - `cargo fmt --all` — clean (touched only `source.rs`; unrelated
    workspace fmt drift left untouched per CLAUDE.md).
  - `cargo check --no-default-features --features pg18` — `Finished`
    exit 0, 17.16s (subagent's report); operator re-ran on cached
    state: `Finished` 0.10s.
  - `cargo clippy ... -- -D warnings` — not re-run this slice; same
    pre-existing rabitq backlog.
  - `cargo pgrx test` — skipped per `feedback_dyld_buffer_blocks_known`.
- Timestamp: 2026-05-23.

## Provenance

Slice 003 authored by delegated `general-purpose` subagent (agentId
`ab529acffb53c52ff`) per operator direction to spin work out for
context conservation. Operator reviewed the diff (`git diff
src/am/ec_hnsw/source.rs`), verified counts via grep, re-ran cargo
check before commit.

## Deferrals from slice 002, dispositioned here

| Slice-002 deferral | Slice-003 disposition |
| --- | --- |
| `DetoastedVarlena<'a>` lifetime | **Deferred again.** 9+ call sites outside HNSW (lib.rs, ec_spire/*, ec_diskann/*, ec_ivf/*). Cross-AM scope; properly belongs to Tasks 55/56/57. Recorded in closeout's handoff list. |
| `EcVectorView` wiring | **Stays a shim.** No `EcVector` type exists in the codebase (searched `src/quant/`, `src/storage/`, `src/am/common/`). Slice 002's `TODO(slice-003)` stays as documented design choice. |
| `flat_array_*` helper duplication | **Resolved.** All three HNSW-local copies deleted; consumers route through `common/datum.rs`'s private copies. |

## Behavior changes — flagged for reviewer

1. `resolve_source_attnum` folds the previous "invalid NUL byte"
   diagnostic into `AttnumLookup`'s "does not name a user column"
   message. Behavioral parity for any well-formed column name;
   user-supplied NUL-bearing names get the missing-column error
   instead. Low risk (build-time only; no scan-path impact).

No other behavior changes — all migrations are signature-preserving
substitutions of the typed wrappers for the open-coded patterns.
