---
task: 230
packet: 008-self-contained-owner-sample
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 02
---

# Task 230 self-contained remote-owner sample and smoke result

Review the completed preregistered one-step hot/cold smoke at accepted head
`177aae194930e0d2958cca02d197daef55277958`. If DONE, review-close Packet 008
and authorize Packet 004 to restart from an empty step 1.

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

## Static validation and preregistration

- `cargo fmt --check`: exit 0.
- Focused SQL-shape test: 1 passed, 0 failed.
- `cargo clippy -p ecaz-cli --all-targets`: exit 0; 77/78 baseline unchanged.
- `tiny-hotcold-smoke.json`: one fresh real-10k hot/cold, release-guarded,
  stage-counter-only suite step with two latency iterations and isolated id-only
  I/O. It exercises build, topology, serving, both remote-owner proofs, the
  62-row attribution contract, and cleanup without producing decision evidence.
- Suite audit: exit 0, one step. Dry-run config SHA-256:
  `7abf830f498f96ac0211714235635cfd760b10d36e01bcddf1aa7ec97394b9ee`.
- Packet 008 seq-01 was review-closed DONE and authorized this smoke in
  `feedback/2026-08-29-01-reviewer.md`.

## Smoke result

- The release extension was reinstalled and the matching CLI built at accepted
  head `177aae194930e0d2958cca02d197daef55277958` before execution.
- Suite status: `completed=1 failed=0 skipped=0 dry_run=0 missing_artifacts=0
  stale=0`; the step exit code is 0.
- Release preflight: three nodes unanimous, `extension_build_profile=release`,
  `debug_override=false`, and extension SHA `177aae194...`.
- Both non-coordinator owners passed custom-scan, pinned-sample, and exact-owner
  identity checks. The topology gate reports `pass=true`, three owners, two
  remote owners verified, and 10,000 source rows.
- The raw latency artifact contains exactly 62
  `[distann-materialization-work]` rows: 61 server counters plus
  `client_result_rows`. This exercises the Packet 005 invariant end to end.
- Isolated id-only tier I/O reports `pass=true`, 20 result rows, zero cold-tier
  accesses, 66/66 hot-tier hits/accesses, and shared-buffer hit ratio 1.0.
- The suite captured the durable manifest/results and removed
  `/home/peter/.ecaz/clusters/task230-packet008-hotcold-smoke`.
- This stage-counter-only smoke is validation evidence, not Packet 004 decision
  evidence.

## Review request

Please review the smoke receipts and cleanup. If DONE, review-close Packet 008
and authorize the unchanged Packet 004 config/policy to restart from an empty
step 1 after the required release reinstall, matching CLI build, and final entry
gates at the then-accepted head.
