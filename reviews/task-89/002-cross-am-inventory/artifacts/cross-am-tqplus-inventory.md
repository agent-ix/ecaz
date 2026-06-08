# Task 89 Cross-AM TQ+ Implementation Inventory

Date: 2026-06-08

Head inspected: `c8eb583ca`

## Summary

Task 89's all-index requirement is larger than simply cherry-picking the Task
86 IVF TQ+ commits. Current `main` has these TurboQuant surfaces:

| AM | Current TurboQuant support | Current profile hook | TQ+ implication |
| --- | --- | --- | --- |
| IVF | `storage_format = 'turboquant'` / `auto` maps to `IvfQuantizerProfile::TurboQuant` | none | Add `turboquant_profile`, shared TQ+ calibration math, IVF metadata/calibration persistence, scan/insert routing. |
| SPIRE | `storage_format = 'turboquant'` maps to `SpireAssignmentPayloadFormat::TurboQuant` | none | Add `turboquant_profile`, persist calibration in partition/local-store metadata or AM-owned calibration record, route assignment encoding/scoring through TQ+. |
| HNSW | `storage_format = 'turboquant'` via `quant::Family::TurboQuant`; metadata has V1 scalar and V3 hot/cold layouts | none | Add profile reloption/metadata, likely new metadata version or flag, graph build/insert/scan scoring paths with scan-local calibration load. |
| DiskANN | no `turboquant` storage format; only `pq_fastscan` and `rabitq` | none | First add baseline DiskANN TurboQuant codec/search codec, then add TQ+ as a profile. This is a true baseline gap, not just a TQ+ toggle. |

## Shared Math Starting Point

The reverted Task 86 commits are still the best source for the TQ+ math:

- `e0ae9fe7d` adds IVF TQ+ encode/query/scoring paths.
- `c7e85e8ac` hardens calibration validation and the task-local format plan.
- `55e492899` adds IVF page coverage for the TQ+ delta gate.

Do not re-land the old public shape verbatim. ADR-076 chooses
`storage_format = 'turboquant', turboquant_profile = 'tqplus'`, not a new
preferred `storage_format = 'turboquant_tqplus'`.

The shared code should move toward `src/quant/prod.rs` owning:

- a validated calibration model (`shift`, `scale`);
- deterministic training from sampled vectors;
- source-vector encoding through the calibrated domain;
- query preparation that folds calibration into the LUT and bias;
- no-QJL 4-bit lane validation.

AMs should own only persistence and traversal binding.

## IVF Touch Points

- `src/am/ec_ivf/options.rs`
  - Add `turboquant_profile` string reloption.
  - Parse `standard | tqplus`, default `standard`.
  - Keep `quantizer` alias semantics for storage format only.
- `src/am/ec_ivf/quantizer.rs`
  - Extend `IvfQuantizerProfile` or add a separate TurboQuant profile enum.
  - Route TQ+ encode/query/score using shared `prod` calibration helpers.
- `src/am/ec_ivf/page.rs`
  - Persist profile/calibration presence without reusing public `pq_codebook`
    semantics for new production writes.
  - Preserve read-only legacy support for Task 86 tag 4 only if needed.
- `src/am/ec_ivf/build.rs`, `insert.rs`, `scan.rs`
  - Train calibration after source sampling.
  - Load calibration once per scan/insert operation.
  - Validate unsupported lanes before build/query/insert.

## SPIRE Touch Points

- `src/am/ec_spire/options/mod.rs`
  - Add `turboquant_profile` reloption alongside `storage_format` / `quantizer`.
  - Keep `SpireStorageFormat::assignment_payload_format()` returning
    `TurboQuant`; profile is an additional discriminator.
- `src/am/ec_spire/quantizer/mod.rs`
  - Add profile-aware assignment payload encoding and prepared scorer selection.
- `src/am/ec_spire/storage/*`
  - Decide the AM-owned calibration record/metadata location.
  - Avoid per-candidate calibration fetch; load per scan/local-store context.
- `src/am/ec_spire/build.rs`, `insert.rs`, `scan/*`
  - Thread profile and calibration through build assignment creation, live
    inserts, and candidate scoring.

## HNSW Touch Points

- `src/am/ec_hnsw/options.rs`
  - Add `turboquant_profile` reloption.
- `src/am/ec_hnsw/page.rs`
  - Add metadata evidence that TQ+ calibration is present. Likely a new
    metadata version or explicit flag per ADR-070 rather than silently treating
    V1/V3 TurboQuant bytes as TQ+.
- `src/am/ec_hnsw/graph.rs`, `codec.rs`
  - Extend graph storage descriptor/codec validation to distinguish standard
    TurboQuant from TQ+ where needed.
- `src/am/ec_hnsw/build.rs`, `insert.rs`, `scan.rs`, `shared.rs`, `vacuum.rs`
  - Build/insert encode with TQ+ calibration.
  - Load calibration once per scan opaque, not during every graph expansion.
  - Keep exact/rerank source behavior unchanged except for the approximate
    traversal score path.

## DiskANN Touch Points

DiskANN currently has no TurboQuant storage format:

- `src/am/ec_diskann/options.rs::StorageFormat` accepts only
  `pq_fastscan | rabitq`.
- `src/am/ec_diskann/page.rs` accepts only
  `VAMANA_SEARCH_CODEC_GROUPED_PQ | VAMANA_SEARCH_CODEC_RABITQ`.
- `src/am/ec_diskann/quantizer.rs::DiskannBuildCodec` has only
  `PqFastScan` and `RaBitQ`.

Implementation order should therefore be:

1. Add baseline `storage_format = 'turboquant'` for DiskANN with a new Vamana
   search codec kind and standard TurboQuant scoring.
2. Add `turboquant_profile = 'tqplus'` over that baseline.
3. Measure standard vs. TQ+ on the same DiskANN suite cells.

This is required for Task 89 because the acceptance criteria say all four AMs
must be tested against a TQ baseline.

## Proposed Slice Order After ADR Gate

1. Shared TQ+ math in `src/quant/prod.rs` with unit coverage.
2. Common profile reloption parsing shape, first in IVF/SPIRE where reloption
   patterns already match.
3. IVF production-profile port, using Task 86 code as source material but not
   preserving the old public API as preferred syntax.
4. SPIRE port, reusing the IVF/shared math and AM-local calibration storage.
5. HNSW port with metadata/version discipline.
6. DiskANN baseline TurboQuant, then DiskANN TQ+.
7. Suite configs and validation packets for DBPedia, second corpus, and drift.

## Risks

- DiskANN is the largest unknown because baseline TurboQuant is absent.
- HNSW and DiskANN need graph traversal calibration caching; per-candidate
  calibration loads would invalidate the latency gate.
- Any new metadata version must fail closed so old standard TurboQuant indexes
  cannot be misread as TQ+.
- The benchmark matrix is large. The code should land in narrow reviewable
  slices before the full measurement closeout.
