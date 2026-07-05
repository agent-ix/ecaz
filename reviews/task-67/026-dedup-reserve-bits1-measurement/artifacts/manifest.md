# Task 67 Packet 026 Manifest

- Task bucket: `reviews/task-67/`
- Packet path: `reviews/task-67/026-dedup-reserve-bits1-measurement/`
- Head SHA: `521bbec4bbd58fd9809b18a7261e160728b6c04a`
- Code under test: `1988ee51fb2b2527f2a6dcbcdc7de17bd0674406`
- Revert commit: `521bbec4bbd58fd9809b18a7261e160728b6c04a`
- Timestamp: 2026-05-30
- Lane: AWS Intel `10k-intel`, PG18, real 10k corpus, `ec_ivf`, RaBitQ bits=1
- Fixture: `target/real-corpus/staged-task50/ec_real_10k_{corpus,queries,manifest}.json|tsv`, 200-query recall and latency suites
- Storage format: isolated one-index-per-table surfaces, `storage_format=rabitq`, `quant_bits=1`
- Rerank mode: `rerank=heap_f32`, `rerank_width=100`
- Scratch mode: `ivf_scratch_soa_batch_decode=on`
- Surface isolation:
  - Scalar prefix: `task67_dedup_10k_rabitq1_scalar`
  - Auto prefix: `task67_dedup_10k_rabitq1_auto`

## Experiment

The code under test capped the initial `candidate_dedup` reserve for bounded
rerank scans to `min(candidate_bound, rerank_width * HEAPTID_INLINE_CAPACITY)`.
The experiment regressed latency and was reverted.

## Commands

Local validation before the experiment commit:

```sh
cargo fmt --check
cargo test -p ecaz --lib candidate_dedup_initial_capacity_caps_when_running_top_prunes
cargo test -p ecaz --lib candidate_top_k_rejects_only_scores_worse_than_full_worst
```

Suite audits:

```sh
target/debug/ecaz bench suite audit --config reviews/task-67/026-dedup-reserve-bits1-measurement/artifacts/task67-dedup-reserve-bits1-scalar-suite.json
target/debug/ecaz bench suite audit --config reviews/task-67/026-dedup-reserve-bits1-measurement/artifacts/task67-dedup-reserve-bits1-auto-suite.json
```

AWS setup:

```sh
target/debug/ecaz cloud resume --profile 10k-intel --log-file reviews/task-67/026-dedup-reserve-bits1-measurement/artifacts/preflight/cloud-resume.log
target/debug/ecaz cloud install --profile 10k-intel --git-ref 1988ee51f --skip-extension-recreate --database postgres --timeout 3600 --log-file reviews/task-67/026-dedup-reserve-bits1-measurement/artifacts/preflight/cloud-install-1988ee51f.log
```

AWS suite execution:

```sh
target/debug/ecaz cloud bench --profile 10k-intel --config reviews/task-67/026-dedup-reserve-bits1-measurement/artifacts/task67-dedup-reserve-bits1-scalar-suite.json
target/debug/ecaz cloud bench --profile 10k-intel --config reviews/task-67/026-dedup-reserve-bits1-measurement/artifacts/task67-dedup-reserve-bits1-auto-suite.json
```

AWS shutdown:

```sh
target/debug/ecaz cloud pause --profile 10k-intel --log-file reviews/task-67/026-dedup-reserve-bits1-measurement/artifacts/preflight/cloud-pause.log
target/debug/ecaz cloud status --profile 10k-intel
```

`artifacts/preflight/cloud-status-after-pause.log` records the final paused
state with `~$0.00/hr running` cost.

## Artifact Inventory

Suite configs:

- `artifacts/task67-dedup-reserve-bits1-scalar-suite.json`
- `artifacts/task67-dedup-reserve-bits1-auto-suite.json`

Local audit and cloud preflight:

- `artifacts/local/suite-audit-scalar.log`
- `artifacts/local/suite-audit-auto.log`
- `artifacts/preflight/cloud-status-after-pause.log`

Scalar suite:

- `artifacts/scalar/results.jsonl`
- `artifacts/scalar/suite-manifest.json`
- `artifacts/scalar/suite-run.log`
- `artifacts/scalar/load-10k-rabitq1-dedup-reserve-scalar.log`
- `artifacts/scalar/recall-10k-rabitq1-dedup-reserve-scalar.log`
- `artifacts/scalar/latency-10k-rabitq1-dedup-reserve-scalar.log`
- `artifacts/scalar/truth-ec-real-10k-q200-k10.json`

Auto suite:

- `artifacts/auto/results.jsonl`
- `artifacts/auto/suite-manifest.json`
- `artifacts/auto/suite-run.log`
- `artifacts/auto/load-10k-rabitq1-dedup-reserve-auto.log`
- `artifacts/auto/recall-10k-rabitq1-dedup-reserve-auto.log`
- `artifacts/auto/latency-10k-rabitq1-dedup-reserve-auto.log`
- `artifacts/auto/truth-ec-real-10k-q200-k10.json`

## Key Results

Latency means:

| nprobe | scalar | auto |
| --- | ---: | ---: |
| 16 | 1.33 ms | 1.31 ms |
| 32 | 1.76 ms | 1.73 ms |
| 64 | 2.65 ms | 2.60 ms |

Auto recall:

| nprobe | recall@10 | mean q-time |
| --- | ---: | ---: |
| 16 | 0.9985 | 1.34 ms |
| 32 | 1.0000 | 1.62 ms |
| 64 | 1.0000 | 2.40 ms |

Compared with packet 022 auto latencies of 1.08 / 1.47 / 2.14 ms, this
experiment regressed at every measured `nprobe`. The reverted experiment does
not advance the Task 67 SQL wall-time gate.
