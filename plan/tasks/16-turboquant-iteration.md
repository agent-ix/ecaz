# Task 16: TurboQuant Iteration with PqFastScan Learnings

Status: in progress — levers 1–3 landed and were measured; lever-4 / lever-5
comparison closed in packet `437`, but the serious-lane closure goal is still open.

Follow-on to ADR-032.

## Scope

With PqFastScan merged as a first-class peer format (task 15), revisit the
TurboQuant format and port the three architectural wins that made
PqFastScan fast. Goal: narrow the TurboQuant vs PqFastScan latency gap on
the 50k warm real seam enough that TurboQuant remains a credible choice
for workloads that do not need PqFastScan's absolute throughput.

## Context

ADR-021 and ADR-025 established that TurboQuant scoring at 1536@4-bit is
bottlenecked by a 96 KB LUT that blows Graviton's 64 KB L1D. The scoring
kernel itself is well-optimized (AVX2/NEON) — the problem is the cache
working set.

The PqFastScan detour proved out three patterns that attack the same
bottleneck from different directions. All three already have code on
this branch — they are not new inventions, they are re-plumbings onto
the TurboQuant path.

## Levers (in expected-win order)

### Lever 1: Binary prefilter → TurboQuant rerank

Use the RaBitQ sign-bit sidecar from ADR-031
(`BinarySignNoQjl4BitQuery` in `src/quant/prod.rs`) as a cheap first
stage. Only candidates that survive the binary filter pay the TurboQuant
LUT cost. Mirrors the PqFastScan pipeline with the second-stage scorer
swapped in.

### Lever 2: Heap-f32 rerank

Port `GroupedRerankMode::HeapF32` (`src/am/scan.rs:462`) from PqFastScan
onto TurboQuant scans. With exact rerank coming from the heap, traversal
can drop to the 4+0 TurboQuant_mse fast path (no QJL), cutting LUT size
and per-edge payload. ADR-025 §Mitigation 3 sketched this but never
shipped it.

### Lever 3: Hot/cold payload split

Move QJL bits and gamma out of the hot graph tuple. Keep only
scoring-critical bytes inline per graph edge; park QJL and gamma on a
cold page read only for survivors. Mirrors the PqFastScan hot/cold
payload model. Likely requires an `INDEX_FORMAT_V3` wire bump rather
than an in-place change.

### Lever 4 (optional): Tiled LUT scoring

ADR-021 §6 open question. Process scoring in 512-dim tiles so each LUT
tile (~32 KB) fits in L1D alongside the `sq` and candidate slices.
More invasive than levers 1–3. Measure first whether levers 1–3 make
this unnecessary.

### Lever 5 (optional): int8 LUT

`Int8ApproxNoQjl4BitQuery` already exists in `src/quant/prod.rs`. Shrinks
the LUT 4x. Composes with tiling.

## Subtasks

- [x] **Close the deferred `418` measurement note** with one `50k`
  before/after build-time row for the landed
  `BuildCodeDistance::new(...)` hot-path correction. This is not a task-15
  blocker, but it should be recorded before broader TurboQuant iteration
  work starts. Closed in packet `422`.
- [x] **Instrument TurboQuant scan path** for per-stage cost on the 50k
  warm real seam (traversal, LUT gather, QJL accumulate, rerank). Baseline
  numbers land in a measurement packet before any levers are wired. Closed in
  packet `423`.
- [x] **Lever 1 — binary prefilter** ahead of TurboQuant scoring.
  Confirm the sidecar can be built either from existing code bytes or
  as a persisted payload (ADR-031 already enumerates both). Packet `423`
  showed this was already active on current head; no new task-16 code landing
  was required for the lever itself.
- [x] **Lever 2 — heap-f32 rerank** mode for TurboQuant scans. Reuse the
  heap-fetch infrastructure added for PqFastScan (tuple slot,
  `grouped_heap_rerank_source_attnum` pattern). Landed in packets `424` / `425`.
- [x] **Lever 3 — hot/cold payload split** for TurboQuant element
  tuples. Treat as `INDEX_FORMAT_V3` rather than in-place mutation of
  `INDEX_FORMAT_V1_SCALAR`. Landed in packets `427` / `428`.
