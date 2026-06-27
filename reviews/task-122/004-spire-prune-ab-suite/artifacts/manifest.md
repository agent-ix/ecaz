# Task 122 / 004 SPIRE prune A/B suite manifest

- Head SHA: `aa799704b` (`Gate SPIRE pre-materialization prune`)
- Task bucket: `reviews/task-122/004-spire-prune-ab-suite/`
- Timestamp: `2026-06-27T04:29:20Z`
- Lane: local PG18 debug smoke, staged `ec_real_10k`
- Fixture: `data/staged-current/ec_real_10k_corpus.tsv`,
  `data/staged-current/ec_real_10k_queries.tsv`,
  `data/staged-current/ec_real_10k_manifest.json`
- Access method: `ec_spire`
- Quant/storage formats: `turboquant`, `rabitq`
- Rerank mode: `rerank_width=25`, `ec_spire.max_candidate_rows=25`
- Isolation: one index/table prefix per quant and scale

## Code under review

`aa799704b` adds `ec_spire.pre_materialization_prune`, defaulting on, and wires
the existing pre-materialization threshold checks through that GUC. This enables
same-binary A/B runs with `ec_spire.pre_materialization_prune=on/off`.

Validation before this packet:

- `cargo fmt --check`
- `cargo test -p ecaz --lib --no-default-features --features pg18 collect_scan_placement_diagnostics`
- `cargo check -p ecaz --lib --no-default-features --features pg18`

## Suite config and harness checks

Artifact: `task122-spire-prune-ab-suite.json`

The suite contains 36 steps for 10k / 50k / 100k:

- TQ load, recall prune-on/off, latency prune-on/off, SPIRE pipeline prune-on/off, storage
- RaBitQ load, recall, latency, storage

Audit command:

```sh
/Users/peter/.cargo/bin/ecaz bench suite audit \
  --config reviews/task-122/004-spire-prune-ab-suite/artifacts/task122-spire-prune-ab-suite.json \
  --log-file reviews/task-122/004-spire-prune-ab-suite/artifacts/suite-audit.log
```

Dry-run command:

```sh
/Users/peter/.cargo/bin/ecaz bench suite run \
  --config reviews/task-122/004-spire-prune-ab-suite/artifacts/task122-spire-prune-ab-suite.json \
  --dry-run \
  --log-file reviews/task-122/004-spire-prune-ab-suite/artifacts/suite-dry-run.log
```

Setup / GUC check artifacts:

- `cargo-pgrx-install.log`
- `guc-check.log`

`guc-check.log` shows the backend was `debug` and the default
`ec_spire.pre_materialization_prune` value was `on`.

## 10k debug smoke

Command:

```sh
/Users/peter/.cargo/bin/ecaz bench suite run \
  --config reviews/task-122/004-spire-prune-ab-suite/artifacts/task122-spire-prune-ab-suite.json \
  --only-tag ec_real_10k \
  --allow-debug-backend \
  --host /Users/peter/.pgrx \
  --port 28818 \
  --log-file reviews/task-122/004-spire-prune-ab-suite/artifacts/suite-run-10k-debug.log
```

Primary structured artifacts:

- `suite/suite-manifest.json`
- `suite/results.jsonl`

Key result lines from `suite/results.jsonl`:

- TQ prune on recall: recall@k `1.0000`, ndcg@k `1.0000`, mean q-time `92.14 ms`
- TQ prune off recall: recall@k `1.0000`, ndcg@k `1.0000`, mean q-time `91.92 ms`
- TQ prune on latency: p50 `92.4 ms`, p95 `96.1 ms`, p99 `106.9 ms`
- TQ prune off latency: p50 `93.8 ms`, p95 `99.0 ms`, p99 `105.9 ms`
- TQ prune on pipeline candidates: item_sum `8495`, ready_sum `2500`, blocked_sum `249055`, candidate_sum `8495`, heap_rerank_sum `0`
- TQ prune off pipeline candidates: item_sum `251555`, ready_sum `2500`, blocked_sum `249055`, candidate_sum `251555`, heap_rerank_sum `0`
- TQ prune on heap rerank: item_sum `2500`, candidate_sum `2500`, heap_rerank_sum `2500`
- TQ prune off heap rerank: item_sum `2500`, candidate_sum `2500`, heap_rerank_sum `2500`
- TQ storage index: `8.9 MiB`, `931.4 B` per row
- RaBitQ recall: recall@k `1.0000`, ndcg@k `1.0000`, mean q-time `72.88 ms`
- RaBitQ latency: p50 `74.2 ms`, p95 `81.3 ms`, p99 `85.6 ms`
- RaBitQ storage index: `9.0 MiB`, `939.6 B` per row

Interpretation:

- The GUC and suite plumbing work.
- With `rerank_width=25` and `max_candidate_rows=25`, the prune-on path
  materialized `8,495` TQ candidates versus `251,555` with pruning off while
  preserving the same 2,500 heap rerank rows and 1.0000 recall in this smoke.
- This is not closeout evidence: it used a debug backend and only the 10k scale.
  Task closeout still requires release A/B results at 10k / 50k / 100k with
  recall, latency, and storage.
