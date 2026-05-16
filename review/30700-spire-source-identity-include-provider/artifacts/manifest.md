# Artifact Manifest: 30700 SPIRE Include Source Identity Provider

Head SHA: `fcdd8938a20c2eba5e362a055f5acbe72586a40c`
Packet: `review/30700-spire-source-identity-include-provider`
Timestamp: `2026-05-09T19:43:02-07:00`

## Scope

- Lane: Task 30 Phase 11.2 writer-side global vector identity.
- Fixture: local unit tests and PG18 pgrx tests.
- Storage format: Leaf V2 global fixed-width vec-id storage plus row-encoded
  delta assignments.
- Rerank mode: not a rerank measurement packet.
- Surface: AM reloptions, AM build/insert callbacks, writer identity
  diagnostics, and remote receive validation.
- Index isolation: not a benchmark packet; no shared-table or multi-index
  performance claim.

## Validation Commands

| Command | Result |
| --- | --- |
| `cargo fmt` | Passed; existing rustfmt unstable import-grouping warnings printed |
| `cargo test source_identity --lib` | Passed: 5 passed, 0 failed |
| `cargo test remote_candidate_batch_validation --lib` | Passed: 3 passed, 0 failed |
| `cargo pgrx test pg18 test_ec_spire_srcid` | Passed: 6 passed, 0 failed |
| `cargo pgrx test pg18 test_ec_spire_include_requires_srcid_reloption` | Passed: 1 passed, 0 failed |
| `git diff --check` | Passed |

## Key Result Lines Cited By Request

- `cargo test source_identity --lib`: `test result: ok. 5 passed; 0 failed`.
- `cargo test remote_candidate_batch_validation --lib`: `test result: ok. 3 passed; 0 failed`.
- `cargo pgrx test pg18 test_ec_spire_srcid`: `test result: ok. 6 passed; 0 failed`.
- `cargo pgrx test pg18 test_ec_spire_include_requires_srcid_reloption`: `test result: ok. 1 passed; 0 failed`.
- `git diff --check` produced no output and exited successfully.
