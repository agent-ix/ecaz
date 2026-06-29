# Task 124 Packet 012 Artifacts

- Head SHA: `36d0089cef8e82cb994947b055651326e9d0484e`
- Task bucket: `reviews/task-124/012-tq-stage2-topk-fusion`
- Lane: local Apple/NEON PG18, `ec_real_100k`
- Purpose: test a temporary TQ stage-2 score/top-k/materialization fusion slice.
- Outcome: negative. The temporary code was reverted and is not proposed for landing.

## Temporary Code Under Test

- Artifact: `discarded-topk-fusion.diff`
- Change: after index-side TQ stage-2 scoring, retain only the top `stage2_final_rerank_width`
  candidates before final exact f32 rerank, instead of first sorting the full stage-2 prefix.
- Rationale: remove avoidable full-prefix sort/materialization work on the TQ stage-2 +
  final-exact path.

## Validation Commands

- `cargo fmt --check`
  - Result: passed.
- `cargo test -p ecaz am::ec_ivf::scan --lib --no-default-features --features pg18`
  - Result: passed, 32 scan tests.
- `cargo check -p ecaz --lib --no-default-features --features pg18`
  - Result: passed.
- `cargo build --release -p ecaz`
  - Result: passed.
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`
  - Result: passed.

## Suite

Command:

```text
/Users/peter/.cargo/bin/ecaz bench suite run --config reviews/task-124/012-tq-stage2-topk-fusion/artifacts/task124-tq-stage2-topk-fusion-100k-suite.json --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/012-tq-stage2-topk-fusion/artifacts/suite-run.log
```

Report command:

```text
/Users/peter/.cargo/bin/ecaz bench suite report --manifest reviews/task-124/012-tq-stage2-topk-fusion/artifacts/topk-fusion-100k/suite-manifest.json
```

Artifacts:

- `task124-tq-stage2-topk-fusion-100k-suite.json`: SuiteConfig.
- `suite-run.log`: suite runner log.
- `topk-fusion-100k/suite-manifest.json`: structured suite manifest.
- `topk-fusion-100k/results.jsonl`: structured parsed results.
- `topk-fusion-100k/load-100k-tq-w75-g50-final15-topk-fusion.log`
- `topk-fusion-100k/recall-100k-tq-w75-g50-final15-topk-fusion.log`
- `topk-fusion-100k/latency-100k-tq-w75-g50-final15-topk-fusion.log`
- `topk-fusion-100k/storage-100k-tq-w75-g50-final15-topk-fusion.log`

Do not commit generated truth-cache files from this packet.

## Key Results

Config:

- `storage_format=coarse_rerank`
- `rerank_placement=index`
- `rerank_format=turboquant`
- `rerank_width=75`
- `rerank_group_width=50`
- `stage2_final_rerank_width=15`

100k recall:

| nprobe | recall@10 | ndcg@10 |
| --- | ---: | ---: |
| 32 | 0.9730 | 0.9969 |
| 64 | 1.0000 | 1.0000 |

100k latency:

| nprobe | p50 | p95 | p99 |
| --- | ---: | ---: | ---: |
| 32 | 5.13 ms | 5.62 ms | 5.97 ms |
| 64 | 9.26 ms | 9.49 ms | 9.83 ms |

100k storage:

| index | size | per row |
| --- | ---: | ---: |
| `task124_tq_topk_fusion_w75_g50_100k_coarse_rerank_idx` | 100.8 MiB | 1057.2 B |

Counters:

| nprobe | TQ isa | TQ scalar candidates | TQ candidates |
| --- | --- | ---: | ---: |
| 32 | neon | 0 | 7500 |
| 64 | neon | 0 | 7500 |

## Comparison To Packet 011

Packet 011 selected-payload slab baseline at the same 100k config:

| nprobe | packet 011 p50/p95/p99 | packet 012 p50/p95/p99 | result |
| --- | --- | --- | --- |
| 32 | 4.83 / 5.35 / 5.55 ms | 5.13 / 5.62 / 5.97 ms | regression |
| 64 | 8.91 / 9.14 / 9.25 ms | 9.26 / 9.49 / 9.83 ms | regression |

Recall stayed unchanged and storage stayed at 100.8 MiB, so this slice does not
improve the Task 124 objective and should not land.
