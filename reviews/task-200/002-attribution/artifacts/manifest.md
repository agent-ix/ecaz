# Task 200 attribution artifacts

- Packet: `reviews/task-200/002-attribution/`
- Owning task: `plan/tasks/200-ec-distann-backend-memory-retention.md`
- Code head: `fa84ff3b0` (fix checkpoint); attribution runs used the prior
  dirty extension build `897c69045249a876de151c1da0544001ead82352-dirty`.
- Fixture: `/home/peter/.ecaz/clusters/task200-counters-off-100k`; reused, never
  rebuilt for attribution. It contains the 100k physical generation from the
  reproduction packet.
- A1 command: `ecaz bench latency --prefix task179_physical_100k
  --profile ec_distann --iterations 300 --warmup-iterations 10
  --hold-transaction --sample-backend-memory`, one coordinator backend.
- A1 result: RSS 252360–259596 KB across 300 production queries; see
  `held-tx-a1/latency.memory-series.log` and `held-tx-a1/latency.log`.
- A2 open-only commands: `BEGIN; SELECT
  ec_distann_physical_scan_open_benchmark('dm_idx', 1|200); SELECT
  pg_log_backend_memory_contexts(pg_backend_pid()); COMMIT;`.
  Both ended at `TopTransactionContext: 142606336 total`.
- A2 owner command: `BEGIN; SELECT
  ec_distann_physical_owner_seed_scan_benchmark('dm_idx', source, 20, 32)
  FROM task179_physical_100k_queries LIMIT 1; ... COMMIT;`.
  It ended at `TopTransactionContext: 5595201536 total`.
- Root cause: pgrx `Vec<u8>` bytea conversion retained detoast copies in
  `TopTransactionContext`; raw SPI datum plus `DetoastedVarlena` ownership is
  the fix in `fa84ff3b0`.
- Provenance note: the measured extension tree also contained approximately
  30 unrelated dirty `src/am` files owned by other agents. Final code and
  regression source are committed at `fa84ff3b0`; the dirty SHA is retained
  here for honest attribution provenance.
- No corpus, query TSV, PGDATA, or cluster directory is committed.
