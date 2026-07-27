# Task 188 batch-10 stage-counter diagnostic

- Config: `task188-batch10-stage-counters-suite.json`
- Config SHA-256: `ad3cd8064cad5b26c06c0e10a0e34052c0c6c8bf08086bbc97adee1f54b6ec35`
- Command: `ecaz bench suite run` with `--distann-stage-counters`, one 100k
  physical step, BW4/BW8, explicit `materialization_batch_size=10`
- Outcome: setup reached the physical build, then failed with
  `EC_BUILD_INCOMPLETE: remote handoff stage failed: could not write to file
  pg_wal/xlogtemp.421118: No space left on device`
- Decision use: no stage-counter values from this failed run are used.

The accepted packet-005 latency rows remain the decision source. The mechanism
explanation is qualified with the packet-002 instrumented attribution rows:
BW4 9.72 traversal hop rounds / 25.86 remote candidates per scan versus BW8
5.58 / 29.56. Those rows were eager-0 and are not relabeled as batch-10
measurements. A future fresh run needs additional disk before claiming direct
batch-10 counter confirmation.
