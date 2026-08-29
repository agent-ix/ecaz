---
task: 230
packet: 008-self-contained-owner-sample
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 01
---

# Task 230 self-contained remote-owner sample

Review the correction at `cb6666410` and authorize the preregistered one-step
hot/cold smoke before Packet 004 restarts again.

## Third harness discovery

Packet 007 fixed hot/cold's logical-column mismatch by joining the internal
`a_2` identity to owner-local `dm.source_id` for the raw query vector. The next
fresh restart failed closed in the first row-heap step: the join produced no
valid `identity|vector` sample at the owner-proof phase, so the parser reported
`remote owner sample is malformed` before any benchmark measurement.

The mistake was changing the already-proven row-heap sample and introducing a
dependency on the logical source relation at a phase whose durable contract is
the internal generation relation. Packet 007's review reasoning about ownership
was correct when a join row existed, but the real fixture proved joinability is
not guaranteed there.

## Self-contained correction

- Row heap returns to its exact pre-Packet-007 query:
  `source_id::text || '|' || source::text` from the generation row relation.
- Hot/cold samples `a_2` and casts the same internal hot row's mandatory exact
  vector `a_4::real[]` for the query vector.
- Neither layout joins `dm`; both identity and vector now come from one owned
  generation tuple.
- The exact pinned-owner and returned-candidate probes retain Packet 007's
  layout-selected `source_id`/`a_2` predicates.
- The focused test now pins both full sample SQL shapes and forbids `JOIN dm` in
  hot/cold.

No decision config, threshold, parser, or runtime extension behavior changes.
The failed step emitted no benchmark result.

## Validation and required smoke

- `cargo fmt --check`: exit 0.
- Focused SQL-shape test: 1 passed, 0 failed.
- `cargo clippy -p ecaz-cli --all-targets`: exit 0; 77/78 baseline unchanged.
- `tiny-hotcold-smoke.json`: one fresh real-10k hot/cold, release-guarded,
  stage-counter-only suite step with two latency iterations and isolated id-only
  I/O. It exercises build, topology, serving, both remote-owner proofs, the
  62-row attribution contract, and cleanup without producing decision evidence.
- Suite audit: exit 0, one step. Dry-run config SHA-256:
  `7abf830f498f96ac0211714235635cfd760b10d36e01bcddf1aa7ec97394b9ee`.
- No smoke result exists yet.

## Review request

Please review the self-contained row-heap/hot-cold sample queries and the
one-step smoke config. If DONE, authorize the smoke after release reinstall and
matching CLI build at the accepted head. Packet 004 will restart from empty
step 1 only after that smoke passes and its evidence is review-closed.
