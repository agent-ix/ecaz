# Task 188 no-reconnect follow-up

- Runner/build: `ecaz bench suite`, fixed committed release extension
  `d845d8e4347d59dafd2b1ed28cd252d7d7c6e134`, clean worktree, three-node
  physical fixture.
- Configuration: stage counters enabled; `benchmark_backend_batch_size=0`;
  no reconnect option; coverage statement enabled (`skip_recall` absent);
  both `bw4-control` and `bw8-candidate`; 50 iterations, 10 warmups;
  `sample_backend_memory` enabled at 250 ms.
- Coverage completed successfully for 200 queries with
  `zero_fraction=0`, `physical_topology_gate pass=true`, and source rows
  `100000`. The suite exited 0 and wrote `results.jsonl`.
- `bw4-control` memory series: 6 samples over 0–1263 ms, RSS
  `246792 -> 254764 KB` (`+7972 KB` startup rise), HWM constant at
  `378020 KB`.
- `bw8-candidate` memory series: 5 samples over 0–1010 ms, RSS
  `249388 -> 256636 KB` (`+7248 KB` startup rise), HWM constant at
  `379348 KB`.
- The bounded series and successful coverage run do not reproduce the prior
  multi-GB growth with the workaround disabled. This supports treating the
  reconnect workaround as dead weight after Task 188 owner confirmation;
  this packet does not edit Task 188's immutable config or remove its lane.

Primary evidence:

- `task188-fixed-no-reconnect-suite.json`
- `task188-fixed-no-reconnect-run-r2/suite-manifest.json`
- `task188-fixed-no-reconnect-run-r2/results.jsonl`
- `task188-fixed-no-reconnect-run-r2/distann-multinode-summary.log`
- `task188-fixed-no-reconnect-run-r2/physical-bw4-control-latency.memory-series.log`
- `task188-fixed-no-reconnect-run-r2/physical-bw8-candidate-latency.memory-series.log`
