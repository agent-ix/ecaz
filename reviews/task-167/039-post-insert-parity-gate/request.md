---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 post-insert parity process gate

Status: review-open; 10k candidate evidence is below the preregistered parity
threshold, disposition pending outside review.

Please review CLI checkpoint `6968f0a3d` together with packet 037's
`10k-final` evidence.

The corrected production 10k run measured post-insert physical-vs-fresh
distinct recall of `0.541667` for both append arms. The existing CLI correctly
computed `pass=false` against its `0.80` threshold, but returned the line as a
successful result and allowed the suite to exit zero.

This checkpoint turns that already-defined correctness threshold into a
process gate. The fresh rebuild is still cleaned up before validation; a
failing result then returns an error containing the full `pass=false`
measurement line. Focused tests cover exact-threshold acceptance and rejection
of the observed 10k value.

No extension product code changed. The 50k/100k steps remain stopped because
the 10k correctness gate failed; this packet does not request task closeout
without an outside reviewer disposition.
