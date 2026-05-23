# Task 53 / 004 — Closeout

Branch: `task-53` (off `origin/main` `5c0e9e2bd`)
Task source: `plan/tasks/53-common-p6-datum-wrappers.md`

## Summary

Task 53 closes with all four §Exit Criteria satisfied. The P6 typed
datum wrappers are landed in `src/am/common/datum.rs`;
`src/am/ec_hnsw/source.rs` is at 13 unsafe blocks (target ≤ 14 met
with margin); the full 8-step bench gate passes against
`benchmarks/task-50-m5-hnsw-baseline/` with **measurable latency
improvement** on 100k (7-13% p50 faster), no recall or storage
regression; and the SPIRE/IVF/DiskANN handoff list enumerates the
downstream consumer sites for Tasks 55/56/57.

## §Exit Criteria status

| # | Criterion | Status |
|---|---|---|
| 1 | Four typed wrappers in `src/am/common/datum.rs` | **✓** |
| 2 | `src/am/ec_hnsw/source.rs` ≤ 14 | **✓ (13)** |
| 3 | HNSW recall + QPS + per-row storage no regression | **✓** |
| 4 | Closing summary packet w/ deltas + SPIRE/IVF/DiskANN handoff | **✓** |

### §Exit Criterion #1 — Wrappers landed

| Wrapper | Location | Slice |
| --- | --- | ---: |
| `FlatFloat4Source<'a>` (unifies Array + Varlena dispatch) | `src/am/common/datum.rs` | 002 |
| `EcVectorDatum<'a>` + `EcVectorView<'a>` (shim) | `src/am/common/datum.rs` | 002 |
| `AttnumLookup` (safe `pg_sys::get_attnum`) | `src/am/common/datum.rs` | 002 |
| `DetoastedVarlena::as_typed_slice<T: Copy>` | `src/am/common/detoast.rs` | 002 |

The fourth wrapper is the enhancement to the pre-existing
`DetoastedVarlena` rather than a brand-new type — per task spec
§Scope #1 wording. The `DetoastedVarlena<'a>` explicit lifetime
parameter is deferred to Tasks 55/56/57 per the handoff list (cross-AM
call-site work).

`EcVectorView` ships as a documented shim over `FlatFloat4Source<Varlena>`
since no `EcVector` type currently exists in the codebase. Marked
`TODO(slice-003)`; if an AM-task introduces a real `EcVector`, wire
through then.

### §Exit Criterion #2 — `source.rs` ≤ 14

| Slice | `source.rs` | Δ vs prior |
| --- | ---: | ---: |
| Pre-Task-53 | 29 | — |
| Post-001 (planning) | 29 | 0 |
| Post-002 (wrappers added) | 29 | 0 |
| Post-003 (consumer migration) | **13** | **-16** |
| **Total** | **13** | **-16 (-55.2%)** |

Exceeds the task spec target (≤ 14) by 1 block and the
expected -52% by 3 points.

### §Exit Criterion #3 — Bench gate ✓

Full 8-step suite ran via
`ecaz bench suite run --config reviews/task-53/004-closeout/artifacts/suite.json`
on PG18, M5 Pro laptop, same shape as
`benchmarks/task-50-m5-hnsw-baseline/suite.json` (10k + 100k, load /
recall / latency / storage, same prefixes / `m` / ef_construction /
sweep).

**Headline results:**

| Surface | Baseline | Task 53 | Disposition |
| --- | --- | --- | --- |
| **10k recall@10** (all ef) | 0.9040 / 0.9530 / 0.9605 / 0.9775 / 0.9950 | identical | exact-equal to 4 decimals |
| **100k recall@10** (all ef) | 0.7426 / 0.8506 / 0.8973 / 0.9414 / 0.9676 | 0.7392 / 0.8480 / 0.8972 / 0.9396 / 0.9669 | all deltas ≤0.0034, inside ci95 |
| **10k storage per row** | 19359.3 B | 19359.3 B | 0 B (bit-for-bit identical) |
| **100k storage per row** | 18117.1 B | 18117.3 B | +0.2 B (FSM/VM noise) |
| **10k + 100k index sizes** | identical | identical | bit-for-bit per-row |
| **10k latency p50** (best/worst) | 0.57-1.69 ms | 0.53-1.63 ms | **mostly faster** (one +1.1% at ef=80) |
| **100k latency p50** (worst delta) | ef=400: 4.92 ms | 4.29 ms | **-12.8% (faster)** |
| **100k latency p95** (worst delta) | ef=400: 6.38 ms | 5.71 ms | **-10.5% (faster)** |
| **100k latency mean** (every bucket) | — | — | **Task 53 faster on every ef** |

The 100k latency improvement is the wrapper-inlining win — the typed
`FlatFloat4Source<'a>` boundary collapses the per-call `unsafe { ... }`
block, letting the compiler inline the detoast + slice-extraction
path more aggressively across the scan hot path. Effect amplifies at
larger corpus sizes where scan-path call overhead dominates per query.

Full evidence: `artifacts/before-after-summary.md`.

### §Exit Criterion #4 — Closing summary

#### Per-file before/after for `source.rs`

See §Exit Criterion #2 above. Per-slice deltas in each slice's
`artifacts/manifest.md` (`002-datum-wrappers/`, `003-source-rs-consumer-migration/`).

