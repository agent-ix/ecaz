# Task 97 Packet 001 Artifact Manifest

- head SHA: `e7ce5d153470bf35bee8daa6233f27bfaff52db5`
- task bucket: `reviews/task-97/001-qjl-surface-inventory/`
- lane: coder-1 LUT lane, Task 97 TurboQuant QJL block kernel family
- timestamp: `2026-06-09T17:23:35-07:00`
- isolated one-index-per-table or shared-table surfaces: not applicable;
  static source inventory only
- tests/benches: not run; this packet has no code change and no runtime claim

## Artifacts

### `qjl-mode-rule-audit.log`

- command: `rg -n 'TILED_FWHT_COMPAT|fn tile_dim|fn qjl_enabled|fn prepared_query_uses_lut|fn mse_bits|exact_score_mode\(|exact_score_uses_qjl|exact_score_uses_lut|DEFAULT_QUANT_BITS|validate_tqvector_bits|encode_to_ecvector expects' src/quant/rotation.rs src/quant/prod.rs src/lib.rs`
- key result: QJL is disabled only for the current tiled compatibility lane
  (`dim=1536,bits=4`); public SQL encoding remains canonical 4-bit.

### `ivf-qjl-surface-audit.log`

- command: `rg -n 'IvfPreparedQuery::TurboQuant\(|TurboQuantNoQjl4BitLut|score_turboquant_no_qjl_4bit_batch_for|CandidateMeta::Gamma|score_ip_from_parts\(|score_ip_batch<Id>|score_ip_from_parts_with_min_bound' src/am/ec_ivf/quantizer.rs`
- key result: IVF selects `TurboQuantNoQjl4BitLut` for no-QJL and the generic
  `TurboQuant` prepared query for gamma-aware scoring.

### `spire-qjl-surface-audit.log`

- command: `rg -n 'SpirePreparedAssignmentScorer|no_qjl_4bit_lut|score_turboquant_no_qjl_4bit_batch_for|CandidateMeta::Gamma|CandidateMeta::GammaAndResidualSigns|score_ip_from_parts\(|score_candidate_batch_ip|score_batch_ip|DEFAULT_QUANT_BITS' src/am/ec_spire/quantizer/mod.rs src/am/ec_spire/scan/candidates.rs`
- key result: SPIRE uses the no-QJL LUT batch helper only when
  `no_qjl_4bit_lut` is present; otherwise it falls back to gamma-aware
  `score_ip_from_parts`.

### `hnsw-qjl-surface-audit.log`

- command: `rg -n 'TurboQuantExactScoreMode|resolve_turboquant_exact_score_mode|HnswTurboQuantPreparedQuery|score_and_cache_turboquant_full_lut_payload_batch|score_scan_element_result|CandidateMeta::Gamma|score_ip_from_parts\(|FullLut|TiledLut|Int8Approx|no-QJL 4-bit lane' src/am/ec_hnsw/scan.rs src/am/ec_hnsw/build.rs`
- key result: HNSW default exact mode scores through
  `HnswTurboQuantPreparedQuery::Exact` with gamma; non-default full/tiled/int8
  modes require the no-QJL 4-bit lane.

### `diskann-tq-surface-audit.log`

- command: `rg -n 'DiskannTurboQuantPrefilterCodec|PreparedLutNoQjl4BitQuery|int8_approx_no_qjl_4bit_supported|no-QJL 4-bit dimension lane|score_ip_from_parts_lut_no_qjl_4bit|mse_code_len|qjl' src/am/ec_diskann/quantizer.rs src/am/ec_diskann/insert.rs reviews/task-91/012-diskann-turboquant-search-codec`
- key result: DiskANN TurboQuant currently requires no-QJL 4-bit query prep and
  rejects QJL-active dimensions.

### `standard-fixture-dimension-audit.log`

- command: `rg -n 'dim 1536|dim=1536|--dim 1536|1536-dimensional|1536-dim|DBpedia|qdrant-dbpedia|standard 1536|non-tiled dimension|Dimension coverage' reviews/task-92 reviews/task-96 plan/tasks/12-real-corpus-recall.md plan/tasks/task30-phase13a-spire-aws-verification-design.md plan/tasks/task30-phase13b-spire-aws-verification-runbook.md plan/tasks/99-cross-am-quant-isa-block-kernel-closeout.md crates/ecaz-cli/suites`
- key result: current standard/DBpedia evidence path is 1536-dimensional, and
  Task 99 explicitly calls for at least one non-tiled dimension to make QJL
  lanes exercisable.
