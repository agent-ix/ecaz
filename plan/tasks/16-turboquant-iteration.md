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
- **Packet `442` replaces the old canonical quantized-row surface with a
  canonical `ecvector` row model.** The indexed column can now be raw
  `ecvector(dim)` by default, and `heap_f32`/build-source paths fall back to
  that indexed column when no alternate source column is configured. The old
  public-row meaning of `tqvector` was removed from the product model.
- **Packet `443` narrows `tqvector` to the TurboQuant-family persisted
  quantized artifact.** Commit `8e2add6` renames the transitional
  `ecqvector` sibling back to `tqvector` (`tqvector_ip_ops`,
  `encode_to_tqvector(...)`, `IndexedVectorKind::Tqvector`). The name's
  scope has been narrowed: `tqvector` is now an explicit artifact type for
  TurboQuant-family tests/tooling/debugging, not a canonical row type.
  Taxonomy rule established: family-specific persisted quantized artifacts use
  family-specific sibling names.
- **Packet `445` compacts the TurboQuant sibling artifact and locks in
  canonical/sibling separation.** Current head now stores `tqvector` as a
  compact canonical artifact (`dim + gamma + code bytes`; `bits=4`,
  `seed=42` are enforced invariants, not per-row bytes) and adds a pg test
  proving an indexed `ecvector` column does not silently fall back to a
  sibling `tqvector` column on `pq_fastscan`.
- **Packet `446` measures the serious lane on the real `ecvector`
  surface.** Default-storage `ecvector` leaves the serious lane in the
  `5.2ms-5.9ms` range (`turboquant` vs `pq_fastscan` at `m=16`,
  `ef_search=128`), while inline-storage `ecvector` carries the packet-`441`
  win onto the productized type:
  - inline `turboquant`: `3.427ms`, confirming rerun `3.195ms`
  - inline `pq_fastscan`: `2.987ms`, confirming rerun `2.954ms`
  - recall stayed pinned at `0.9629` for `turboquant` and `0.9635` for
    `pq_fastscan`
  The result is now explicit: the decisive lever is still heap storage
  layout, and on the same inline `ecvector` serious lane `pq_fastscan`
  remains faster than `turboquant`.
- **Packet `447` closes the mixed-inline tradeoff question.** Forcing
  canonical `ecvector` inline is not a storage-footprint explosion so much as
  a storage-placement shift: total heap+TOAST bytes stay about flat
  (`823.0MB` default vs `819.2MB` inline), but the heap working set moves from
  `468` pages to `50,000` pages on a cluster with `16,384` buffer pages.
  Consequences measured on the 50k seam:
  - vacuum scan cost stayed essentially flat (`19.121s` default vs `19.250s`
    inline) because total pages scanned stay nearly the same
  - fresh TurboQuant build time was slightly better inline
    (`180.774s -> 173.784s`, `-3.87%`)
  - small row rewrites became materially heavier inline
    (`4.0MB -> 14.3MB` WAL on the steady 1k-row update batch, `3.56x`, with
    HOT dropping from `38` to `0`)
  So the inline lever is now clearly a workload policy question: strong
  serious-lane win for read-mostly rows, real row-churn penalty for mutable
  rows.
- **Reviewer feedback plus ADR-044 keep the storage-policy default open.**
  Packet `447` proved `PLAIN` is fast but expensive on churn-heavy rows, but
  that is not enough data to pick a default. ADR-044 now owns the remaining
  decision surface and requires the must-measure cells before task 16 can call
  the `ecvector` storage-policy question closed:
  - `EXTERNAL` (`attstorage = 'x'`) serious-lane + WAL/HOT cell
  - `MAIN` sanity cell
  - `PLAIN + fillfactor` sweep (`70 / 80 / 90`)
  - decomposition / alternative-implementation follow-ups (`detoast` vs
    `decompress`, larger touched-column update probe, C1 index-side cold-page
    sketch)
  ADR-043 now explicitly defers the storage-policy default to ADR-044 instead
  of overstating packet `447` as the final answer.

