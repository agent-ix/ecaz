---
agent: codex
role: coder
model: GPT-5
date: 2026-08-21
seq: 1
---

# Task 167 final real-corpus matrix

Status: runtime in progress; review-open, not closeout-ready.

This packet owns the final Task 167 production 10k/50k/100k evidence at code
checkpoint `5568aba17`. The release CLI and PG18 extension were built from a
clean worktree at that checkpoint; packet 036 records their build logs, hashes,
and passing warning-free synthetic gate.

The immutable suite configuration is
[`../032-recovery-runtime/artifacts/task167-recovery-suite.json`](../032-recovery-runtime/artifacts/task167-recovery-suite.json).
The 10k step runs first as a gate. The 50k and 100k steps may run only after
10k succeeds. Every selected real step includes physical concurrency,
insert-then-query parity, append A/B, recall, latency, and storage measurements.

No result is claimed until packet-local suite manifests, results JSONL, and
cited logs are present and summarized in the artifact manifest.