- [x] **Measurement packet** comparing TurboQuant-today vs
  TurboQuant-with-levers-1–3 at the same recall target on 50k warm real
  seam. Closed across packets `423`, `426`, `429`, `430`, and `432`.
- [x] **Decide** whether to pursue lever 4 (tiled LUT) and/or lever 5
  (int8 LUT) based on whether levers 1–3 close the gap. Closed across packets
  `433`, `436`, and `437`: lever 4 is real on the live quantized lane but too
  small on the recall-preserving `heap_f32` lane, while lever 5 is not
  justified for the task-16 serious-lane goal.

## Owns

- Iteration track for ADR-006 (TurboQuant quantizer) latency.

## Dependencies

- **Task 15 must land first.** This task assumes PqFastScan is a stable
  peer format and reuses its hot/cold pagination and heap-rerank
  machinery. Starting before task 15 merges means building infrastructure
  twice.

## Unblocks

- A TurboQuant format that is fast enough to keep as the default for
  most workloads.
- A clean answer to "when should I pick TurboQuant over PqFastScan?".

## Outcome So Far

- TurboQuant's fast quantized lane improved materially.
- The recall-preserving serious lane remained dominated by heap rerank/source
  fetch-decode cost in the measurements so far.
- A packed raw-f32 rerank source helps that serious lane, but the lever-4 /
  lever-5 live matrix shows lever 4 helps the quantized lane more than lever 5,
  while neither closes the serious lane.
- **Packet `441` located the remaining serious-lane cost in heap-source
  storage layout, not scorer math.** Forcing `source_raw bytea` inline via
  `ALTER COLUMN … SET STORAGE PLAIN` on a mixed-inline corpus reduced the
  q200 serious lane from `4.838ms` to `3.137ms` (`-35.16%`) with recall
  bit-identical (`graph_recall_at_10 = 0.9629`,
  `mean_abs_score_error = 0`) and the rerank-decode micro-bucket collapsing
  from `1386us` to `1us`. That is the measurement that productizes the
  ADR-043 `ecvector` native-type direction.
- **Vacuum-concurrency regression from packet `437` is fixed in packet
  `438`.** `scripts/vacuum_concurrency_scratch.sh --duration 60` now passes
  on current head; the stale metadata-entry-point repair lives in generic
  AM code (`src/am/shared.rs`, `src/am/vacuum.rs`, `src/am/scan.rs`).
- **Packet `442` replaces the old quantized-row surface with a canonical
  `ecvector` row model plus `ecqvector` sibling artifact type.** The indexed
  column can now be raw `ecvector(dim)` by default, `heap_f32`/build-source
  paths fall back to that indexed column when no alternate source column is
  configured, and the explicit quantized-artifact tests now live on
  `ecqvector` instead of the removed `tqvector` SQL type.

## Landing checklist

Task 16's formal subtasks (1–7 above) are all closed. Before task 16 can
**merge**, the following items still need to land. Items are independent
unless called out.

### Measurement

- [ ] **Rerun packet `440` at q200 ×≥2.** One run per side established
  `-4.33%` for persisted `source_raw` vs `source`. Rerun to confirm the
  direction survives restart noise. Cheap; unblocks treating the supported
  path as a validated runtime win rather than a single-cell inference.
- [ ] **Rerun packet `441` at q200 ×≥2** on the `tqhnsw_real_50k_tq_mixed_inline_corpus`
  surface. The `-35.16%` delta is far outside the packet-`432` noise
  envelope, but one confirming run makes the task-16 headline number
  bulletproof.
- [ ] **Head-to-head vs PqFastScan on the same inline surface.** Task 16's
  stated outcome goal is "narrow the TurboQuant vs PqFastScan latency gap
  on the 50k warm real seam". No packet in the 422–441 arc has put
  TurboQuant-with-inline-source next to PqFastScan on the same corpus,
  recall target, and runtime. Add one q200 cell on PqFastScan against the
  mixed-inline (or equivalent) surface. Without this cell, task 16 closes
  the gap question by inference, not measurement.
