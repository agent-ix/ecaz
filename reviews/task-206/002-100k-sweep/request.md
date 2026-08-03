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
those outputs are recorded in `artifacts/suite-dry-run.md`. An actual run was
started after installing the release extension for this SHA and passed the
release preflight, but it stalled during physical setup while loading/building
the 100k coordinator corpus before any benchmark rows were produced. The
diagnostic is recorded in `artifacts/setup-attempt.md`; no latency, recall, or
storage result is claimed.

Please review the pre-registration and leave findings under this packet's
`feedback/` directory.
