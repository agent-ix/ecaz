# Review Request: TQ Stage-2 Top-K Fusion Negative Result

## Summary

This packet evaluates and rejects a narrow TQ stage-2 score/top-k/materialization
fusion slice.

The temporary code retained only the top `stage2_final_rerank_width` candidates
after TQ stage-2 scoring and before final exact f32 rerank, instead of sorting the
full stage-2 prefix first. It was benchmarked, regressed latency, and was
removed. There is no source code change proposed for landing from this packet.

Temporary diff under test:

- `reviews/task-124/012-tq-stage2-topk-fusion/artifacts/discarded-topk-fusion.diff`

Artifact manifest:

- `reviews/task-124/012-tq-stage2-topk-fusion/artifacts/manifest.md`

## Validation

Passed before benchmark:

- `cargo fmt --check`
- `cargo test -p ecaz am::ec_ivf::scan --lib --no-default-features --features pg18`
- `cargo check -p ecaz --lib --no-default-features --features pg18`
- `cargo build --release -p ecaz`
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`

Suite:

```text
/Users/peter/.cargo/bin/ecaz bench suite run --config reviews/task-124/012-tq-stage2-topk-fusion/artifacts/task124-tq-stage2-topk-fusion-100k-suite.json --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/012-tq-stage2-topk-fusion/artifacts/suite-run.log
```

## Result

Same 100k TQ4 final15 config as packet 011:

- `rerank_format=turboquant`
- `rerank_width=75`
- `rerank_group_width=50`
- `stage2_final_rerank_width=15`

Recall stayed unchanged:

| nprobe | recall@10 |
| --- | ---: |
| 32 | 0.9730 |
| 64 | 1.0000 |

Latency regressed versus packet 011:

| nprobe | packet 011 p50/p95/p99 | packet 012 p50/p95/p99 |
| --- | --- | --- |
| 32 | 4.83 / 5.35 / 5.55 ms | 5.13 / 5.62 / 5.97 ms |
| 64 | 8.91 / 9.14 / 9.25 ms | 9.26 / 9.49 / 9.83 ms |

Storage did not move:

- `100.8 MiB`, `1057.2 B/row`

TQ scorer still used NEON, not scalar:

| nprobe | isa | scalar candidates | candidates |
| --- | --- | ---: | ---: |
| 32 | neon | 0 | 7500 |
| 64 | neon | 0 | 7500 |

## Decision

Do not land this top-k fusion slice. It does not address the storage wall and it
made warm-cache latency worse.

The next Task 124 implementation attempt should be the larger structural storage
slice identified by review feedback: compact the packed rerank group layout /
direct payload addressing, or run the requested Phase 6 cold/IO validation to
decide Shelve vs deeper layout work.
