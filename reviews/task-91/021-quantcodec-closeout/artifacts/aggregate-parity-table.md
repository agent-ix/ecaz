# Task 91 Aggregate Parity Table

Task 91 is a refactor/migration task. Its closeout evidence is the reviewed
set of per-AM parity tests, source audits, storage-binding rename packets, and
ADR/task-file status checks.

## AM x Quant Matrix

| AM | Quant / Format | QuantCodec surface | Primary evidence | Result |
|---|---|---|---|---|
| IVF | TurboQuant | `IvfQuantizer` / `IvfQuantCodec<'_>` | `reviews/task-91/002-ivf-trait-growth-retouch/` | `common_quant_codec_turboquant_batch_is_bit_exact_with_scalar` passed with `f32::to_bits()` parity. |
| IVF | TurboQuant no-QJL LUT32 | `IvfQuantizer` / `IvfQuantCodec<'_>` | `reviews/task-91/002-ivf-trait-growth-retouch/` | `common_quant_codec_turboquant_no_qjl_lut32_batch_is_bit_exact_with_scalar` passed with `f32::to_bits()` parity. |
| IVF | RaBitQ | `IvfQuantizer` / `IvfQuantCodec<'_>` | `reviews/task-91/002-ivf-trait-growth-retouch/`, `reviews/task-91/016-quantcodec-cutoff-scoring/` | Batch parity passed; cutoff-capable `try_score_ip_candidate` hook covered by focused IVF cutoff test. |
| IVF | grouped-PQ / PqFastScan | model-bound `IvfQuantCodec<'_>` | `reviews/task-91/002-ivf-trait-growth-retouch/`, `reviews/task-91/005-pq-fastscan-parity-followup/`, `reviews/task-91/016-quantcodec-cutoff-scoring/` | Model-bound batch parity passed; PqFastScan batch-vs-direct `to_bits()` parity passed; grouped-PQ cutoff hook passed. |
| SPIRE | TurboQuant assignment payloads | `SpireAssignmentQuantCodec` | `reviews/task-91/003-spire-quantcodec-follow-through/`, `reviews/task-91/004-spire-scan-quantcodec-dispatch/`, `reviews/task-91/013-spire-selected-row-quantcodec/`, `reviews/task-91/014-spire-column-batch-quantcodec/`, `reviews/task-91/015-spire-sampled-cutoff-quantcodec/` | Assignment batch, V2 leaf-column batch, selected-row, column batch, and sampled fallback routes covered by focused bit-exact tests. |
| SPIRE | RaBitQ assignment payloads | `SpireAssignmentQuantCodec` | `reviews/task-91/006-spire-closeout-audit/`, `reviews/task-91/013-spire-selected-row-quantcodec/`, `reviews/task-91/014-spire-column-batch-quantcodec/`, `reviews/task-91/015-spire-sampled-cutoff-quantcodec/`, `reviews/task-91/016-quantcodec-cutoff-scoring/` | No-cutoff routes migrated through `QuantCodec`; final bounded cutoff route migrated through `try_score_ip_candidate`; source audit shows `src/am/ec_spire/scan/candidates.rs` has no direct scorer helper references. |
| HNSW | TurboQuant exact modes | `HnswTurboQuantScanCodec<'_>` | `reviews/task-91/008-hnsw-turboquant-scan-quantcodec/` | Exact, full-LUT, tiled-LUT, and int8 prepared-query variants route through the codec with bit-level score parity. |
| HNSW | grouped-PQ / PqFastScan | `HnswGroupedPqScanCodec` | `reviews/task-91/009-hnsw-grouped-rabitq-quantcodec/` | Grouped-PQ search-code scoring routes through `QuantCodec`; tag carries group size matching IVF/DiskANN semantics. |
| HNSW | RaBitQ | `HnswRaBitQScanCodec` | `reviews/task-91/009-hnsw-grouped-rabitq-quantcodec/` | RaBitQ search-code scoring routes through `QuantCodec`; traversal score polarity preserved outside the codec. |
| DiskANN | Binary sidecar / Hamming | `DiskannBinarySidecarPrefilterCodec` | `reviews/task-91/010-diskann-prefilter-quantcodec/` | Binary-sidecar prefilter scoring routes through `QuantCodec`; DiskANN distance-score polarity remains at prefilter layer. |
| DiskANN | grouped-PQ | `DiskannGroupedPqPrefilterCodec` | `reviews/task-91/010-diskann-prefilter-quantcodec/`, `reviews/task-91/012-diskann-turboquant-search-codec/` | Grouped-PQ prefilter scoring routes through `QuantCodec`; SQL storage-format smoke includes `pq_fastscan`. |
| DiskANN | RaBitQ | `DiskannRaBitQPrefilterCodec` | `reviews/task-91/010-diskann-prefilter-quantcodec/`, `reviews/task-91/012-diskann-turboquant-search-codec/` | RaBitQ prefilter scoring routes through `QuantCodec`; SQL storage-format smoke includes `rabitq`. |
| DiskANN | TurboQuant direct search code | `DiskannTurboQuantPrefilterCodec<'_>` | `reviews/task-91/012-diskann-turboquant-search-codec/`, `reviews/task-91/019-careful-turboquant-mirror/` | Task 90 absorbed: metadata discriminator, build/insert encode, prefilter scorer, SQL smoke, and careful hardening mirror all landed. |

## Acceptance Criteria Mapping

| Task 91 acceptance criterion | Evidence | Closeout status |
|---|---|---|
| One `QuantCodec` trait and every quant x AM scoring path routed through it | `artifacts/quantcodec-impl-audit.log`; packets 002-016; reviewer architectural review in `reviews/task-91/018-adr071-072-acceptance/feedback/2026-06-09-02-reviewer.md` | Satisfied. |
| HNSW storage binding renamed and quantizer-family responsibilities removed | `reviews/task-91/007-hnsw-storage-binding-rename/`; `reviews/task-91/008-hnsw-turboquant-scan-quantcodec/`; `reviews/task-91/009-hnsw-grouped-rabitq-quantcodec/` | Satisfied. |
| DiskANN exposes TurboQuant search codec through `QuantCodec` | `reviews/task-91/012-diskann-turboquant-search-codec/`; `reviews/task-91/019-careful-turboquant-mirror/` | Satisfied. |
| Task 90 closed by reference | `plan/tasks/90-diskann-turboquant-search-codec.md`; `artifacts/task-status-audit.log` | Satisfied. |
| ADR-071 accepted and ADR-072 amended | `reviews/task-91/018-adr071-072-acceptance/`; `artifacts/adr-audit.log` | Satisfied. |
| Per-AM behavioral parity | Per-cell packets above | Satisfied by focused bit-exact and SQL smoke evidence; no new broad local test run in this closeout packet. |
| No measurable latency regression | Existing packet-level tests are functional parity; reviewer architectural review classifies remaining closeout as documentation/evidence, not additional code | No new latency suite run in this closeout packet. Broader performance work remains with Task 87/92+ kernels. |
| Existing pg_test surfaces pass | DiskANN PG18 SQL smoke in packet 012; other migrated paths covered by focused unit tests | Satisfied for touched SQL-facing DiskANN format surface; no new broad local pg_test run in this closeout packet. |
