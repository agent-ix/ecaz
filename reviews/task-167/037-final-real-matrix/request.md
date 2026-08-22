---
agent: codex
role: coder
model: GPT-5
date: 2026-08-21
seq: 1
---

# Task 167 final real-corpus matrix

Status: review-open; corrected production 10k measurement completed but failed
the post-insert fresh-rebuild parity gate; 50k/100k stopped.

This packet owns the final Task 167 production 10k/50k/100k evidence. The
passing warning-free synthetic prerequisite remains in packet 036. Packet 038
separates production insert-work evidence from optional query-stage timing at
CLI checkpoint `a49ffd92a`; the clean release CLI and production PG18 extension
are built at exact runtime head `cce839647` and the corrected suite audit passes.

The corrected immutable suite configuration is
[`artifacts/task167-final-real-suite.json`](artifacts/task167-final-real-suite.json).
The 10k step runs first as a gate. The 50k and 100k steps may run only after
10k succeeds. Every selected real step includes physical concurrency,
insert-then-query parity, append A/B, recall, latency, and storage measurements.

The first 10k attempt passed clean production provenance, three-owner build,
published topology, serving, and both remote-owner proofs. Its recall child
also completed. The following latency child failed because the preregistered
`distann_stage_counters=true` switch calls the benchmark-feature-only
`ec_distann_stage_scoring_snapshot()` function, which is intentionally absent
from the required production `pg18` build.

This is not a 10k result: concurrency, parity, append A/B, latency, and storage
did not complete, and 50k/100k remain blocked. The corrected config explicitly
disables benchmark-only query-stage timing while retaining unconditional
production insert-work accounting. It uses fresh external cluster directories;
the corrected 10k run completed every measurement and drill.

The query-level benchmark results were strong: physical and single-control
recall were both `0.9990`; physical mean/p95 latency was `15.70/18.00 ms`; and
cluster graph-side storage amplification was `1.238533x`. Append-when-room was
`1.029029x` the disabled arm, all eight insert-work metrics counted exactly 160
attempts, concurrency passed, and steady retries were zero.

However, the required post-insert comparison against a fresh local rebuild
failed: both the disabled and enabled append arms achieved only `0.541667`
distinct recall, below the fixture's `0.80` correctness threshold. The summary
correctly recorded `pass=false`, but the suite incorrectly exited zero because
the CLI did not propagate that boolean as an error. This packet therefore does
not claim a passing 10k gate. The 50k and 100k steps remain unrun, and the next
checkpoint must enforce the parity result at process level before disposition.
