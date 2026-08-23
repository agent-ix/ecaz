---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 shipped-only 50k quality diagnosis

Status: measurement complete; the preregistered shipped-only quality gate
failed. Task 167 remains open. No acceptance or closeout is claimed.

Packet 045 reproduced a 50k heldout deficit of `0.026250`, but the measured
physical graph contained both the shipped robust-prune inserts and a later
rejected append-when-room diagnostic arm. Code checkpoint `c3b01290b` changes
the measurement order so exact quality is evaluated after only the 160 shipped
inserts and before the candidate is allowed to mutate the fixture.

This packet preregisters one isolated `ec_real_50k` PG18 fixture with the same
production operating point as packet 045: three owners, degree 32, head cap
4096, beam 4, heap 32, 100 hops, 200 heldout queries, 48 separate
inserted-neighborhood queries, exact fp32 truth, and pinned search GUCs.

The hard bands are fixed from packet 045 before this result is observed:

- inserted-neighborhood maximum deficit: `0.015`;
- heldout maximum deficit: `0.007`.

The command failed as required by the preregistration. After only the 160
shipped-default robust-prune inserts, the 200-query heldout population measured
`0.848722` physical distinct recall against `0.857333` from a fresh rebuild.
The deficit was `0.008611`, exceeding the fixed `0.007000` band by `0.001611`.
The diagnostic append-when-room candidate did not run and did not mutate this
fixture.

This isolates the earlier packet-045 result: removing the candidate-arm
contamination reduced the observed 50k heldout deficit from `0.026250` to
`0.008611`, but it did not make the shipped robust-prune path pass. The
threshold is unchanged. Task 167 therefore remains open for diagnosis of the
residual robust-prune insertion quality loss; a separate 10k/50k/100k final
confirmation is still required after a corrective checkpoint.

The suite exited nonzero at the quality gate, so the rejected candidate arm was
not measured and no packet summary was emitted by the then-current harness.
The original child log remains the source evidence. Commits `6d205bdbb`,
`7b20d18fa`, and `7e3d3d714` subsequently taught report-only extraction to
retain and structure this failed-step metric; `results.jsonl` was regenerated
from that original log without rerunning the benchmark.

Configuration and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
