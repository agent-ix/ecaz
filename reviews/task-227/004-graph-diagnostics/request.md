---
task: 227
packet: 004-graph-diagnostics
agent: Codex
role: coder
model: gpt-5
date: 2026-08-24
seq: 01
---

# Task 227 persisted-graph diagnostic tooling

This packet requests review of code checkpoint `5864619a6`. It completes the
distributed persisted-graph portion of Task 227's P0 diagnostic-tooling gate
without changing production scan or storage behavior.

Two benchmark-feature-only, paginated SQL endpoints stream the minimal
persisted adjacency shape `(owner, vec_id, tombstone, neighbors)` for physical
retained generations and the monolithic control. The participant path reads
only graph tuples: it neither fetches nor detoasts exact row-tier vectors. Both
paths validate on-disk records, use stable 64-bit identities, and cap each SQL
chunk at 4,096 rows.

The CLI analyzer emits one deterministic JSON document with the same summary
shape for each physical owner, the physical aggregate, and the monolithic
control. It reports stored/live/tombstone counts; directed, local, remote, and
cross-owner stitch edges; invalid, duplicate, and self edges; SCC and weak
components; zero/min/p50/p95/p99/max in/out degree; exact iterative
articulation/bridge candidates; deterministic adjacency SHA-256; and directed
reachability from the persisted head plus owner-specific head subsets. The
artifact also carries physical epoch identity and monolithic build parameters
and relation bytes; the monolithic adjacency digest is its diagnostic build
identity.

`ecaz dev distann-multicluster --graph-diagnostic` requires the physical lane
and a monolithic control, validates feature availability and record counts,
rejects duplicate stable ids, and writes
`physical-graph-diagnostic.json`. `ecaz bench suite` validates and expands the
option and declares the artifact for compact and full packet modes.

Validation in `artifacts/` passed:

- graph/analyzer and suite tests: 4 passed;
- feature-gated PG18 extension compilation: passed;
- live physical plus monolithic PG18 callback: 1 passed.

The repository-wide strict clippy target remains red on existing unrelated
warnings (including `ecaz-cloud` argument-count and CLI-wide MSRV/dead-code
findings); new Task 227 MSRV and redundant-async findings exposed by that run
were corrected before this checkpoint. `git diff --check` passes.

This is diagnostic-only tooling, so no 10k/50k/100k closeout claim is made.
The frozen 100k graph comparison and query/truth classification remain packet
005 work.

Please review graph-only payload isolation, signed pagination, component and
edge definitions, iterative articulation/bridge safety, seed-set semantics,
physical/monolithic metadata, and suite artifact gating.
