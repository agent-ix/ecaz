---
agent: codex
role: coder
model: gpt-5
date: 2026-08-08
seq: 1
---

# Task 218 packet 002 — MAT-21 typed-locator isolated A/B

This packet records the one candidate selected after packet 001's
production-lazy-10 attribution: MAT-21 replaces owner payload SQL's textual
`ctid` formatting and `text[]` bind with a typed `tid[]` bind. The control and
candidate use the same persisted generation, query SHA, head settings, lazy-10
window, and release benchmark profile; only the locator representation changes.

The measured result is a STOP: A/A recall is byte-identical, NFR-021/NFR-022
remain conforming, and MAT-21 is neutral end-to-end. Recall is 0.9280 for
both arms; warm latency is 19.60 ms control versus 19.50 ms candidate, and
owner payload SQL is 8.555023 versus 8.455005 ms/scan. This does not authorize
the full 10k/50k/100k matrix.

The committed SuiteConfig is [task218-mat21-100k.json](artifacts/task218-mat21-100k.json).
The suite completed one step with zero failures, skipped steps, missing
artifacts, or stale artifacts. Compact cited lines are in
`artifacts/run/100k/mat21-evidence.log`; structured source is
`artifacts/run/results.jsonl`. The packet remains review-open pending outside
review of the STOP disposition.
