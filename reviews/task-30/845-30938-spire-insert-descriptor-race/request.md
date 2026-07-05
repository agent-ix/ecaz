---
topic: spire-insert-descriptor-race
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30938
stage: phase-12.4
status: open
---

# Review Request: SPIRE Insert Descriptor Race

## Scope

Please review commit `121cc46bae02850fcd9c41d6c4452ea88ae41070`
(`Pin SPIRE insert descriptor race behavior`).

This slice closes the Phase 12.4 concurrent INSERT descriptor-generation race
fixture:

- Adds `test_ec_spire_insert_descriptor_race_sql`.
- The fixture creates isolated loopback coordinator and remote SPIRE indexes,
  rewrites a positive leaf to a remote node, registers a stale descriptor
  generation, and enables coordinator-routed INSERT.
- It holds one coordinator INSERT transaction open after the remote prepare and
  descriptor refresh, then drives a second same-descriptor INSERT from another
  backend.
- It asserts the loser reaches its own remote PREPARE before blocking on the
  descriptor refresh, then observes the documented `40001`
  `serialization_failure` retry path after the winner commits.
- It asserts the losing prepared remote transaction rolls back, only the
  winner's remote row is visible, only the winner's placement row is published,
  no SPIRE prepared xacts remain, and the descriptor generation advanced once.
- It changes the coordinator insert descriptor refresh miss from a generic
  internal error to the same stable SQLSTATE/message/detail contract used by
  `ec_spire_register_remote_node_descriptor`.
- It updates ADR-069 and the Phase 12 tracker for the now-pinned behavior.

## Review Focus

- Confirm the fixture is exercising a real two-backend descriptor race rather
  than a single-backend stale descriptor path.
- Confirm the accepted v1 policy is correct: one INSERT wins, concurrent losers
  get SQLSTATE `40001` with whole-write retry guidance.
- Confirm remote prepared transaction cleanup and placement publication checks
  are strong enough to catch loser leakage.

## Validation

Artifacts are packet-local under `artifacts/` and described in
`artifacts/manifest.md`.

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_insert_descriptor_race_sql`

Key result: `1 passed; 0 failed; 1689 filtered out`.