- [ ] **ef_search matrix for lever-4 `full_lut` on the quantized lane.**
  Packet `437` showed `-16%` at `ef_search = 128` on one cell. Before
  lever 4 becomes any kind of persisted default, run `ef_search = 64 /
  128 / 256` so the decision rests on a shape, not a point.
- [ ] **Mixed-inline storage cost measured as an explicit tradeoff.**
  Packet `441` showed heap footprint grew from `43MB` to `390MB`
  (≈9×) for the `-35%` latency win. Quantify what this means for
  buffer-cache pressure, vacuum cost per page, WAL on updates, and
  index build time — all named as a tradeoff in the readout, not just
  a win.

### Productization

- [ ] **ADR-043 ratified.** Status PROPOSED → ACCEPTED. Open-questions
  resolution:
  - Name: **RESOLVED — `ecvector`** (Ecaz).
  - pgvector cast policy: lean install-time conditional.
  - Bare-typmod support: tentative yes.
- [x] **Task 17 implementation: `ecvector` column type.** Packet `442`
  lands the canonical `ecvector` row model and removes the prior
  `tqvector` SQL type in favor of an explicit `ecqvector` sibling
  artifact type.
- [ ] **Task 16's head-to-head measurement uses `ecvector`, not the
  bytea+`STORAGE PLAIN` recipe.** The recipe was the research surface;
  the closure measurement runs on the productized type so the
  "narrow the gap" answer is in the same terms users will adopt.
- [ ] **Document `ecvector` as the column type** in README/quickstart.

### Lever decisions

- [ ] **Lever 4 (`full_lut` on quantized lane).** After the ef_search
  matrix, decide: persist as reloption, flip as default, or leave as
  experimental `TQVECTOR_TURBOQUANT_EXACT_SCORE_MODE` env. Document
  the choice + rationale.
- [ ] **Lever 5 (`int8_approx`).** On current x86 host, packet `437`
  showed `+2.97%` regression on heap-f32 lane and neck-and-neck on
  quantized. Direction is "not justified on this host"; keep code on
  branch until NEON / Graviton / Apple hardware tests. Add per-hardware
  tuning notes to ADR-025 naming the NEON-no-f32-gather constraint and
  the cache-hierarchy deltas that could invert the lever-4/lever-5
  ordering on Arm.

### Infrastructure and hygiene

- [x] Vacuum concurrency regression fixed (packet `438`).
- [ ] **Plan file outcome section updated** after each rerun /
  head-to-head cell lands. Task 16's outcome narrative should reflect
  measured truth, not just the 441 snapshot.
- [ ] **Script-surface test for `ALTER INDEX … SET/RESET
  (rerank_source_column)`.** Packet `440`'s methodology relies on the
  ALTER cycle round-tripping; lock this in with a `pg_test` so a future
  refactor cannot silently break the measurement reproducibility.
- [ ] **`install_adr030_pg17_pg_test.sh` → backend `.so` version
  assertion.** Packet `440` caught a stale-install hazard manually
  (flat `~26.96ms` tipped it off). A script-level version check would
  convert that manual diagnosis into an automatic safety belt.

### Merge

- [ ] All measurement items above green.
- [ ] ADR-043 ACCEPTED; canonical `ecvector` row model landed.
- [ ] Task 16 merged to `main`.

### Unblocks on merge

- ADR-042 (native HNSW build path) — already PROPOSED, composes with
  ADR-043 per ADR-043 §Relationship.
- Billion-scale follow-on work (ADR-034 DiskANN, ADR-035 SPANN) —
  both benefit from `ecvector` as a compact, type-safe HeapF32 rerank
  source.

## Out of scope

- Changing the TurboQuant quantizer itself (MSE codebook, QJL stage
  structure). Levers here are scoring-pipeline and payload-layout
  changes around the existing quantizer.
- OPQ transform front-end (PqFastScan-side follow-on, ADR-030).

## Notes

- Main tradeoff to watch: levers 1 and 2 reduce *how many* vectors get
  TurboQuant-scored. If Layer-0 traversal with a large frontier remains
  hot, per-candidate cost still dominates and levers 3–5 become
  load-bearing.
- Start with levers 1 + 2 — most of the wiring exists on this branch
  already for the PqFastScan path.
