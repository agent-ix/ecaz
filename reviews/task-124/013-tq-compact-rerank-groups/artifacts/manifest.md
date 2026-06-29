# Task 124 Packet 013 Artifacts

- Head SHA: `4f28d1e3095fee76e9d2d7e46e67d76963883226`
- Task bucket: `reviews/task-124/013-tq-compact-rerank-groups`
- Lane: local Apple/NEON PG18, `ec_real_100k`
- Purpose: test a temporary TQ structural storage slice using compact rerank group headers.
- Outcome: insufficient / negative. The temporary code was reverted and is not proposed for landing.

## Temporary Code Under Test

- Artifact: `discarded-compact-rerank-groups.diff`
- Change: add a compact `0x2D` rerank group header tag for TurboQuant index-side
  groups that omits per-entry gammas, heap-TID counts, heap-TID offsets, and
  payload offsets. Decode materializes the implicit logical arrays so scan and
  vacuum can keep the existing tuple API. Build and online insert select compact
  headers only for TurboQuant rerank formats.
- Rationale: reduce persisted TQ sidecar metadata bytes and page IO without
  changing recall semantics.

## Validation Commands

- `cargo fmt --check`
  - Result: passed.
- `cargo test -p ecaz am::ec_ivf::page --lib --no-default-features --features pg18`
  - Result: passed, 43 page tests.
- `cargo test -p ecaz am::ec_ivf::scan --lib --no-default-features --features pg18`
  - Result: passed, 30 scan tests.
- `cargo check -p ecaz --lib --no-default-features --features pg18`
  - Result: passed.
- `cargo build --release -p ecaz`
  - Result: passed.
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`
  - Result: passed.

## Suite

Command:

```text
/Users/peter/.cargo/bin/ecaz bench suite run --config reviews/task-124/013-tq-compact-rerank-groups/artifacts/task124-tq-compact-rerank-groups-100k-suite.json --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/013-tq-compact-rerank-groups/artifacts/suite-run.log
```

Report command:

```text
/Users/peter/.cargo/bin/ecaz bench suite report --manifest reviews/task-124/013-tq-compact-rerank-groups/artifacts/compact-groups-100k/suite-manifest.json
```

Artifacts:

- `task124-tq-compact-rerank-groups-100k-suite.json`: SuiteConfig.
- `suite-run.log`: suite runner log.
- `compact-groups-100k/suite-manifest.json`: structured suite manifest.
- `compact-groups-100k/results.jsonl`: structured parsed results.
- `compact-groups-100k/load-100k-tq-w75-g50-final15-compact-groups.log`
- `compact-groups-100k/recall-100k-tq-w75-g50-final15-compact-groups.log`
- `compact-groups-100k/latency-100k-tq-w75-g50-final15-compact-groups.log`
- `compact-groups-100k/storage-100k-tq-w75-g50-final15-compact-groups.log`

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
| 32 | 4.79 ms | 5.27 ms | 5.61 ms |
| 64 | 8.80 ms | 9.09 ms | 9.44 ms |

100k storage:

| index | size | per row |
| --- | ---: | ---: |
| `task124_tq_compact_groups_w75_g50_100k_coarse_rerank_idx` | 100.8 MiB | 1056.8 B |

Counters:

| nprobe | TQ isa | TQ scalar candidates | TQ candidates |
| --- | --- | ---: | ---: |
| 32 | neon | 0 | 7500 |
| 64 | neon | 0 | 7500 |

## Comparison To Packet 011

Packet 011 selected-payload slab baseline at the same 100k config:

| nprobe | packet 011 p50/p95/p99 | packet 013 p50/p95/p99 | result |
| --- | --- | --- | --- |
| 32 | 4.83 / 5.35 / 5.55 ms | 4.79 / 5.27 / 5.61 ms | mixed, tiny |
| 64 | 8.91 / 9.14 / 9.25 ms | 8.80 / 9.09 / 9.44 ms | mixed, tiny |

Storage changed from 1057.2 B/row to 1056.8 B/row at 100k, while remaining
reported as 100.8 MiB overall. This does not move the 4.5x storage wall called
out by the packet 011 reviewer.

## Decision

The compact-header slice is structurally aligned with the reviewer direction,
but the measured effect is too small to justify a new on-disk tag and codec
complexity. The code is reverted and should not land.