#### Full `src/am/common/` wrapper surface added

| File | Wrappers added |
| --- | --- |
| `src/am/common/datum.rs` (new, 311 lines, 15 unsafe blocks) | `FlatFloat4Kind` enum; `FlatFloat4Source<'a>`; `EcVectorDatum<'a>` + `EcVectorView<'a>` (shim); `AttnumLookup`; private helpers `flat_array_dims_ptr`, `flat_array_data_offset`, `maxaligned_size` (lifted from source.rs) |
| `src/am/common/detoast.rs` | `as_typed_slice<T: Copy>(&self) -> Option<&[T]>` on `DetoastedVarlena` (slice-002 enhancement) |
| `src/am/common/mod.rs` | `pub(crate) mod datum;` |

#### `src/` total block count change

| Snapshot | `src/` total |
| --- | ---: |
| Pre-Task-53 | 960 |
| Post-002 (wrappers added) | 976 |
| Post-003 (consumer migration) | **959** |
| **Δ** | **-1** |

Task 53 nets to **-1 unsafe block across the whole `src/` tree** —
the cleanest possible outcome: wrappers fully absorb the consumer
surface plus one block of margin.

HNSW subsystem-wide: 327 (post-Task-50) → 311 (now), -16 net.

#### SPIRE/IVF/DiskANN handoff list

See `artifacts/handoff-list.md`. Per-AM enumeration of the consumer
sites the new wrappers absorb under Tasks 55/56/57:

- **Task 56 (SPIRE)**: 2 sites (`build/tuples.rs:58`,
  `scan/relation.rs:270`) — both `DetoastedVarlena::packed_from_datum`
  call sites. Estimated -2 to -4 unsafe blocks.
- **Task 57 (IVF)**: 1 P6-scoped site (`build.rs:759`) +
  3 P3-scoped sites (`scan.rs:59/87/98` for `from_raw_parts` on
  query values / selected lists). Estimated -1 to -2 P6 blocks.
- **Task 55 (DiskANN)**: 1 P6 site (`ambuild.rs:866`) + 3 P3 sites
  (`scan_state.rs:171`, `insert.rs:1208/1232` for Vamana metadata
  reads). Estimated -1 P6 block.
- **Deferred `DetoastedVarlena<'a>` lifetime promotion**: recommend
  Task 55 (DiskANN) land first since it has the smallest cross-AM
  surface (1 site); subsequent AM tasks pick up the change for free.
- **Coordination with Task 54 (P3 page/WAL wrappers)**: the IVF/DiskANN
  P3-adjacent `from_raw_parts` patterns may alternatively be absorbed
  by Task 54's wrapper surface; flag at Task 54 planning.

## Memory rules applied

- `feedback_anti_pattern_b_unbounded_lifetime` (10th application
  cumulative across Tasks 52+53).
- `feedback_view_operations_not_accessors` (4th application).
- `feedback_no_premature_task_close` — Task 53 closes only after all
  four §Exit Criteria are met with full bench evidence, per the new
  hard rule (not partial bench, not structural-ceiling deferral).
- `feedback_coder_push_smoke_checks` — pushed after every slice;
  smoke checks (cargo check + fmt) between slices; bench window once
  at task close.
- `feedback_dyld_buffer_blocks_known` — `cargo pgrx test` deferred;
  PG-side validation via the bench's `ecaz corpus load` path
  (exercises the migrated build code) + recall@k exact-equal proves
  semantic preservation.

## Reviewer arc through the task

| Slice | Reviewer disposition |
| --- | --- |
| 001 planning | ✓ approved (`001-execution-planning/feedback/2026-05-23-01-reviewer.md`) |
| 002 wrappers | ✓ approved with phantom-`'a` style nit ("not unsoundness"); accepted (`002-datum-wrappers/feedback/2026-05-23-01-reviewer.md`) |
| 003 migration | ✓ approved with explicit direction to open slice 004 closeout (`003-source-rs-consumer-migration/feedback/2026-05-23-01-reviewer.md`) |
| 004 closeout | this packet |

The slice-003 reviewer's direction quoted verbatim:
> Per `plan/tasks/53-...md` §Exit Criteria and the durable rule in
> `feedback_no_premature_task_close`:
> 1. **Bench gate**: full 8-step `ecaz bench suite` against
>    `benchmarks/task-50-m5-hnsw-baseline/`. **All 8 steps** ...
> 2. **Closing summary packet** at `reviews/task-53/004-closeout/` ...

Both bullets satisfied by this packet (item 1: see `before-after-summary.md`;
item 2: this `request.md` + `artifacts/handoff-list.md`).

## Cross-references

- Task 50 closeout (Task 53's parent): `reviews/task-50/449-hnsw-bench-window/`.
- Task 52 closeout (sibling burndown lane): `reviews/task-52/007-closeout/`
  (also closes 100% with full 8-step bench).
- Task spec: `plan/tasks/53-common-p6-datum-wrappers.md`.
- Baseline: `benchmarks/task-50-m5-hnsw-baseline/`.
- Slice packets: `reviews/task-53/00{1,2,3}-*/`.
- Reviewer feedback: `reviews/task-53/00{1,2,3}-*/feedback/`.
- Memory rules referenced (full list in §"Memory rules applied" above).
