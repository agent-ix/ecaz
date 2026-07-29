---
agent: codex
role: coder
model: gpt-5
date: 2026-07-26
task: 38
packet: 007-m5-mutation-controls
status: review-requested
---

# Review Request: Task 38 M5 Mutation Controls

Review code checkpoint `374166bd3` and the packet-local evidence in
`artifacts/manifest.md`.

## Scope

This checkpoint closes the two M5-verifiable mutation-control gaps recorded in
packet 005:

1. The cancellation negative control arms `ecaz.fault_palloc_nth = 1` in the
   normal cancellation worker. The production cancellation result oracle must
   reject the deliberate AM palloc ERROR because it is not SQLSTATE `57014`.
2. The resource/palloc negative control runs a real AM scan with the same
   palloc fault still armed after the expected ERROR. The normal AM recovery
   probe must reject that unrecovered state; after disarm/reset, the identical
   scan must pass.

Both controls run over all seven isolated fixtures: HNSW, IVF, DiskANN, SPIRE,
and DistANN RaBitQ, TurboQuant, and grouped-PQ. The ordinary memory smoke now
uses the same real AM scan recovery probe after scan/insert/vacuum palloc
failures instead of proving only that `SELECT 1` works.

The checkpoint also adds the repeatable `make fault-mutation-control` operator
and documents both mutation cases in `docs/hardening.md`.

## Evidence

- `artifacts/all-fixtures-mutation-control-live.log`: live local PG18 result at
  code checkpoint `374166bd3`.
- `artifacts/postcondition-audit-live.log`: direct post-run execution of the
  three required shared leak queries.
- `artifacts/static-validation.log`: formatter, diff, build, and clippy result
  summary.
- `artifacts/manifest.md`: command, provenance, hash, counts, and limitations.

## Requested Review

Please verify:

- the cancellation control exercises the production SQLSTATE oracle and cannot
  falsely accept the injected palloc failure;
- the resource control proves rejection before disarm and successful recovery
  after disarm for the same real AM scan;
- all seven fixture markers and final cleanup marker are present;
- the normal memory-smoke recovery strengthening is correct; and
- the packet does not overclaim the still-missing Intel/Linux provider,
  socket, or cgroup evidence.

No AWS, remote host, CI, or Intel execution was used.
