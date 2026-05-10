# Request

Review the DiskANN quantization/storage optimization research summary and recommended next experiments.

This packet does not land a new scan algorithm. It turns the M5 cross-engine measurements and pgvectorscale comparison into a concrete implementation decision surface so the next DiskANN work is empirical and repeatable rather than ad hoc.

## Evidence Base

Packet `30546` added the repeatable suite and produced the raw numbers copied into this packet's `artifacts/` directory:

- [artifacts/manifest.md](/Users/peter/dev/tqvector/review/30547-diskann-quant-storage-optimization-research/artifacts/manifest.md)
- [results.jsonl](/Users/peter/dev/tqvector/review/30547-diskann-quant-storage-optimization-research/artifacts/results.jsonl)
- [compare-vectorscale-binary-real10k.log](/Users/peter/dev/tqvector/review/30547-diskann-quant-storage-optimization-research/artifacts/compare-vectorscale-binary-real10k.log)
- [compare-vectorscale-grouped-real10k.log](/Users/peter/dev/tqvector/review/30547-diskann-quant-storage-optimization-research/artifacts/compare-vectorscale-grouped-real10k.log)
- [storage-diskann-prefilter-real10k.log](/Users/peter/dev/tqvector/review/30547-diskann-quant-storage-optimization-research/artifacts/storage-diskann-prefilter-real10k.log)

Prior design context:

- `review/11095-task29-diskann-pgvectorscale-comparison/prefilter-detail.md`
- `spec/adr/ADR-030-fastscan-grouped-subvector-scoring.md`
- `spec/adr/ADR-031-rabitq-binary-prefilter.md`
- `spec/adr/ADR-044-ecvector-rerank-source-location-and-storage-policy.md`
- `spec/functional/FR-035-diskann-scan-prefilter-rerank.md`

## Findings

### 1. The current recall gap is mostly closed by the implemented binary sidecar

At matched sweep widths, the implemented `binary_sidecar` prefilter is already recall-competitive with pgvectorscale on real10k:

| Sweep | ec_diskann binary recall | ec_diskann binary p50 | pgvectorscale recall | pgvectorscale p50 |
|---:|---:|---:|---:|---:|
| 64 | 0.9965 | 2.15 ms | 0.9955 | 0.59 ms |
| 200 | 0.9990 | 4.63 ms | 1.0000 | 1.14 ms |
| 800 | 1.0000 | 15.4 ms | 1.0000 | 3.76 ms |

This changes the optimization target. The biggest remaining issue is not "find any prefilter that preserves recall"; it is "make the matched-width scan/rerank path several times cheaper."

### 2. Grouped-PQ is not a good first-stage DiskANN traversal prefilter

The grouped-PQ path has similar latency to binary sidecar but much lower recall at small and mid widths:

| Sweep | ec_diskann grouped recall | ec_diskann grouped p50 | pgvectorscale recall | pgvectorscale p50 |
|---:|---:|---:|---:|---:|
| 64 | 0.9320 | 2.14 ms | 0.9955 | 0.60 ms |
| 200 | 0.9850 | 4.56 ms | 1.0000 | 1.13 ms |
| 800 | 0.9990 | 15.2 ms | 1.0000 | 3.73 ms |

Grouped-PQ should stay available as a fallback and may still be useful as a later second-stage/refinement payload, but it should not drive near-term DiskANN traversal work.

### 3. Our binary sidecar is not pgvectorscale SBQ

The implemented sidecar is SRHT sign bits. pgvectorscale's SBQ stores source-coordinate bits thresholded by trained per-dimension means. Both are 1-bit-per-dimension Hamming/XOR traversal codes at 1536 dimensions, but the semantics differ:

| Property | ec_diskann binary sidecar | pgvectorscale SBQ |
|---|---|---|
| Per-node code at 1536d | 192 B | 192 B |
| Threshold | zero sign after SRHT rotation | per-dimension training mean |
| Query transform | SRHT then sign pack | source-domain mean threshold |
| Metadata needed | current quantizer/SRHT seed | per-dimension mean/threshold vector |
| Runtime scorer | XOR + popcount | XOR + popcount |
| Drop-in switch? | already implemented | no; requires persisted payload semantics and metadata |

