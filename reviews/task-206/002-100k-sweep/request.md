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

Head SHA: `55a730d80e20d17177d23fb9f7246382665e37ed`

`artifacts/task206-100k-sweep.json` is the canonical `ecaz bench suite`
configuration for Task 206 phase 2. It registers nine independent arms:
BW × H = {32, 64, 128} × {4, 5, 8}, with `top_k=200`, `head_seed_count=200`,
and the monolithic head control (`build_shards=1`). The fixture run directory
is outside the repository under `$ECAZ_CLUSTER_ROOT`.

The suite audit passed and the dry-run emitted all nine expanded commands;
those outputs are recorded in `artifacts/suite-dry-run.md`. No latency, recall,
or storage result is claimed yet. The archived 100k corpus path is present,
but the actual multinode fixture/extension run has not been executed on this
host.

Please review the pre-registration and leave findings under this packet's
`feedback/` directory.
