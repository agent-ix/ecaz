# Task 91 Packet 001 Artifact Manifest

- head SHA: `5cdcf38a529ddec50665a4ea44b806f03383897f`
- task bucket: `reviews/task-91/`
- packet path: `reviews/task-91/001-phase1-trait-audit/`
- timestamp: `2026-06-09T02:51:39Z`
- scope: design-only trait audit for cross-AM `QuantCodec` migration
- lane / fixture / storage format / rerank mode: not applicable; no benchmark or pg_test run
- isolated one-index-per-table vs shared-table surfaces: not applicable; docs-only packet

## Artifacts

### `trait-audit.md`

- command context:
  - `sed -n '1,240p' plan/tasks/91-cross-am-quantcodec-migration.md`
  - `sed -n '1,220p' src/am/common/quant_codec.rs`
  - `sed -n '1,260p' src/am/common/candidate_batch.rs`
  - `rg -n "QuantCodec|score_ip_batch|CandidateMeta|HnswStorageCodec|TurboQuantExactScoreMode|DiskannBuildCodec|DiskannPreparedPrefilter|SpirePreparedAssignmentScorer" src/am src/quant`
- result: design audit produced; no executable validation expected
- key cited decisions:
  - keep `QuantCodec::score_ip_batch` as Task 91/92 universal batch entry point;
  - use enum dispatch at AM boundaries, not hot-loop `dyn QuantCodec`;
  - keep residual signs in `CandidateMeta::GammaAndResidualSigns`;
  - grow model binding for grouped-PQ before migrating HNSW/DiskANN;
  - rename storage-binding adapters so "codec" refers to `QuantCodec`.

### `dispatch-contract.md`

- command context:
  - `sed -n '180,630p' src/am/ec_ivf/quantizer.rs`
  - `sed -n '1,620p' src/am/ec_spire/quantizer/mod.rs`
  - `sed -n '1,520p' src/am/ec_diskann/quantizer.rs`
  - `sed -n '5140,5205p' src/am/ec_hnsw/scan.rs`
- result: Phase 2 implementation contract added
- key cited decisions:
  - grouped-PQ model bytes bind into the codec/prepared-query adapter before
    query preparation;
  - prepared-query enums stay concrete but delegate quant scoring through
    `QuantCodec`;
  - counters increment only after successful shape validation and scoring;
  - IVF Phase 2 evidence must cover model-bound grouped-PQ construction.

## Validation

No tests run. Task 91 Phase 1 is explicitly design-only and has no Rust code
changes in this packet.
