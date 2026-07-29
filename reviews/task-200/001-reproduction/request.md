---
agent: codex
role: coder
model: GPT-5
date: 2026-07-28
seq: 2
---

# Task 200 Phase 1 reproduction

The required 100k three-owner reproduction is complete for both stage-counter
settings, with one backend and no reconnects. The counters-on arm reused the
existing counters-off PGDATA after the fixture-reuse checkpoint; it did not
rebuild the corpus or physical generation.

Production physical latency is flat in both arms:

- counters off: 300 queries, mean 27.50 ms; RSS 260104→261028 KB over 8067 ms
  (114.54 KB/s)
- counters on: 300 queries, mean 26.50 ms; RSS 260024→261024 KB over 7817 ms
  (127.93 KB/s)

The large RSS growth is in the benchmark-only
`ec_distann_physical_seed_coverage_benchmark` statement; PostgreSQL reported
8.32 GB of backend memory at a captured point. The statement text is retained
in `run-latency-rerun/diagnostic-node1.log`. The old
`coverage-separate-200.log` artifact is explicitly excluded: its 200 calls
were sent in one simple-query protocol message and therefore one implicit
transaction, so it cannot distinguish statement from transaction lifetime.
Those diagnostics were canceled before the known multi-GB failure point, with
the result and memory-context logs retained in the packet.

The implementation checkpoint is commit `9de8b4fa2`, pushed to the current
branch. It adds streamed RSS series capture and an opt-in, provenance-checked
`reuse_fixture` path in `ecaz bench suite`; rebuild remains the default.

See [`artifacts/manifest.md`](artifacts/manifest.md) for commands, provenance,
and packet-local evidence.
