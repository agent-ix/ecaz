# Review Request: pgrx ambuild + Grouped-PQ4 Quantizer Wiring (Slice B of Phase 5C-3)

Branch: `adr034-diskann-rebased`
Author: coder-2
Target: `src/am/ec_diskann/ambuild.rs` (new),
`src/am/ec_diskann/{mod.rs,routine.rs,reader.rs}`
Commit: `9fb1930`

## What this packet is

Slice B closes Phase 5C-3 by wiring the Vamana build pipeline to
PostgreSQL's index AM surface:

- `ec_diskann_ambuild` / `ec_diskann_ambuildempty` replace the
  "not yet implemented (task 17 phase 4)" stubs in `routine.rs`.
- `BuildState` collects `(heap_tid, source_vector)` rows during
  `table_index_build_scan`, enforcing dimension consistency.
- `flush_build_state` trains the grouped-PQ4 + SRHT quantizer stack
  via `am::common::training::{train_grouped_pq4_model,
  derive_grouped_pq4_code, derive_persisted_binary_words,
  SrhtForwardTransform, persisted_binary_sidecar_word_count}`,
  derives per-row search codes and the optional binary sidecar
  words, then calls `build::build_and_persist_vamana` with an
  fp32 source-IP distance closure.
- Slice A's `stage_grouped_codebook_chain` is invoked after graph
  persistence; the returned head TID is patched into
  `metadata.grouped_codebook_head`.
- Metadata + `DataPageChain` are written under a single
  `GenericXLog` per buffer, matching the ec_hnsw ambuild wire
  discipline.
- Input guards: `validate_single_ecvector_attribute` rejects
  expression / partial / multi-column / non-ecvector indexes
  before any scan runs.

Also included: a drive-by clippy fix in `reader::first_live_tid`
(`for`-loop-with-early-return → `.next().map_or(Ok(None), ...)`),
surfaced by the final clippy sweep on this slice.

## Why this

Phase 5C-3 is the seam where a pure-Rust Vamana builder
(`build::build_and_persist_vamana`) meets PostgreSQL's heap scan.
The design doc (`plan/design/diskann-build-algorithm.md`)
prescribes fp32 source IP as the build distance: we have source
vectors in hand during the scan, and graph quality at build time
is more important than mirroring the scan-time quantized metric
(grouped-PQ4 IP is used at scan time only). That choice is
reflected in `source_inner_product_distance(a, b)` — a thin
`max(0, -ip)` wrapper — rather than routing through the
quantizer scorer.

Quantizer constants (`PQ_FASTSCAN_TARGET_GROUP_SIZE = 16`,
`PQ_FASTSCAN_DEFAULT_MAX_TRAIN_SIZE = 1024`,
`PQ_FASTSCAN_DEFAULT_KMEANS_ITERS = 8`) match ec_hnsw's
native-build lane so the two AMs stay comparable on quantized
recall.

## Tests

- Existing `ec_diskann` algorithmic suite (100 tests) still green
  — Slice B adds no new unit tests of its own because the pgrx
  callbacks require a running backend. End-to-end coverage lands
  in Phase 6B (scan smoke) and the recall harness task.
- Negative-path guards in `validate_single_ecvector_attribute`
  are exercised implicitly by CREATE INDEX once Phase 6B lets
  the planner select the AM.

## Verification

```
cargo build --lib                    # clean
cargo clippy --lib --no-deps         # clean (was: 1 error + 8 pre-existing sort_by warnings)
cargo test --lib ec_diskann          # 100 passed
```

## Non-changes (affirming choices)

- **No cold-rerank payload.** `VamanaMetadataPage::empty` leaves
  `PAYLOAD_FLAG_COLD_RERANK_PAYLOAD` clear (ADR-046 frozen rule 1,
  ADR-047 frozen rule 4, packet 11018). V0 reranks from the heap
  `ecvector` row.
- **Metadata written twice.** Once as an empty skeleton before the
  heap scan (so block 0 exists for the reader), once with the
  final populated state after data pages are written. This mirrors
  ec_hnsw and keeps the "block 0 always has a decodable metadata
  page" invariant.
- **No shared source-extraction module.** `ecvector_datum_to_vec`
  is inlined here rather than promoting `ec_hnsw/source.rs` into a
  shared module. Promotion is an invasive refactor touching the
  foreign native-build lane (see
  `memory/project_native_build_conflict_surface.md`); defer until
  a third AM actually needs it.
- **`disable_cost` cost-model stub unchanged.** Phase 9 replaces
  it once scan (Phase 6B) is real; until then the planner must
  not pick `ec_diskann` on its own.
- **`aminsert` still panics.** Phase 7 (ADR-046 insert lock)
  wires that in. Until then a non-empty index can be built, but
  not extended by DML.

## Dependencies

- **Packet 11014** (ADR-045 fixed-length discipline) — metadata
  overwrite path respects it.
- **Packet 11015** (Phase 5A Vamana algorithm core),
  **Packet 11017** (Phase 5C-1 persist sequencer),
  **Packet 11018** (Phase 5C-2 build orchestrator) — the build
  primitives this slice calls into.
- **Packet 11029** (Slice A codebook chain staging) — the shard
  persistence helper this slice consumes.
- **ADR-046 / ADR-047** frozen rules (packets 11024 / 11025) —
  V0 flag discipline inherited here.

## Not doing in this packet

- **Scan wiring.** Phase 6B replaces the `ambeginscan`/`amrescan`/
  `amgettuple`/`amendscan` stubs.
- **Insert (`aminsert`).** Phase 7 per ADR-046.
- **Vacuum.** Phase 8B per ADR-047.
- **Cost model.** Phase 9 replaces the `disable_cost` shim.
- **Bulk source-vector streaming.** Full collection into a `Vec`
  is acceptable for V0 since ambuild runs under
  `maintenance_work_mem`; a spill-to-disk builder is a follow-on
  optimisation task.
