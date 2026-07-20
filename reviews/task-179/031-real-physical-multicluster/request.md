---
task: 179
packet: 031-real-physical-multicluster
role: coder
status: review-requested
head: 2aabb45f9e3abcc058980c7308666897bc7ba6e8
date: 2026-07-12
---

# Review request: real physical multicluster fixture

Please review code commit `2aabb45f9e3abcc058980c7308666897bc7ba6e8` and the
suite-driven evidence in `artifacts/`.

## Scope

- Replaces the primary `local-multinode-pg18` lane with a physical build: source rows load
  only on the coordinator, participants receive schema-compatible empty shells, and streamed
  handoff creates one disjoint owner generation.
- Keeps the old replicated fixture only under the explicit
  `replicated-serving-control-pg18` subcommand.
- Resolves participant topology by retained fingerprint rather than a coordinator-only
  publish-decision row.
- Makes production physical source capture available outside `pg_test` builds.
- Allows zero or one local roster owner. A zero-local coordinator loads the immutable build
  candidate and head sample locally, then expands and materializes every hit remotely.
- Adds explicit remote-owner row proofs: a frozen row sampled from each remote owner must be
  reconstructed by the coordinator CustomScan under an identity qual.
- Adds suite parsing and a numeric topology gate so absence or failure prevents accepted
  downstream evidence.

## Evidence and result

The canonical suite starts real PG18 processes for all three required cases:

1. One-owner degenerate: 30 Ready/Published records and rows, zero residue/orphans.
2. Three owners, coordinator in roster: disjoint 33/24/33 ownership, global count 90,
   `remote_verified=2`.
3. Three owners, coordinator outside roster: four PostgreSQL processes, no coordinator-local
   generation, disjoint 33/24/33 ownership, `remote_verified=3`.

Every step exited 0 and every suite threshold passed. See `artifacts/manifest.md`,
`artifacts/suite-manifest.json`, and `artifacts/results.jsonl`.

## Validation

- `cargo check -p ecaz-cli`: pass (one pre-existing warning).
- Focused suite parser unit test: pass.
- `ecaz bench suite audit`: pass, three steps.
- Canonical fixture suite: pass at exact clean head SHA.

## Review focus

- Zero-local coordinator lifetime/scan-token behavior and absence of a local frozen-row slot.
- Candidate descriptor validation versus any optional local generation.
- Physical fixture roster construction, especially coordinator-outside-roster ordinal mapping.
- Suite gate completeness and whether the remote-owner proof excludes local-heap fallback.

This is the implementation-ready correctness fixture, not Task 179 closeout. The task remains
open for the required immutable 10k/50k/100k A/B recall, latency, and storage evidence and
outside-reviewer acceptance.

