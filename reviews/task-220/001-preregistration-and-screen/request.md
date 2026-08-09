---
agent: codex
role: coder
model: gpt-5
date: 2026-08-09
seq: 1
---

# Task 220 — MAT-16 preregistration and isolated screen

The implementation checkpoint is `9bc0b05eb`. This packet preregisters one
100k production lazy-10 control/candidate screen before inspecting results.
The immutable generation, query surface, graph/search settings, locator mode,
owner-plan cache setting, and materialization batch size are shared. The only
arm difference is `packed_payload`: the control retains per-row `bytea[]`
construction; the candidate emits null flags, cumulative offsets, and one flat
`bytea` payload buffer.

The SuiteConfig is `artifacts/task220-mat16-100k.json`. It requires the
same-generation recall pair, materialization correctness, recall, warm
latency, storage, stage counters, owner payload/endpoint attribution, and
NFR-021/NFR-022 conformance. The run directory is outside the repository at
`/home/peter/.ecaz/clusters/task220-mat16-packed-payload-100k`.

The pre-registered decision rule is: stop without a 10k/50k/100k matrix if
the candidate is recall-unsafe, semantically non-identical, storage-invalid,
non-conforming, or neutral/regressive on the end-to-end latency contract;
otherwise continue to packet 003 for the standard matrix.

Please review the preregistration and implementation checkpoint before the
screen is run.
