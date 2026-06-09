# Task 91: Cross-AM `QuantCodec` Trait Migration

Status: proposed (2026-06-08)
Owner: coder (to be assigned). One coder, one branch.
Priority: 2 (kernel/architecture unification follow-up to Task 87)

## Why

Task 87 surfaced a shared `QuantCodec` trait
(`src/am/common/quant_codec.rs`) as the natural home for the
quantizer-family layer that ADR-071 targeted but explicitly deferred
to follow-up work. Task 87 packet 008 wired IVF onto the trait as a
first beachhead.

Task 87's own goal — candidate-batched scoring with measurable
scoring-share latency wins — has different validation gates,
different Stop Conditions, and a different closeout matrix than a
cross-AM refactor. Bundling them is a category error: Task 87's
"back the AM out if <5% scoring-share win" Stop Condition wrongly
threatens refactor work that has no perf goal, and the refactor's
behavioral-parity gates do not exercise Task 87's perf premise.

Task 91 owns the refactor side cleanly so Task 87 can ship its
performance work on its own timeline.

## Scope

### In scope

1. **Trait audit and growth** of `QuantCodec` (`src/am/common/
   quant_codec.rs`) so the surface holds for HNSW and DiskANN before
   migration. Suspected gaps to resolve in this phase:
   - QJL residual-sign metadata sidecar (HNSW `TiledLut` and
     `Int8Approx` exact-score paths);
   - grouped-PQ model state ownership (`encode_source` cannot today
     reach the trained PQ model — packet 008 flagged this as a known
     limit on IVF);
   - dispatch shape: `dyn QuantCodec` vs `enum DispatchedQuantCodec`,
     given `QuantCodec` has an associated `PreparedQuery` type.
     Enum dispatch is the default pick; revisit only if a sealed enum
     turns out to break Task 88 streaming composition.
2. **HNSW migration** of the scoring path
   (`src/am/ec_hnsw/scan.rs:5005–5037`: the three
   `TurboQuantExactScoreMode` branches + the gamma-aware fallback +
   PqFastScan + RaBitQ if present) onto `QuantCodec`. The existing
   `HnswStorageCodec` storage-binding adapter
   (`src/am/ec_hnsw/codec.rs`) is **kept**: it owns metadata page
   construction, tuple-fit checks, and page-format identity, which
   are correctly the index-local adapter layer per ADR-072. Its
   variants gain a method that returns the right `QuantCodec` for
   their format.
3. **DiskANN migration** of `DiskannPreparedPrefilter::{BinarySidecar,
   GroupedPq, RaBitQ}` (`src/am/ec_diskann/quantizer.rs:152`) onto
   `QuantCodec` impls. `DiskannBuildCodec` stays as the storage
   binding; the quant arms move to the trait.
4. **DiskANN TurboQuant codec landing** — absorbs Task 90's content.
   Add a `TurboQuant` `QuantCodec` impl plus the
   `VAMANA_SEARCH_CODEC_TURBOQUANT` discriminator, metadata bytes,
   build/insert encode, and prefilter scorer. Task 90 closes by
   reference to Task 91's DiskANN-TQ slice.
5. **SPIRE follow-through** so any SPIRE scoring paths still on the
   pre-Task-87 inline shape route through `QuantCodec` (current
   `SpireAssignmentQuantCodec` impl covers the assignment scorer;
   audit selected-block and RaBitQ-cutoff paths).
6. **Rename** `HnswStorageCodec` → `HnswStorageBinding` (or
   `HnswStorageAdapter`) so "codec" unambiguously means the
   `QuantCodec` trait. Same rename pattern applied to DiskANN's
   storage binding if its name reads similarly.
7. **ADR updates**:
   - ADR-071 status flip from PROPOSED → ACCEPTED, narrowed to
     "quantizer-family layer adopted via `QuantCodec` in Task 91";
   - ADR-072 amendment noting the quantizer-family layer is no longer
     index-local — it is `QuantCodec` — while the storage-binding
     layer remains correctly index-local;
   - ADR explicitly closes the "do not start a flag-day extraction"
     framing for the quantizer-family layer only.
8. **Behavioral parity tests** across all four AMs for every existing
   storage format and every existing scoring path.

### Out of scope

- New SIMD kernels. Kernel work lives in Task 87 (no-QJL 4-bit
  batched) and any follow-ups, not here.
- `CandidateBatch` data-flow abstraction. Task 87 owns it.
- Streaming ANN result iteration (Task 88).
- TurboQuant TQ+ calibration (Task 89).
- On-disk format changes other than the DiskANN TurboQuant search-code
  discriminator landing under §4.
- Performance regression chasing beyond the no-regression gate. This is
  a refactor, not a perf task.

## Acceptance criteria

1. Exactly one `QuantCodec` trait exists in the tree, and **every
   quant×AM scoring path goes through it.** No bespoke per-AM
   prepared-query enum carries quant kernel knowledge after migration.
2. `HnswStorageCodec` is renamed and its quantizer-family
   responsibilities are gone; only storage-binding responsibilities
   remain.
3. DiskANN exposes a TurboQuant search codec through `QuantCodec`,
   discoverable via the same registration step the other AMs use.
4. Task 90 is closed by reference to this task's DiskANN-TQ slice.
5. ADR-071 status flipped and ADR-072 amended per §7.
6. Per-AM behavioral parity: recall byte-equal at every existing
   format × fixture cell against the pre-migration baseline.
