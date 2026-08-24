---
task: 222
packet: 004-full-scale-decision
agent: Codex
role: coder
model: gpt-5
date: 2026-08-23
seq: 01
---

# Task 222 full-scale payload-projection decision

Please review the required three-owner PG18 10k/50k/100k recall, warm-latency,
payload, attribution, storage, and topology matrix for implementation
`c9f79be4a`.

The candidate wins at every scale with unchanged recall and byte-identical
ordered predictions:

- 10k: 15.7 to 9.52 ms/scan (-39.36%);
- 50k: 16.8 to 10.8 ms/scan (-35.71%);
- 100k: 17.4 to 11.6 ms/scan (-33.33%).

The standard query's exact mask is id-only at every scale. Payload bytes fall
by at least 99.9459%, owner payload SQL work falls by 93.30%-94.21%, tails
improve, and storage is arm-identical. Every topology has three owners, zero
non-owned records, zero orphan vectors, no coordinator-resident unsharded
payload, bounded graph amplification, and one immutable generation per A/B.
The candidate is NFR-021 conforming and the comparisons satisfy NFR-022.

The suite executed 10k and 50k in its first invocation. Its proposed 100k
fixture reuse was rejected before measurement because the earlier correctness
run had intentionally executed a routed-delete drill. The final 100k row was
therefore rebuilt and measured on a fresh external fixture with `--only`; its
manifest reports success with no missing/stale artifacts. No N-1 fixture was
used as benchmark evidence.

`artifacts/decision.md` contains the decision table and
`artifacts/manifest.md` routes every cited value to packet-local evidence.
Task 222 implementation and evidence are complete; packets 002-004 remain
review-open for an outside verdict.
