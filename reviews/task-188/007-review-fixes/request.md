---
task: 188
packet: 007-review-fixes
role: coder
status: open
date: 2026-07-27
head: 0a270a4b3
---

# Review request: address packet 006 findings

This checkpoint addresses all actionable findings from packet 006 feedback
sequences 02 and 03:

- the shared latency worker replays untimed warmup after every reconnect;
- `worker_batch_size` is emitted in direct latency rows and distann physical
  latency result rows;
- latency suite configs can reach `worker_batch_size`, including a suite
  default;
- packet 006's reconnect-contaminated mean/p95/p99 claims are qualified and
  p50 is the only latency comparison cited from that run;
- the explicit batch-10 remote-candidate result and its deduplication/batching
  interpretation are promoted;
- the pre-refactor/default-path equivalence check is recorded in the packet
  artifacts and shows no material movement.

The backend-growth finding is not reimplemented here: reviewer sequence 03
split it to Task 200 because the growth occurred on the existing main
extension/backend path, while Task 188 changes only CLI/benchmark harness
code.

Code commit: `2d9f6099b` (`fix(bench): rewarm reconnect batches and emit provenance`).
Packet/documentation commit: `0a270a4b3` (`docs(review): qualify reconnect latency evidence`).

Validation:

- `cargo test -p ecaz-cli expands_latency_with_cache_state_label` — passed;
- `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo check -p ecaz-cli` — passed, with one unrelated pre-existing dead-field warning;
- pre-refactor `1426c838b` vs review-fix `2d9f6099b` equivalence at PG18 HNSW 10k, `ef_search=64`, 30 queries, batch size zero — mean 2.56 vs 2.57 ms, p50 2.38 vs 2.38 ms, p95 2.81 vs 2.82 ms.

See `artifacts/equivalence.md` and `artifacts/manifest.md` for provenance and
packet-local raw evidence. This request remains open for outside review.
