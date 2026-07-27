# Task 188 batch-10 stage-counter diagnostic

- Config: `task188-batch10-stage-counters-suite.json`
- Config SHA-256: `ad3cd8064cad5b26c06c0e10a0e34052c0c6c8bf08086bbc97adee1f54b6ec35`
- Command: `ecaz bench suite run` with `--distann-stage-counters`, one 100k
  physical step, BW4/BW8, explicit `materialization_batch_size=10`
- Outcome: setup reached the physical build, then failed with
  `EC_BUILD_INCOMPLETE: remote handoff stage failed: could not write to file
  pg_wal/xlogtemp.421118: No space left on device`
- Decision use: no stage-counter values from this failed run are used.

## 2026-07-27 rerun

The same preregistered suite was rerun after disk cleanup with approximately
653 GB free. Physical setup completed successfully: all three nodes reached
`Published`, the source row count was 100,000, and orphan counts were zero.
The repeated-query phase did not complete. Its PostgreSQL latency backend grew
to approximately 52 GB RSS, leaving about 10 GB available memory and still
growing, so the run was terminated to protect the host. No latency or
stage-counter values from this rerun are used as evidence; the final diagnostic
remains outstanding. The packet-local outcome is recorded at
`artifacts/run/rerun-20260727/outcome.md`.

The accepted packet-005 latency rows remain the decision source. The mechanism
explanation is qualified with the packet-002 instrumented attribution rows:
BW4 9.72 traversal hop rounds / 25.86 remote candidates per scan versus BW8
5.58 / 29.56. Those rows were eager-0 and are not relabeled as batch-10
measurements. A future fresh run needs additional disk before claiming direct
batch-10 counter confirmation.

## Efficient rerun result

The efficient diagnostic completed successfully in
`artifacts/run/efficient-20260727-r2/`. It skipped the duplicate single-index
build, recall matrix, and seed-coverage query, and reconnected the latency
worker every five timed queries. The physical 100k serving gate passed with
100,000 source rows and zero orphans. Both BW4 and BW8 emitted 37 stage rows
and 28 materialization-work rows; traversal reconciliation passed for both.

The run's only latency value used for comparison is p50: 28.10 ms for BW4 and
25.80 ms for BW8. Its mean/p95/p99 samples are harness-affected because the
worker did not re-warm after reconnecting, so the first timed query of each
five-query batch was cold. In particular, the reported custom-scan means
`233.338562 -> 228.812245 ms` are contaminated in the same way and are not
warm per-scan costs. The direct stage values worth retaining are
remote-expand `11.691947 -> 9.218777 ms` and traversal-total
`13.439234 -> 11.034471 ms`.

The first fresh direct batch-10 work attribution showed traversal hop rounds
`9.72 -> 5.58` per scan. Remote candidates are the stronger result:
packet-002 eager-0 rows were `25.86 / 29.56` per scan for BW4/BW8, while this
explicit batch-10 rerun measured `6.64 / 6.62`. The roughly four-fold change is
from batch deduplication/materialization semantics, not a unit change: the
same candidate work is counted after being coalesced into the ranked batch.
This removes BW8's earlier remote-work penalty and brings it to parity with
BW4. Recall remains intentionally sourced from packet 005.

See the packet-local `outcome.md`, `resource-checks.md`, `results.jsonl`, and
the two arm latency logs for the complete evidence.