True SBQ is therefore an on-disk format addition, not a GUC-only experiment. The benchmark result says our SRHT sign sidecar is high-quality enough to be viable, but it does not prove SBQ is unnecessary.

### 4. The speed gap is unlikely to be fixed by swapping grouped-PQ for SBQ alone

At matched widths, pgvectorscale is roughly 3.6x to 4.1x faster while doing comparable or better recall. Since both binary sidecar and SBQ have the same 192 B code size and the same XOR/popcount class of scorer, the remaining delta is more likely in these areas:

- pgvectorscale streams results and exact-rescore candidates through its DiskANN scan iterator, while our matched-width path still pays heavy final heap rerank/executor costs.
- Our final rerank source is heap-backed `ecvector`, so each retained candidate can pay heap fetch, varlena/detoast, decode, and exact dot-product cost.
- Our scan loop carries Postgres access-method and candidate materialization overhead that now dominates once the traversal scorer is cheap.
- pgvectorscale's SBQ build/search implementation may also have lower constant factors in graph traversal and candidate queue management, but the equal-size/equal-scorer shape makes rerank/source locality the higher-confidence target.

### 5. Index size is not currently the blocker

For the real10k lane, the `ec_diskann` index is `4.7 MiB` / `494.0 B/row`; the pgvectorscale DiskANN index is `5,136,384 bytes`. These are close enough that near-term performance work should not optimize by shrinking the existing hot tuple first. The better question is whether to spend more index bytes to avoid heap rerank cost.

## Option Matrix

| Option | Expected benefit | Cost / risk | Recommendation |
|---|---|---|---|
| Keep current SRHT binary sidecar as default | Already matches pgvectorscale recall at useful widths | Does not close 4x p50 gap | Keep as default traversal prefilter |
| Add true SBQ payload/metadata | Tests whether mean-threshold bits beat SRHT sign bits | Requires new persisted format, insert/build metadata, query prep, compatibility guard | Worth a bounded experiment, not first speed lever |
| Add index-side exact/rerank payload | Avoid heap fetch/detoast/decode during final rerank | Larger index; write-path and WAL tradeoff; needs format versioning | Highest-value speed lever |
| Add compact rerank payload, e.g. f16 or int8 | Avoid heap fetch with less index bloat than f32 | Approximate final rerank may perturb exact ordering unless final heap check remains | Good second lever after exact payload seam |
| Port RaBitQ q=1 into DiskANN payload | Tests existing quantizer/scorer in graph traversal | Current code is 204 B at 1536d due scalar tail and has heavier estimator than Hamming-only sidecar | Lower priority; measure only after source-locality split |
| Use grouped-PQ as second-stage refiner | Could shrink exact rerank survivor set | Adds more stages and tuning; grouped first-stage result is weak | Defer until heap-rerank cost split is measured |
| TurboQuant as DiskANN traversal payload | Reuses implemented quantizer | Larger/slower than PQFastScan class for this purpose | Do not prioritize for traversal |

## Recommended Next Work

1. Add a repeatable DiskANN cost-split profile that reports traversal time, heap fetch/decode time, and exact dot-product time separately at `list_size/rerank_budget = 64, 200, 800`.
2. Implement a format-gated index-side rerank-source experiment, ideally raw f32 first for a clean ceiling measurement, then f16/int8 only if raw f32 proves the gap is source-locality dominated.
3. Add a true SBQ build/scan format only after the cost split confirms traversal code quality still matters enough to justify a new binary-code semantic.
4. Keep RaBitQ as a later bounded test, using the existing `src/quant/rabitq.rs` implementation, but do not assume it will beat the existing sidecar because q=1 code length is larger and the estimator path is heavier than XOR-only scoring.
5. Keep all cells in `ecaz bench suite` profiles with packet-local logs, matching the repeatability standard from `30546`.

## Review Questions

- Is the prioritization correct that source-local rerank should precede true SBQ?
- Is raw f32 index-side rerank the right first ceiling experiment, or should the first persisted payload be f16/int8 to avoid knowingly large indexes?
- Should true SBQ be represented as a new `ec_diskann.storage_format` value, a sidecar-kind reloption, or a full DiskANN payload format version?
