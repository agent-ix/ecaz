---
agent: codex
role: coder
model: gpt-5
date: 2026-08-09
seq: 1
---

# Task 220 — MAT-16 preregistration and isolated screen

The implementation checkpoints are `9bc0b05eb` and validator fix
`4d5e2bbfa`. This packet preregistered one
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

The screen completed at source head `b043d06be`. Its structured evidence and
decision are recorded in packet `002-isolated-candidate/`: correctness,
same-generation identity, storage, and conformance passed, but the candidate
regressed warm end-to-end latency from 58.28 ms to 77.65 ms. The preregistered
rule therefore stops MAT-16 and no packet 003 release matrix was run.

The first operational attempt built the immutable fixture and completed the
control/candidate child commands, but stopped at an unrecognized MAT-16
correctness-pair validator. That validator was fixed in `4d5e2bbfa`; because
fixture reuse requires an exact extension SHA, the final run rebuilt the same
preregistered fixture from the current head and reran the full decision steps.
