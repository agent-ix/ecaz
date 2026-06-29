# Task 124 Packet 014 Artifacts

- Head SHA: `249780af9b6d01bed56c1e0932d61d83b8464819`
- Task bucket: `reviews/task-124/014-tq-direct-slot-rerank`
- Lane: local Apple/NEON PG18, `ec_real_100k`
- Purpose: test a temporary TQ structural materialization slice using posting-carried direct group slots.
- Outcome: insufficient / negative. The temporary code was reverted and is not proposed for landing.

## Temporary Code Under Test

- Artifact: `discarded-direct-slot-rerank.diff`
- Change: encode a tagged group-slot reference in the posting `rerank_tid` when
  the group header physical offset and slot fit the tagged `ItemPointer`
  representation. Scan decodes the tag, groups by the physical group header TID,
  and computes selected payload offsets by slot instead of scanning the group
  heap-TID arrays. Legacy untagged group-header TIDs continue to use the old
  heap-TID lookup.
- Rationale: remove per-candidate group-local heap-TID scanning from TQ stage-2
  payload materialization without changing recall semantics.

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
/Users/peter/.cargo/bin/ecaz bench suite run --config reviews/task-124/014-tq-direct-slot-rerank/artifacts/task124-tq-direct-slot-rerank-100k-suite.json --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/014-tq-direct-slot-rerank/artifacts/suite-run.log
```

Report command:

```text
/Users/peter/.cargo/bin/ecaz bench suite report --manifest reviews/task-124/014-tq-direct-slot-rerank/artifacts/direct-slot-100k/suite-manifest.json
```

Artifacts:

- `task124-tq-direct-slot-rerank-100k-suite.json`: SuiteConfig.
- `suite-run.log`: suite runner log.
- `direct-slot-100k/suite-manifest.json`: structured suite manifest.
- `direct-slot-100k/results.jsonl`: structured parsed results.
- `direct-slot-100k/load-100k-tq-w75-g50-final15-direct-slot.log`
- `direct-slot-100k/recall-100k-tq-w75-g50-final15-direct-slot.log`
- `direct-slot-100k/latency-100k-tq-w75-g50-final15-direct-slot.log`
- `direct-slot-100k/storage-100k-tq-w75-g50-final15-direct-slot.log`

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
| 32 | 4.77 ms | 5.37 ms | 5.62 ms |
| 64 | 8.76 ms | 9.12 ms | 9.32 ms |

100k storage:

| index | size | per row |
| --- | ---: | ---: |
| `task124_tq_direct_slot_w75_g50_100k_coarse_rerank_idx` | 100.8 MiB | 1057.2 B |

Counters:

| nprobe | TQ isa | TQ scalar candidates | TQ candidates |
| --- | --- | ---: | ---: |
| 32 | neon | 0 | 7500 |
| 64 | neon | 0 | 7500 |

## Comparison To Packet 011

Packet 011 selected-payload slab baseline at the same 100k config:

| nprobe | packet 011 p50/p95/p99 | packet 014 p50/p95/p99 | result |
| --- | --- | --- | --- |
| 32 | 4.83 / 5.35 / 5.55 ms | 4.77 / 5.37 / 5.62 ms | mixed, tiny |
| 64 | 8.91 / 9.14 / 9.25 ms | 8.76 / 9.12 / 9.32 ms | mixed, tiny |

Storage stayed at 1057.2 B/row and 100.8 MiB, so this slice does not address
the storage wall. Latency movement is too small and mixed to justify the tagged
posting TID complexity.

## Decision

The direct-slot slice is structurally aligned with the reviewer direction, but
the measured effect is insufficient. The code is reverted and should not land.
