---
task: 206
packet: 002-100k-sweep
agent: codex
role: coder
model: gpt-5
date: 2026-08-03
seq: 002
---

# Review request: pre-registered 100k traversal sweep

Head SHA: `59aeb6c58fa3e2f0db1774a6c3c8a5ab62308e78`

`artifacts/task206-100k-sweep.json` is the canonical `ecaz bench suite`
configuration for Task 206 phase 2. It registers nine independent arms:
BW × H = {32, 64, 128} × {4, 5, 8}, with `top_k=200`, `head_seed_count=200`,
and the monolithic head control (`build_shards=1`). The fixture run directory
is outside the repository under `$ECAZ_CLUSTER_ROOT`.

The suite audit passed and the dry-run emitted all nine expanded commands;
those outputs are recorded in `artifacts/suite-dry-run.md`. The first run
stalled during physical setup, and its diagnostic is retained in
`artifacts/setup-attempt.md`. A retry completed the full physical matrix and
single-index control after release preflight. Results are summarized in
`artifacts/result-summary-100k.md` and the structured source is
`artifacts/run-100k-retry/results.jsonl`.

At 100k, BW64/H8 achieved recall `0.9584` at p50 `187.7 ms`; BW128/H8
achieved recall `0.9700` at p50 `209.5 ms`. Physical storage was invariant at
`2,496,659,456` bytes across the nine arms. The 3-owner topology and serving
checks passed with zero non-owned rows and zero orphans.

Please review the pre-registration and leave findings under this packet's
`feedback/` directory.
