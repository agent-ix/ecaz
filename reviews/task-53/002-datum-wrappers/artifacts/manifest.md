# Task 53 / 002 — Datum Wrappers · Artifact Manifest

Packet path: `reviews/task-53/002-datum-wrappers/`
Branch: `task-53`

## Surfaces

- `src/am/common/datum.rs` — NEW module, 311 lines, 15 wrapper-side unsafe blocks.
- `src/am/common/detoast.rs` — +25 lines (`as_typed_slice<T: Copy>`), 4 → 5 unsafe blocks.
- `src/am/common/mod.rs` — `pub(crate) mod datum;`.
- `src/am/ec_hnsw/source.rs` — UNCHANGED at 29 unsafe blocks (slice 002 is wrapper-only; consumer migration is slice 003).

## Per-file before/after `unsafe { ... }` blocks

| File | Pre | Post | Delta |
| --- | ---: | ---: | ---: |
| `src/am/common/datum.rs` (new) | — | 15 | +15 (wrapper-side) |
| `src/am/common/detoast.rs` | 4 | 5 | +1 (`as_typed_slice` `align_to`) |
| `src/am/ec_hnsw/source.rs` | 29 | 29 | 0 |

Planning packet's `baseline-unsafe-density.txt` reported
`detoast.rs = 8` — that was wrong (transcription mistake by the
planner). Actual baseline confirmed by grep: 4. This packet uses the
correct on-disk baseline.

## Artifacts

This slice's evidence is static: counts and the diff. No standalone
JSON / log artifact needed.

- Head SHA: parent of packet commit.
- Lane / fixture / storage / rerank: N/A (compile-only).
- Isolation: N/A.
- Command (validation):
  - `cargo fmt --all` — clean (touched only in-scope files; the
    workspace's pre-existing fmt drift on other files is left
    untouched per CLAUDE.md "do not revert unrelated local changes").
  - `cargo check --no-default-features --features pg18` — `Finished`
    exit 0, 2m 11s (subagent's reported timing).
- Timestamp: 2026-05-23.

## Provenance

Slice 002 was authored by a delegated `general-purpose` subagent
(agentId `a2b0215271b21b9e3`), per operator direction to "spin out
in subagents, conserve your context". The subagent's return summary
is reflected verbatim in `request.md` §"What landed".

I (operator-context coder) reviewed the diff before commit:
- `git diff src/am/common/detoast.rs src/am/common/mod.rs` inspected.
- Sampled `src/am/common/datum.rs` structure (line layout, exported
  types, fn signatures).
- Re-ran `cargo check` to confirm `Finished` exit 0.
- Verified unsafe counts match the subagent's report.