7. No measurable latency regression at p50/p95/p99 on any existing AM
   path at the project's standard real-corpus suite.
8. All existing `pg_test` surfaces pass across all four AMs.

## Phases

### Phase 1 — Trait audit and growth packet

Design-only packet that:

- enumerates every existing per-AM scoring path and its `QuantCodec`
  fit;
- grows the trait surface for residual-sign metadata, grouped-PQ
  model state, and dispatch shape;
- specifies which existing per-AM enums collapse into the trait vs
  which remain (storage-binding) adapters;
- specifies the dispatch decision (`dyn` vs enum) and rationale;
- **specifies `QuantCodec::score_batch` as the universal block-kernel
  dispatch entry point.** Tasks 93–98 each implement scalar + ISA-
  gated kernel variants and register them through this method.
  Phase 1 must lock in: (a) the kernel-registration shape per quant,
  (b) how `CandidateBatch::len() >= 32` width-based gating composes
  with the trait method, and (c) the ULP-tolerance contract for SIMD
  variants (strict bit-equality on scalar reference, ULP tolerance
  on SIMD variants per ADR-076).

No code. Reviewer approves before Phase 2.

### Phase 2 — IVF trait-growth retouch

The IVF impl already landed in Task 87 packet 008 is on the pre-growth
trait shape. Phase 2 retouches it to the grown trait so the IVF
adapter is the reference impl for Phase 3–5. Includes the grouped-PQ
model-ownership fix Phase 1 specified.

### Phase 3 — SPIRE migration

Audit SPIRE scoring paths beyond `SpireAssignmentQuantCodec`; route
selected-block / RaBitQ-cutoff paths through `QuantCodec`. Per-AM
behavioral parity gate.

### Phase 4 — HNSW migration

Route HNSW's three TQ exact-score branches + gamma fallback +
PqFastScan + RaBitQ scoring through `QuantCodec`. Rename
`HnswStorageCodec`. Per-AM behavioral parity gate.

### Phase 5 — DiskANN migration

Route grouped-PQ, RaBitQ, binary-sidecar prefilters through
`QuantCodec`. Storage-binding stays in `DiskannBuildCodec` /
`DiskannPreparedPrefilter` (rename if §6 dictates). Per-AM behavioral
parity gate.

### Phase 6 — DiskANN TurboQuant landing (absorbs Task 90)

Add the TurboQuant `QuantCodec` registration on DiskANN: metadata
discriminator, build/insert encode, prefilter scorer. Per the Task 90
acceptance gates plus this task's behavioral parity gate.

### Phase 7 — ADR updates + closeout

Flip ADR-071 status. Amend ADR-072. Close Task 90 by reference.
Aggregate behavioral parity table across all AMs + formats. Status
flip to `complete`.

## Coordination

- **Depends on Task 87** Phase 1 (`CandidateBatch` and `QuantCodec`
  beachhead) landing. Task 91 sits on top of those; Task 87 can ship
  perf slices in parallel with Task 91 phases as long as the trait
  audit (Task 91 Phase 1) does not invalidate Task 87's in-flight
  slice code.
- **Absorbs Task 90.** Task 90 task file should be marked
  `superseded by Task 91` rather than `complete`.
- **Reads against Task 64.** Task 64's HNSW codec adapter is the
  ADR-072 storage-binding pattern; this task does **not** undo Task
  64's work. It moves the quantizer-family responsibilities out of
  HNSW's storage binding into `QuantCodec`.

## Validation gate (per AM)

1. Recall byte-equal at every storage-format × fixture cell vs the
   pre-migration baseline.
2. No p50/p95/p99 latency regression at the project's standard
   real-corpus suite.
3. Storage layout unchanged (no format change in scope except DiskANN
   TurboQuant's new discriminator in Phase 6).
4. All existing `pg_test` surfaces pass for the AM under migration.
5. No new `unsafe` outside existing AM/quantizer boundaries.

## Stop conditions

- If `QuantCodec` trait growth (Phase 1) requires breaking the
  `CandidateBatch` data-flow contract that Task 87 already stabilized,
  pause Task 91 and reopen Task 87 Phase 1 for joint revision.
- If any AM migration cannot reach behavioral parity within a single
  slice, file a Stop Condition packet naming the parity gap and the
  follow-up needed.
- If `dyn QuantCodec` vs enum-dispatch turns out to require a deeper
  type-erasure layer than Phase 1 specified, pause and reopen the
  dispatch design.

## References

- ADR-071 (unified quantizer interface)
- ADR-072 (index-local quantized codec adapters)
- Task 64 (HNSW quantized codec adapters — storage-binding layer)
- Task 87 (`CandidateBatch` + per-AM batched scoring)
- Task 87 packet 007 (Phase 1 common-codec scope revision) and packet
  008 (`QuantCodec` trait + IVF adapter beachhead)
- Task 90 (DiskANN TurboQuant search codec — absorbed)

## Estimated size

Large. 6–10 weeks for one coder including trait growth, four AM
migrations, the DiskANN TurboQuant landing, ADR updates, and
behavioral parity evidence per AM. DiskANN Phase 6 is the highest-
risk slice because it adds a new search-codec discriminator on top of
the migration.
