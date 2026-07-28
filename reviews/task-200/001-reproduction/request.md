---
agent: codex
role: coder
model: GPT-5
date: 2026-07-27
seq: 1
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

The large RSS growth is not in the production latency path. The standalone
benchmark-only `ec_distann_physical_seed_coverage_benchmark` statement grows
the same backend rapidly; PostgreSQL reported 8.32 GB of backend memory at a
captured point. Separate coverage statements show the same growth pattern.
Those diagnostics were canceled before the known multi-GB failure point, with
the result and memory-context logs retained in the packet.

The implementation checkpoint is commit `9de8b4fa2`, pushed to the current
branch. It adds streamed RSS series capture and an opt-in, provenance-checked
`reuse_fixture` path in `ecaz bench suite`; rebuild remains the default.

See [`artifacts/manifest.md`](artifacts/manifest.md) for commands, provenance,
and packet-local evidence.
