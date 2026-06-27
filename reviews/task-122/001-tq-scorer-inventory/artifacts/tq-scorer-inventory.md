# TurboQuant Scorer Inventory

This inventory covers the current latest-main Task 122 branch before any
behavior-changing optimization work.

## Shared Block Scorers

| Lane | Status | Evidence |
| --- | --- | --- |
| TQ no-QJL 4-bit LUT | Block/partial width cascade | `src/am/common/candidate_batch/mod.rs:317`, `src/am/common/candidate_batch/mod.rs:543` |
| TQ QJL | Block32 plus octet8 remainder, scalar tail below octet width | `src/am/common/candidate_batch/mod.rs:331`, `src/am/common/candidate_batch/mod.rs:747` |
| TQ tiled LUT | Width cascade; exact-mode no-QJL scorer | `src/am/common/candidate_batch/mod.rs:170` |
| TQ int8 approximation | Width cascade; exact-mode no-QJL scorer | `src/am/common/candidate_batch/mod.rs:242` |

## Access Method Status

| AM / surface | Current TQ status | Notes |
| --- | --- | --- |
| IVF scan payload slab | Block path for no-QJL and QJL | `score_turboquant_batch_from_payloads` routes both lanes through shared block scorers at `src/am/ec_ivf/quantizer.rs:616`. |
| IVF borrowed payload refs | Block path for no-QJL and QJL | Used by rerank/materialization paths that have non-contiguous payloads, `src/am/ec_ivf/quantizer.rs:752`. |
| IVF scalar single payload | Scalar fallback | `score_ip_from_parts` remains single-candidate, `src/am/ec_ivf/quantizer.rs:368`. |
| IVF exact-dequant rerank | Scalar | Exact-dequant path decodes/scores per payload, `src/am/ec_ivf/rerank.rs:555` and `src/am/ec_ivf/rerank.rs:628`. |
| SPIRE assignment/routed scan | Block path for TQ when CandidateBatch scoring is enabled | `src/am/ec_spire/quantizer/mod.rs:439`. |
| SPIRE V2 leaf scan | Scores full batch before materialization | Good Phase 2 target: `src/am/ec_spire/scan/candidates.rs:2293`, `src/am/ec_spire/scan/candidates.rs:2636`, `src/am/ec_spire/scan/candidates.rs:2709`. |
| HNSW exact payload batch | Block path for full LUT, tiled LUT, int8, and QJL exact modes | `src/am/ec_hnsw/scan.rs:2452`, QJL arm at `src/am/ec_hnsw/scan.rs:2543`. |
| HNSW scan codec batch | TQ codec implements shared batch scoring for scan-code batches | `src/am/ec_hnsw/scan.rs:3306`, test call at `src/am/ec_hnsw/scan.rs:8283`. |
| HNSW single scan scoring helper | Scalar fallback | `src/am/ec_hnsw/scan.rs:5729` builds the codec and scores one candidate. |
| DiskANN prefilter | no-QJL 4-bit block path only | `src/am/ec_diskann/quantizer.rs:380`, codec batch implementation at `src/am/ec_diskann/quantizer.rs:767`. |

## Initial Priority

The highest-value immediate work is not another TQ scoring kernel. The obvious
gap is fusing scored candidates with top-k retention/materialization, starting
with SPIRE V2 leaf or IVF scan/rerank paths where the code currently scores a
full candidate surface and materializes rows after scoring.

The scorer parity gate for the first measured TQ lanes should be:

- no-QJL 4-bit: use shared block scorer only;
- QJL: use shared block scorer, record scalar tail;
- exact-dequant: exclude from latency promotion comparisons until there is a
  separate reason to optimize it;
- DiskANN: no-QJL only unless a separate task adds durable QJL/alternate TQ
  storage.
