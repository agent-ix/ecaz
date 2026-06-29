# Task 124 Packet 003 Artifact Manifest

Head SHA: `eff61e0aa` (`Optimize IVF TQ rerank sidecar payload reads`).

Task bucket: `reviews/task-124/003-tq-stage2-ab-suite/`

Lane: local Apple Silicon PG18 release build, `ec_ivf` `coarse_rerank`.

Fixture: staged `dbpedia-openai-100k` corpus family at 10k, 50k, and 100k, k=10,
100 query limit, 100 latency iterations, nprobe sweep `[32, 64]`.

Storage / rerank matrix:

- Baseline: `storage_format=coarse_rerank`, `coarse_format=rabitq`,
  `coarse_bits=1`, `rerank_placement=source`, `rerank_format=f32`,
  `rerank_width=100`.
- TQ stage-2: same RaBitQ candidate frontier, `rerank_placement=index`,
  `rerank_format=turboquant`, `rerank_width=100`,
  `stage2_final_rerank_width=25`.

## Suite Config

Artifact: `task124-tq-stage2-ab-suite.json`

Command used for initial full A/B run:

```text
./target/release/ecaz --host /Users/peter/.pgrx --port 28818 --database tqvector_bench bench suite run --config reviews/task-124/003-tq-stage2-ab-suite/artifacts/task124-tq-stage2-ab-suite.json --artifact-dir reviews/task-124/003-tq-stage2-ab-suite/artifacts --manifest-out reviews/task-124/003-tq-stage2-ab-suite/artifacts/suite-manifest.json --results-out reviews/task-124/003-tq-stage2-ab-suite/artifacts/suite-results.jsonl
```

Command used for post-optimization scan-only rerun:

```text
./target/release/ecaz --host /Users/peter/.pgrx --port 28818 --database tqvector_bench bench suite run --config reviews/task-124/003-tq-stage2-ab-suite/artifacts/task124-tq-stage2-ab-suite.json --only recall-10k-f32-w100 --only recall-10k-tq-stage2-w100-final25 --only latency-10k-f32-w100 --only latency-10k-tq-stage2-w100-final25 --only explain-10k-f32-w100-p64 --only explain-10k-tq-stage2-w100-final25-p64 --only recall-50k-f32-w100 --only recall-50k-tq-stage2-w100-final25 --only latency-50k-f32-w100 --only latency-50k-tq-stage2-w100-final25 --only explain-50k-f32-w100-p64 --only explain-50k-tq-stage2-w100-final25-p64 --only recall-100k-f32-w100 --only recall-100k-tq-stage2-w100-final25 --only latency-100k-f32-w100 --only latency-100k-tq-stage2-w100-final25 --only explain-100k-f32-w100-p64 --only explain-100k-tq-stage2-w100-final25-p64 --artifact-dir reviews/task-124/003-tq-stage2-ab-suite/artifacts --run-subdir copy-avoidance-scan-full --manifest-out reviews/task-124/003-tq-stage2-ab-suite/artifacts/copy-avoidance-scan-full-manifest.json --results-out reviews/task-124/003-tq-stage2-ab-suite/artifacts/copy-avoidance-scan-full-results.jsonl
```

## Initial Full A/B Results

Artifacts:

- `suite-results.jsonl`
- `suite-report.md`
- `suite-manifest.json`
- raw logs under `suite/`

Key result lines:

- Recall matched f32 at all measured cells:
  - 10k: f32 and TQ stage-2 both `recall@10=1.0000` at nprobe 32 and 64.
  - 50k: f32 and TQ stage-2 both `0.9960` at nprobe 32 and `1.0000` at nprobe 64.
  - 100k: f32 and TQ stage-2 both `0.9730` at nprobe 32 and `1.0000` at nprobe 64.
- Initial TQ latency was not a win:
  - 10k/nprobe64: f32 `p50=1.34 ms`, TQ `p50=1.23 ms`.
  - 50k/nprobe64: f32 `p50=4.89 ms`, TQ `p50=5.16 ms`.
  - 100k/nprobe64: f32 `p50=9.50 ms`, TQ `p50=9.83 ms`.
- Storage was materially worse for index-side TQ sidecar:
  - 10k: f32 index `2.9 MiB`, TQ index `10.9 MiB`.
  - 50k: f32 index `11.6 MiB`, TQ index `50.8 MiB`.
  - 100k: f32 index `22.5 MiB`, TQ index `100.8 MiB`.

100k/nprobe64 initial explain attribution:

