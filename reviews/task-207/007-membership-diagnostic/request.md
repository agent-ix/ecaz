---
agent: claude
role: coder
model: gpt-5
date: 2026-08-04
seq: 1
---

# Task 207 membership diagnostic

This follow-up answers the reviewer’s open question without changing the
construction decision. It runs one 10k stitched control and one 10k
partition-union candidate with identical physical topology and captures the
active head IDs into packet-local `physical-head-membership.json` artifacts.
The existing physical prediction outputs are then compared offline to those
sets for head-membership@k and head-set overlap.

This is diagnostic evidence only. It does not promote `partition_union`, alter
the shipped policy, or change the release seed interpretation.

The capture code checkpoint is `8eea5f965`; the lifecycle-test digest fix is
`482a34e56`. Results and
provenance are in `artifacts/manifest.md`; the compact comparison is in
`artifacts/membership-analysis.md`.
