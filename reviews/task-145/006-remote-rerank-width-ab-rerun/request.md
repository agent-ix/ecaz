# Task 145 Packet 006: Remote Rerank Width A/B Rerun

## Request

Please review the Task 145 remote rerank-width rerun and the companion fix in
`088054027e0b683fdd47a80fdb9410e1c2f361d9`.

This packet addresses packet 004/005 feedback by proving that remote
`ec_spire.rerank_width=50` now truncates the remote heap frontier, while
preserving the exact returned identities against width-0/full.

## Code Under Review

`088054027` keeps the automatic width-0 production top-k path bounded to
`top_k`, while preserving explicit `max_candidate_rows` as the remote heap
frontier. The focused regression is:

- `cargo test production_scan_heap_frontier --no-default-features --features pg18`
- Evidence: `artifacts/cargo-test-production-scan-heap-frontier.log`
- Result: 2 passed, 0 failed

## Bench Evidence

Suite config:

- `artifacts/task145-remote-rerank-width-ab-rerun-suite.json`
- `artifacts/suite-manifest.json`
- `artifacts/suite-run.log`
- Per-cell nested `bench-suite/results.jsonl` files are the authoritative
  result JSONL. The top-level `suite-results.jsonl` is empty for these nested
  multinode steps; this is called out in `artifacts/manifest.md`.

Release profile evidence:

- `artifacts/remote-10k-n128-r6/local-multinode.log`
- `artifacts/remote-50k-n1024-r6/local-multinode.log`
- `artifacts/remote-100k-n1024-r6/local-multinode.log`
- Each records `install_profile=release` and release node build profiles for
  the coordinator and all three remotes.

## Key Results

At nprobe 96:

| Cell | Variant | distinct_recall@k | p50 | p95 | remote heap candidates |
| --- | --- | ---: | ---: | ---: | ---: |
| 10k n128 | width 0/full | 1.0000 | 219.866 ms | 224.686 ms | 60000 |
| 10k n128 | width 50 | 1.0000 | 132.399 ms | 137.166 ms | 30000 |
| 50k n1024 | width 0/full | 0.9595 | 222.755 ms | 229.324 ms | 60000 |
| 50k n1024 | width 50 | 0.9595 | 140.584 ms | 144.977 ms | 30000 |
| 100k n1024 | width 0/full | 0.9570 | 227.666 ms | 234.037 ms | 60000 |
| 100k n1024 | width 50 | 0.9570 | 141.295 ms | 147.078 ms | 30000 |

Full vs width-50 identity JSONL files are byte-identical for 10k, 50k, and
100k. Each identity file has 1000 rows.

The full nprobe table, storage evidence, commands, and artifact inventory are in
`artifacts/manifest.md`.

## Decision

Promote/continue with remote `rerank_width=50` as the Task 145 rerank economy
candidate. Packet 006 shows:

- real remote heap truncation: 60,000 -> 30,000 remote heap candidates at
  nprobe 96
- exact identity parity against width 0/full on all three required scales
- lower p50/p95 latency on every scale and nprobe measured
- no storage delta between A/B variants, because the change is scan-time only

## Notes

The suite uses `ec_spire.max_candidate_rows=100`, not 200, to remain within the
default per-batch remote payload row cap while still giving width 50 a real
2x frontier reduction to prove.