- f32: `Rerank Source Bytes Read=614400`, `Execution Time=12.527 ms`.
- TQ stage-2: `TQ Stage2 Payload Bytes Scored=77200`,
  `TQ Stage2 Final Source Bytes Read=153600`,
  `Rerank Index Payload Segment Pages Read=361`,
  `Rerank Index Segment Payload Bytes Read=2839027`,
  `Rerank Payload Decode Elapsed Us=1202`, `Execution Time=15.363 ms`.

## Partial Loader / Copy-Avoidance Iteration

Code change under review:

- Loads direct index-side TQ rerank groups by requested survivor heap TIDs.
- Copies only requested selected payload ranges instead of assembling a full
  group payload slab.
- Preserves the full-group path for existing fallback callers.

Validation artifacts:

- `cargo-test-ec-ivf-scan-copy-avoidance.log`
- `cargo-build-release-copy-avoidance.log`
- `cargo-pgrx-install-copy-avoidance-escalated.log`

Focused test command:

```text
cargo test -p ecaz am::ec_ivf::scan --lib --no-default-features --features pg18
```

Key validation line:

```text
test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 2194 filtered out
```

Post-change scan-only A/B artifacts:

- `copy-avoidance-scan-full-results.jsonl`
- `copy-avoidance-scan-full-manifest.json`
- raw logs under `copy-avoidance-scan-full/`

Post-change recall:

- 10k: f32 and TQ stage-2 both `recall@10=1.0000` at nprobe 32 and 64.
- 50k: f32 and TQ stage-2 both `0.9960` at nprobe 32 and `1.0000` at nprobe 64.
- 100k: f32 and TQ stage-2 both `0.9730` at nprobe 32 and `1.0000` at nprobe 64.

Post-change latency:

| Scale | nprobe | f32 p50 | TQ p50 | f32 p95 | TQ p95 | f32 p99 | TQ p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 32 | 0.78 ms | 0.71 ms | 0.88 ms | 0.81 ms | 1.01 ms | 1.08 ms |
| 10k | 64 | 1.26 ms | 1.22 ms | 1.38 ms | 1.41 ms | 1.41 ms | 1.60 ms |
| 50k | 32 | 2.42 ms | 2.40 ms | 2.67 ms | 2.69 ms | 2.79 ms | 2.89 ms |
| 50k | 64 | 4.65 ms | 4.67 ms | 4.99 ms | 5.04 ms | 5.23 ms | 5.19 ms |
| 100k | 32 | 4.79 ms | 5.00 ms | 5.23 ms | 5.42 ms | 5.64 ms | 6.15 ms |
| 100k | 64 | 8.81 ms | 9.06 ms | 9.20 ms | 9.38 ms | 9.66 ms | 9.68 ms |

Post-change 100k/nprobe64 explain attribution:

- TQ stage-2: `TQ Stage2 Payload Bytes Scored=77200`,
  `TQ Stage2 Final Source Bytes Read=153600`,
  `Rerank Index Payload Segment Pages Read=216`,
  `Rerank Index Segment Payload Bytes Read=1748632`,
  `Rerank Payload Decode Elapsed Us=514`, `Execution Time=14.054 ms`.
- Compared with the initial TQ explain, segment pages fell `361 -> 216`,
  segment payload bytes fell `2839027 -> 1748632`, and decode time fell
  `1202 us -> 514 us`.

## SIMD / Scalar Counter Evidence

Artifact: `copy-avoidance-scan-full-results.jsonl`.

TQ stage-2 rows at every scale/nprobe report `scalar_candidates=0` and
`width_ge32=100`. Example 100k/nprobe64:

```text
latency-100k-tq-stage2-w100-final25 nprobe=64 turboquant candidates=10000 flushes=100 elapsed_ms=2.362373 scalar_candidates=0 width_ge32=100
```

Conclusion: this Task 124 hot path is using the full block/SIMD TQ scorer. The
remaining latency is from index-side payload locality and materialization, not
from scalar TQ score fallback.

## Outcome

This packet supports an **iterate** outcome, not a Task 124 closeout:

- matched recall is preserved at 10k/50k/100k;
- the scan-time sidecar loader optimization reduces wasted TQ payload IO/decode;
- TQ stage-2 moves near parity but still does not beat f32 consistently at
  50k/100k, and index-side TQ storage remains worse;
- next optimization should target durable TQ payload locality/layout, because
  the scorer is already SIMD and the current linked group/segment sidecar still
  reads far more bytes than the 77.2 KB actually scored.
