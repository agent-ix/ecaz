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

- 10k: 14.7 to 8.76 ms/scan (-40.41%);
- 50k: 16.8 to 10.8 ms/scan (-35.71%);
- 100k: 17.4 to 11.6 ms/scan (-33.33%).

The standard query's exact mask is id-only at every scale. Payload bytes fall
by at least 99.9459%, owner payload SQL work falls by 93.30%-94.09%, tails
improve, and storage is arm-identical. Every topology has three owners, zero
non-owned records, zero orphan vectors, no coordinator-resident unsharded
payload, bounded graph amplification, and one immutable generation per A/B.
The candidate is NFR-021 conforming and the comparisons satisfy NFR-022.

The packet now contains one successful fresh `--only` suite manifest and
normalized results JSONL per scale. A completion audit reran 10k and 50k after
the original combined invocation's later 100k reuse rejection prevented a
single successful aggregate manifest. The published 10k/50k values come from
those reruns; the 100k row was likewise rebuilt and measured fresh. Every
manifest reports one success, two intentional skips, and no failed,
missing, or stale artifacts. No N-1 fixture was used as benchmark evidence.

`artifacts/decision.md` contains the decision table,
`artifacts/completion-audit.md` maps every acceptance criterion to evidence,
and `artifacts/manifest.md` routes every cited value to packet-local evidence.
Task 222 implementation and evidence are complete; packets 002-004 remain
review-open for an outside verdict.
