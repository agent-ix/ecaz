# Task 87 Packet 010: CandidateBatch Measurement Toggles

## Summary

This packet asks for review of a narrow measurement-enablement slice for
Task 87.

Code checkpoint under review:

- `6f830a21aec44422fe068d40afb988fcc1025c76` - `Add Task 87 CandidateBatch measurement toggles`

The structural Task 87 routes are already default-on. This slice adds
default-on diagnostic switches so benchmark suites can collect
baseline-vs-change cells from one installed build instead of juggling
source checkouts:

- `ec_spire.candidate_batch_scoring`
- `ec_hnsw.candidate_batch_scoring`

It also lets `ecaz bench suite` pass arbitrary validated
`--session-guc name=value` entries through recall and SPIRE pipeline
steps, matching the latency command's existing support.

## Changes

- Added `ec_spire.candidate_batch_scoring`, default `on`, gating only
  the Task 87 `SpirePreparedAssignmentScorer::score_batch_ip`
  CandidateBatch route. The prior single-candidate no-QJL LUT path is
  not gated.
- Added `ec_hnsw.candidate_batch_scoring`, default `on`, gating only
  HNSW `FullLut` exact-payload CandidateBatch collection during cached
  successor expansion.
- Moved `--session-guc` validation/application helpers from latency to
  shared `bench` support.
- Added `--session-guc` to `ecaz bench recall` and
  `ecaz bench spire-pipeline`.
- Added suite expansion support for `session_gucs` on recall and
  SPIRE pipeline steps.

## Why This Is Task 87 Work

Packets 003, 004, and 006 are structural route packets. They deliberately
do not claim the final real-corpus validation gate. The next Task 87
evidence needs off/on cells for recall, latency, and scoring-share
attribution. These switches make those cells runnable from one current
source install while keeping production behavior default-on.

## Validation

See `artifacts/manifest.md` for packet-local log metadata.

- `artifacts/cargo-test-hnsw-scan.log`
  - `cargo test --lib am::ec_hnsw::scan::tests --no-default-features --features pg18`
  - result: `74 passed; 0 failed`
- `artifacts/cargo-test-spire-quantizer.log`
  - `cargo test --lib am::ec_spire::quantizer::tests --no-default-features --features pg18`
  - result: `15 passed; 0 failed`
- `artifacts/cargo-test-cli-suite.log`
  - `cargo test -p ecaz-cli commands::bench::suite`
  - result: `41 passed; 0 failed`
- `artifacts/cargo-test-cli-latency-session-gucs.log`
  - `cargo test -p ecaz-cli commands::bench::latency::tests::parse_session_gucs`
  - result: `2 passed; 0 failed`

## Review Focus

- Confirm the AM GUCs gate only Task 87 structural CandidateBatch routes
  and preserve default-on behavior.
- Confirm the SPIRE switch does not disable Task 86's prior
  single-candidate no-QJL LUT scorer.
- Confirm suite `session_gucs` passthrough is appropriate for Task 87
  off/on validation cells.