## Landing checklist

Task 16's formal subtasks (1–7 above) are all closed. Before task 16 can
**merge**, the following items still need to land. Items are independent
unless called out.

### Measurement

- [ ] **Rerun packet `440` at q200 ×≥2.** One run per side established
  `-4.33%` for persisted `source_raw` vs `source`. Rerun to confirm the
  direction survives restart noise. Cheap; unblocks treating the supported
  path as a validated runtime win rather than a single-cell inference.
- [x] **Rerun the inline serious-lane hypothesis at q200 ×≥2 on the
  productized `ecvector` surface.** Packet `446` supersedes the old
  `bytea`/duplicate-column seam with two q200 runs each on inline
  `ecvector`: `turboquant` at `3.427ms` and `3.195ms`, `pq_fastscan` at
  `2.987ms` and `2.954ms`. That is stronger evidence than a third run on the
  obsolete `tqhnsw_real_50k_tq_mixed_inline_corpus` seam because it lands on
  the actual row type.
- [x] **Head-to-head vs PqFastScan on the same inline surface.** Task 16's
  stated outcome goal is "narrow the TurboQuant vs PqFastScan latency gap
  on the 50k warm real seam". No packet in the 422–441 arc has put
  TurboQuant-with-inline-source next to PqFastScan on the same corpus,
  recall target, and runtime. Packet `446` closes that on inline
  `ecvector`: `pq_fastscan` `2.954ms` vs `turboquant` `3.195ms` on the
  confirming q200 runs, with recall `0.9635` vs `0.9629`.
- [ ] **ef_search matrix for lever-4 `full_lut` on the quantized lane.**
  Packet `437` showed `-16%` at `ef_search = 128` on one cell. Before
  lever 4 becomes any kind of persisted default, run `ef_search = 64 /
  128 / 256` so the decision rests on a shape, not a point.
- [x] **Mixed-inline storage cost measured as an explicit tradeoff.**
  Packet `441` showed heap footprint grew from `43MB` to `390MB`
  (≈9×) for the `-35%` latency win. Quantify what this means for
  buffer-cache pressure, vacuum cost per page, WAL on updates, and
  index build time — all named as a tradeoff in the readout, not just
  a win. Closed in packet `447`.
- [ ] **ADR-044 storage-policy matrix measured.** Packet `447` is enough to
  prove "`PLAIN` is fast and costly", but not enough to choose a default.
  Before task 16 closes the `ecvector` storage-policy question, land:
  - the `EXTERNAL` q200 serious-lane + WAL/HOT cell
  - the `MAIN` sanity cell
  - the `PLAIN + fillfactor` sweep (`70 / 80 / 90`)
  - the larger touched-column update probe
  - the detoast-vs-decompress read-path decomposition if practical
  - the C1 index-side cold-page rerank-payload design sketch from ADR-044

### Productization

- [ ] **ADR-043 ratified.** Status PROPOSED → ACCEPTED. Open-questions
  resolution:
  - Name: **RESOLVED — `ecvector`** (Ecaz).
  - pgvector cast policy: lean install-time conditional.
  - Bare-typmod support: tentative yes.
- [x] **`ecvector` column type landed.** Packet `442` lands the canonical
  `ecvector` row model; packet `443` narrows `tqvector` to the
  TurboQuant-family sibling artifact.
- [x] **Task 16's head-to-head measurement uses `ecvector`, not the
  bytea+`STORAGE PLAIN` recipe.** Packet `446` lands the closure
  head-to-head on default and inline `ecvector` corpora inside a fresh
  current-head scratch DB.
- [x] **Document `ecvector` as THE canonical column type** in
  README/quickstart. Packet `445` updates the root README quick start and
  storage-format examples so they create `ecvector` row columns and treat
  `tqvector` only as an artifact/debugging type.

### Quant fields (sibling artifact type contract)

Packet `443` narrowed `tqvector` to a family-specific sibling artifact.
Before task 16 merges, the contract around that name needs to be
nailed down so `tqvector` does not silently drift back into the
canonical-row position.

