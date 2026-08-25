---
task: 227
packet: 003-query-trace
agent: Codex
role: coder
model: gpt-5
date: 2026-08-24
seq: 01
---

# Task 227 bounded per-query residual trace

This packet requests review of code checkpoint `a9f66b120`. It implements the
second P0 diagnostic prerequisite without changing production scan behavior.

The existing Task 185 feature-only seed tracer now captures a bounded Task 227
query trace: seed locators and code scores; per-round requested, returned, and
live exact-input locators; retained approximate frontier in rank order;
threshold and heap saturation; frontier stability and score gaps; owner fanout;
request/response bytes; terminal exact-ranked input; final top-k ids; round
count; and termination reason. Stable vector ids are emitted as fixed-width
hexadecimal strings and vector payloads are never recorded. Capture is capped
at 65,536 locators with an explicit `truncated` flag.

Trace activation remains behind `distann-head-attribution-benchmark`; all scan
hooks are compile-elided without that feature. A new JSONB endpoint serves the
ordinary active coordinator generation, with a fingerprint-addressed form for
participant diagnostics. Attempt reset semantics discard a failed replica
trace before an owner retry, and origin accounting safely handles seed lists
wider than the 32-bit provenance mask.

The DistANN multinode runner exposes `--query-trace`. It records one
`physical-<variant>-query-trace.json` artifact over the exact configured
evaluation slice, embeds the query id/range/offset plus parent and slice hashes,
and preserves generation identity inside every trace. `ecaz bench suite`
validates the physical-only contract, expands the flag, and declares the trace
artifacts for compact and full packets.

Validation is in `artifacts/`:

- bounded capture, query/attempt reset, and >32-seed safety: 1 passed;
- round/frontier/exact/final trace semantics: 1 passed;
- production-feature traversal equivalence: 1 passed;
- CLI validation, expansion, slice, and expected-artifact contract: 3 passed;
- real PG18 callback across physical build, persisted head, traversal, exact
  containment, and JSON return: 1 passed.

This checkpoint is diagnostic-only, so the task-wide 10k/50k/100k recall,
latency, and storage closeout matrix does not apply yet. Instrumented trace runs
remain separate from clean production-latency runs as required by the plan.

Please review the capture bound/reset semantics, round/frontier definitions,
exact-input versus final-result distinction, feature-disabled path, JSON
identity/provenance, and suite artifact contract.
