# Review Request: Task 111h Legacy 0x2A Baseline Attribution

This packet requests review of the legacy `0x2A` direct-TID sidecar baseline
evidence for Task 111h.

No code changed. No new benchmark was run.

What this packet establishes:

- Current v7 HEAD cannot rerun `0x2A` directly because it writes packed
  `0x2B`/`0x2C` rerank groups and rejects v4 metadata.
- The legacy direct-TID `0x2A` baseline already has durable benchmark evidence
  in `reviews/task-111g/005-direct-sidecar-rerank-tids`.
- That packet was driven by `ecaz bench suite`, completed with
  `completed=24 failed=0 skipped=0`, and contains config, manifest,
  `results.jsonl`, and raw load/recall/latency/storage logs.

Primary artifact:

- `artifacts/legacy-0x2a-attribution.md`

Headline legacy rows at 100k:

| Legacy `0x2A` direct-TID cell | nprobe 8 | nprobe 64 | nprobe 200 |
| --- | ---: | ---: | ---: |
| f16 p50 latency | 2.99 ms | 6.02 ms | 13.0 ms |
| f16 recall@10 | 0.7670 | 0.9640 | 0.9975 |
| rabitq4 p50 latency | 2.79 ms | 5.72 ms | 11.9 ms |
| rabitq4 recall@10 | 0.7465 | 0.9165 | 0.9420 |

Storage at 100k:

- f16 index: `416.6 MiB`
- rabitq4 index: `103.6 MiB`

Review focus:

- Confirm it is acceptable for Task 111h to close the legacy `0x2A` benchmark
  row by citing the completed 111g/005 direct-TID suite rather than rerunning
  an old metadata version on current v7.
- Confirm the packet clearly separates:
  - the old directory-only ADR-079 150 ms f16 bug,
  - the fixed direct-TID `0x2A` legacy baseline,
  - the current v7 packed group layout under decision.

Non-claim: this does not close Task 111h. Remaining open items include
table-owned storage evidence/replacement rationale, RaBitQ slab cleanup or
benchmark-away evidence, cold/remote evidence, and the final decision table.