- [x] **Audit default-resolution paths** in `src/am/build.rs`,
  `src/am/insert.rs`, `src/am/scan.rs`, `src/am/vacuum.rs` to confirm
  that when the indexed column is canonical `ecvector`, no fallback
  ever resolves to a `tqvector` sibling. Sibling access must be
  reachable only via explicit fixture / configuration. Packet `445`
  confirms the default-resolution behavior and the runtime explanation
  surface. This is the ADR-043 §Validation "sibling-type containment"
  contract.
- [x] **pg_test: canonical vs sibling separation.** Add a regression
  test that builds a tqhnsw index on an `ecvector` column in a table
  that *also* has a `tqvector` column, and asserts the scan/rerank
  path reads the `ecvector` column, not the sibling. Packet `445`
  lands `test_pq_fastscan_indexed_ecvector_ignores_tqvector_sibling`
  to lock in that a table carrying both does not silently resolve to
  the artifact.
- [x] **pg_test: encoder round-trip for `tqvector` artifact.**
  `encode_to_tqvector(...) → tqvector column → decode` must preserve
  the expected bytes for the current TurboQuant wire format. Packet
  `445` lands `test_encode_to_tqvector_round_trips_canonical_artifact_layout`.
- [x] **Test-surface audit: no accidental `ecqvector` leftovers.**
  Packet `443` review focus §1 asks this explicitly. Grep `src/`,
  `sql/`, `tests/`, and `scripts/` for `ecqvector`; none should
  remain in runtime or user-facing paths (only in the packet `442`
  and `443` request files, plus this plan's historical notes, which are
  intentional artifacts).
- [x] **Documentation: sibling-type rule written down.** The
  "family-specific persisted quantized artifacts use family-specific
  sibling names" taxonomy rule is captured in ADR-043 §Quantized
  sibling artifacts, and packet `445` adds a README pointer so the
  canonical-vs-sibling rule is visible from the repo front door.
- [ ] **Rename public-facing error text and doc references.** Packet
  `443` updated error-text in `src/am/{build,scan,insert,vacuum}.rs`
  to describe the sibling as `tqvector`. Before merge, scan remaining
  docs (`spec/tests.md`, plan files, review-area READMEs) for stale
  `ecqvector` / old-canonical-`tqvector` wording and update. Do not
  preserve the old names as aliases.
- [x] **Compact `tqvector` into the shared efficient-storage family.**
  Packet `445` removes per-row `bits` and `seed` bytes and keeps
  `tqvector` as a canonical TurboQuant-family artifact with
  `bits=4`/`seed=42` enforced at the type surface. Current per-row
  payload is `dim + gamma + code bytes`, shrinking the artifact row
  overhead from 15 raw bytes (`dim + bits + seed + gamma`) to 6 raw
  bytes (`dim + gamma`) before the packed codes. The earlier
  typmod-only 8-byte target was not viable on current head because the
  `tqvector` output/operator functions do not receive typmod; keeping
  `dim` inline is the deliberate compact compromise.

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
- [x] **Plan file outcome section updated** after each rerun /
  head-to-head cell lands. Packet `447` and the ADR-044 follow-up now keep
  the outcome section aligned with measured truth instead of the older
  packet-`441` snapshot.
- [x] **Script-surface test for `ALTER INDEX … SET/RESET
  (rerank_source_column)`.** Packet `440`'s methodology relies on the
  ALTER cycle round-tripping; lock this in with a `pg_test` so a future
  refactor cannot silently break the measurement reproducibility. Packet
  `448` lands `test_turboquant_rerank_source_reloption_reset_round_trip`.
- [x] **`install_adr030_pg17_pg_test.sh` → backend `.so` version
  assertion.** Packet `440` caught a stale-install hazard manually
  (flat `~26.96ms` tipped it off). A script-level version check would
  convert that manual diagnosis into an automatic safety belt. Packet
  `450` makes the install script compare the installed backend module
  against the just-built release artifact and fail fast on mismatch.

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
