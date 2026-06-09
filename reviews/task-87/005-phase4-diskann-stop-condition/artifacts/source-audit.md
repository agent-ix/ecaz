# DiskANN Task 87 Source Audit

Task 87 Phase 4 is a Stop Condition in this branch because the current
DiskANN search-code surface does not expose a TurboQuant no-QJL 4-bit
prefilter/scoring branch to route through `CandidateBatch`.

## Current Build Codec Surface

`src/am/ec_diskann/quantizer.rs` defines the build-time codec enum with
only these variants:

- `DiskannBuildCodec::PqFastScan { model: GroupedPq4Model }`
- `DiskannBuildCodec::RaBitQ { quantizer: Arc<RaBitQQuantizer> }`

`DiskannBuildCodec::prepare` accepts only the current DiskANN storage
formats:

- `StorageFormat::PqFastScan`
- `StorageFormat::RaBitQ`

`DiskannBuildCodec::search_codec_kind` maps those variants to the only
persisted search-code discriminators this path currently emits:

- `VAMANA_SEARCH_CODEC_GROUPED_PQ`
- `VAMANA_SEARCH_CODEC_RABITQ`

There is no `TurboQuant` build-code variant and no
`VAMANA_SEARCH_CODEC_TURBOQUANT` equivalent in this branch.

## Current Search Prefilter Surface

`src/am/ec_diskann/quantizer.rs` defines the runtime prefilter enum with
only these scoring branches:

- `DiskannPreparedPrefilter::BinarySidecar`
- `DiskannPreparedPrefilter::GroupedPq`
- `DiskannPreparedPrefilter::RaBitQ`

`DiskannPreparedPrefilter::score` dispatches those branches to:

- binary-sidecar Hamming/Jaccard scoring;
- grouped-PQ LUT scoring;
- RaBitQ scalar IP estimation.

There is no branch that prepares a `PreparedLutNoQjl4BitQuery` or calls
`ProdQuantizer::score_ip_from_parts_lut_no_qjl_4bit` for DiskANN search
codes.

## Task 90 Ownership

`plan/tasks/90-diskann-turboquant-search-codec.md` is the follow-up task
for deciding and, if feasible, landing a narrow, on-disk-format-neutral
DiskANN TurboQuant search codec. It also states that grouped-PQ and
RaBitQ must not be reinterpreted as satisfying Task 87's TurboQuant
no-QJL 4-bit gate.

## Stop Condition

Task 87 should not add a fake DiskANN `CandidateBatch` route over
grouped-PQ or RaBitQ and count it as the TurboQuant no-QJL Phase 4
slice. Until Task 90 lands an accepted TurboQuant DiskANN search-code
surface, Phase 4 remains deferred by this Stop Condition.
