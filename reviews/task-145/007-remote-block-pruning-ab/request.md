# Task 145 Packet 007: Remote Block-Pruning A/B

## Review Request

Please review the current-head remote block-pruning A/B packet for Task 145.

This packet responds to Task 145 packet 006 feedback by rerunning at head after remote leaf-block GUC propagation and by proving the block-pruning counters are nonzero on the remote production-read path.

## Changes Under Review

Code commits on branch `task-145-spire-rerank-economy-low-probe`:

- `aac7fba8a fix(task-145): propagate remote leaf block scan gucs`
- `32f82b6a3 fix(task-145): propagate remote leaf block gucs to profiles`
- merge head for this packet: `f2317c97d6d9a85e42ddc06e58c052a2a8fdf608`

## Evidence

Packet-local manifest:

- `reviews/task-145/007-remote-block-pruning-ab/artifacts/manifest.md`

Primary suite:

- `reviews/task-145/007-remote-block-pruning-ab/artifacts/task145-remote-block-pruning-ab-suite.json`
- `reviews/task-145/007-remote-block-pruning-ab/artifacts/suite-run-r2.log`
- `reviews/task-145/007-remote-block-pruning-ab/artifacts/suite-manifest-r2.json`

Nested per-cell results:

- `remote-10k-n128-block-r2/bench-suite/results.jsonl`
- `remote-50k-n1024-block-r2/bench-suite/results.jsonl`
- `remote-100k-n1024-block-r2/bench-suite/results.jsonl`

## Result

This is not a promote result for `block-global128`.

The positive result is that remote block pruning is now demonstrably engaged at current head:

- 10k n128, nprobe96: `leaf_block_skipped_sum=25912`, recall stays 1.0000.
- 50k n1024, nprobe96: `leaf_block_skipped_sum=1765`, recall stays 0.9595.
- 100k n1024, nprobe96: `leaf_block_skipped_sum=49387`, but recall drops.

At nprobe96:

| Cell | block-off recall/p50/p95 | block-global128 recall/p50/p95 |
| --- | --- | --- |
| 10k n128 | 1.0000 / 136.745 ms / 143.083 ms | 1.0000 / 135.147 ms / 140.111 ms |
| 50k n1024 | 0.9595 / 139.848 ms / 147.403 ms | 0.9595 / 139.496 ms / 144.044 ms |
| 100k n1024 | 0.9570 / 142.663 ms / 147.220 ms | 0.9340 / 144.001 ms / 154.852 ms |

Identity comparison:

- 10k: byte-identical.
- 50k: byte-identical.
- 100k: diverged on 62/1000 query rows.

## Decision

Treat this as an iterate/escalate packet:

- AC1 remote block-pruning instrumentation is now real and nonzero.
- `global128` is too aggressive at 100k n1024; it is recall-stressing and loses recall.
- Next Task 145 work should tune the pruning threshold/geometry rather than promote this setting.

Focused validation:

- `cargo test production_executor_compact_receive_requests_use_dispatch_state --no-default-features --features pg18` passed.

Suite validation:

- `ecaz bench suite audit` passed.
- Full `ecaz bench suite run` completed 10k/50k/100k with release install and release per-node `ecaz_build_profile()`.
