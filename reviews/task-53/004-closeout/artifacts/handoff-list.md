# Task 53 Cross-AM Handoff List

Per Task 53 §Exit Criterion #4 — naming each SPIRE / IVF / DiskANN
consumer site that the new `src/am/common/datum.rs` (slice 002)
wrappers will absorb under Tasks 55/56/57.

The wrappers landed in this task are HNSW-only consumer-migrated;
the cross-AM consumer migration is intentionally deferred per Task
53 §Non-Goals ("Do not migrate IVF or SPIRE consumer call sites in
this task. They will consume the wrappers under Tasks 57 ... and 56
...").

## Wrapper inventory (slice 002)

Available for cross-AM consumers:

- `src/am/common/datum.rs::FlatFloat4Source<'a>` — unified flat-float4
  array/varlena dispatch. Replaces ad-hoc detoast + `ArrayType` header
  read + `from_raw_parts` data slice patterns.
- `src/am/common/datum.rs::EcVectorDatum<'a>` / `EcVectorView<'a>` —
  thin shim over `FlatFloat4Source<Varlena>`. Reviewer call: when a
  real `EcVector` type lands, wire this through; until then the shim
  is a stable wrapper boundary.
- `src/am/common/datum.rs::AttnumLookup` — safe `pg_sys::get_attnum`
  wrapper.
- `src/am/common/detoast.rs::DetoastedVarlena::as_typed_slice<T: Copy>`
  (slice 002 enhancement) — typed-slice view with strict alignment
  validation. Replaces `bytes.align_to::<T>()` + slice patterns.
- (Deferred) `DetoastedVarlena<'a>` lifetime promotion — requires
  cross-AM call-site rewrites; **the AM-specific tasks 55/56/57 should
  decide collectively when to promote** (suggest: in whichever AM
  task lands first, since the lifetime change is mechanical once one
  AM consumes it lifetime-aware).

## Task 56 — SPIRE handoff

Consumer sites in `src/am/ec_spire/`:

| File:Line | Pattern | Absorbing wrapper |
| --- | --- | --- |
| `build/tuples.rs:58` | `unsafe { DetoastedVarlena::packed_from_datum(datum) }` | `DetoastedVarlena` direct use; can become `DetoastedVarlena<'a>` when lifetime lands. |
| `scan/relation.rs:270` | `unsafe { DetoastedVarlena::packed_from_datum(datum) }` (in `let bytes = ...` chain) | Same. Plus consider `as_typed_slice::<f32>()` if the bytes are reinterpreted as f32 downstream. |
| `build.rs:29`, `scan.rs:29` | imports of `DetoastedVarlena` | Will pick up the lifetime when promoted. |

Estimated reduction at Task 56's source.rs / scan equivalents: -2 to
-4 unsafe blocks once `DetoastedVarlena<'a>` lifetime lands and
typed-slice migration completes.

## Task 57 — IVF handoff

Consumer sites in `src/am/ec_ivf/`:

| File:Line | Pattern | Absorbing wrapper |
| --- | --- | --- |
| `build.rs:759` | `unsafe { DetoastedVarlena::packed_from_datum(datum) }` (with SAFETY comment at 757-758) | `DetoastedVarlena` direct use. |
| `scan.rs:59` | `unsafe { std::slice::from_raw_parts(self.query_values, self.query_dimensions as usize) }` | Convert to typed accessor on a wrapper over `IvfScanQuery` (out of P6 scope but P3-adjacent — flag for Task 54 / Task 57 coordination). |
| `scan.rs:87` | same as :59 | same |
| `scan.rs:98` | `unsafe { ... from_raw_parts(self.selected_lists, ...) }` | same — out of P6 scope. |

Estimated reduction at Task 57: -1 to -2 P6 unsafe blocks (datum
boundary). The `query_values` / `selected_lists` slice constructions
are P3 / IVF-internal patterns; Task 57's scope is the IVF AM's
unsafe burndown, which may extend beyond P6.

## Task 55 — DiskANN handoff

Consumer sites in `src/am/ec_diskann/`:

| File:Line | Pattern | Absorbing wrapper |
| --- | --- | --- |
| `ambuild.rs:866` | `unsafe { DetoastedVarlena::plain_from_datum(datum) }` | `DetoastedVarlena` direct use; lifetime-promote when ready. |
| `scan_state.rs:171` | `unsafe { slice::from_raw_parts(metadata_ptr.cast_const(), VAMANA_METADATA_BYTES) }` | Out of P6 scope (Vamana-internal metadata layout). Flag for Task 55 internal scope. |
| `insert.rs:1208, :1232` | `unsafe { slice::from_raw_parts(special, VAMANA_METADATA_BYTES) }` | Out of P6 scope (PG-page special-area read; P3-adjacent). |

Estimated reduction at Task 55: -1 P6 unsafe block (datum boundary
in `ambuild.rs`). The other DiskANN unsafe blocks are P3 / Vamana-
internal, addressed by Task 55's broader scope.

## Cross-task coordination notes

1. **`DetoastedVarlena<'a>` lifetime ordering** — whichever AM task
   (55, 56, or 57) lands first should perform the lifetime promotion;
   subsequent AM tasks pick up the change for free. Recommend Task
   55 (DiskANN) since DiskANN's consumer surface is smallest (1
   site) and the lifetime ripple is easiest to validate there.

2. **`EcVectorView` wiring** — if any AM task introduces a typed
   `EcVector` (e.g., as part of cross-AM source-vector representation
   consolidation), wire `src/am/common/datum.rs::EcVectorDatum::view()`
   through to it. Until then the shim stands.

3. **Task 54 (P3 page/WAL wrappers)** scope adjacency: the IVF
   `from_raw_parts(self.query_values, ...)` and DiskANN
   `from_raw_parts(special, VAMANA_METADATA_BYTES)` patterns may
   alternatively be absorbed by Task 54's page/WAL wrappers if the
   wrapper surface is broader than literal page reads. Coordinate at
   Task 54's planning packet.

## Verification commands for downstream tasks

To re-locate these consumer sites after rebases / merges:

```bash
grep -nE "DetoastedVarlena" src/am/ec_spire src/am/ec_ivf src/am/ec_diskann -r
grep -nE "from_raw_parts.*f32|pg_sys::ArrayType|pg_detoast|FromDatum" src/am/ec_spire src/am/ec_ivf src/am/ec_diskann -r
```

## Cross-references

- Slice 002 wrapper module: `src/am/common/datum.rs`.
- Slice 002 request: `reviews/task-53/002-datum-wrappers/request.md`
  §"Anomalies / deferrals for slice 003".
- Slice 003 request: `reviews/task-53/003-source-rs-consumer-migration/request.md`
  §"Slice 002 deferrals — dispositioned here".
- Task plans: `plan/tasks/55-diskann-unsafe-burndown.md`,
  `plan/tasks/56-spire-unsafe-burndown.md`,
  `plan/tasks/57-ivf-unsafe-burndown.md`,
  `plan/tasks/54-common-p3-page-wal-wrappers.md`.
