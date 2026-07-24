# Task 198 packet 003 artifact manifest

- Branch head: `4c09b9a99`
- Installed extension SHA: `2ff72b3e49609c44cec881f72edf183a83554412`
  (`git diff 2ff72b3e4..4c09b9a99 -- src sql` is empty; later commits
  touch only the CLI correctness harness and suite config)
- Task bucket: `reviews/task-198/003-lifecycle-and-faults/`
- Lane: Intel local, three independent PG18 owner processes, coordinator is
  owner zero
- Fixture: exact/disjoint hash ownership, one index per source table
- Storage: RaBitQ graph neighbor values, exact final score, owner-side lazy10
  payloads, optional faithful coordinator traversal replica
- Corpus: staged `ec_real_10k`; evaluation queries 1--10; training landmarks
  use rows 201--400 of the staged 100k query file
- Timestamp: 2026-07-23 America/Los_Angeles

## Commands and artifacts

Release installation:

```text
PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18,distann-head-attribution-benchmark
```

See `release-install.log`, `release-binary-preflight.log`, and
`cli-release-build.log`. Installed and target libraries are both 24,602,960
bytes with SHA-256
`8004ebd15da0e39df0d8237237a8e89581f8c932d92db21d3eca6f47c64db235`.

Suite audit and run:

```text
target/debug/ecaz bench suite audit --config reviews/task-198/003-lifecycle-and-faults/artifacts/task198-lifecycle-smoke-10k.json
target/debug/ecaz bench suite run --config reviews/task-198/003-lifecycle-and-faults/artifacts/task198-lifecycle-smoke-10k.json --artifact-dir reviews/task-198/003-lifecycle-and-faults/artifacts/run
```

The CLI process is a fixture driver; every backend attested the installed
extension as `release`. The immutable config, `suite-manifest.json`,
`results.jsonl`, raw recall/latency logs, node logs, and compact summary are
under `run/`.

## Key result lines

- Build rollback with owner three unavailable: pass, catalog residue zero.
- Successful replica: 10,000/10,000 rows; 131,520,000 source bytes copied;
  158,326,784 physical bytes; 137,460,584 WAL bytes; 5,181 ms build;
  3,366,912-byte peak copy batch.
- Idempotent build replay: same content digest.
- Recall: owner `0.9900`, replica `0.9900`.
- Two-sample lifecycle-smoke latency: owner 21.80 ms, replica 17.90 ms.
- Traversal attribution: owner 9.607095 ms; replica 4.315152 ms;
  replica graph/vector read 4.085136 ms; replica score 0.145550 ms; remote
  traversal wait/expand zero; reconciliation passed.
- Ready semantic identity, mid-scan full restart, corrupt/partial fallback,
  durable Ready-to-Stale invalidation (`40001`, exactly one failing attempt),
  retry owner fallback, retire/reclaim replay, and removed-image fallback:
  all pass.
- Null, external uncompressed TOAST, qualified rejection/deepening,
  mixed-local/remote, and post-first-batch remote-failure scenarios: all
  ordered-result identities pass between owner and replica arms.

This is lifecycle evidence, not the Task 198 performance decision. Packet 004
owns the required 200-query/50-iteration isolated 100k A/B.
