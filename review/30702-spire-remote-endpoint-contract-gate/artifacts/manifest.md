# Artifact Manifest: 30702 SPIRE Remote Endpoint Contract Gate

Head SHA: `79ea08aa0423dd90fac5aa0d44b51e30bee3dd45`
Packet: `review/30702-spire-remote-endpoint-contract-gate`
Timestamp: `2026-05-09T20:00:49-07:00`

## Scope

- Lane: Task 30 Phase 11.3 remote search endpoint contract.
- Fixture: SQL-visible contract surfaces plus focused PG18 pg_test.
- Storage format: not a storage measurement packet.
- Rerank mode: RaBitQ-only contract wording; PQ/PQFastScan remain unsupported.
- Surface: remote endpoint contract gate, libpq parameter/result contracts, and
  operator entrypoint contract.
- Index isolation: not a benchmark packet; no shared-table or multi-index
  performance claim.

## Validation Commands

| Command | Result |
| --- | --- |
| `cargo fmt` | Passed; existing rustfmt unstable import-grouping warnings printed |
| `cargo test remote_search_endpoint_contract --lib` | Passed: 0 tests run after filtering; library compiled |
| `cargo pgrx test pg18 test_ec_spire_remote_search_receive_contract` | Passed: 1 passed, 0 failed |
| `git diff --check` | Passed |

## Key Result Lines Cited By Request

- `cargo test remote_search_endpoint_contract --lib`: `test result: ok. 0 passed; 0 failed`.
- `cargo pgrx test pg18 test_ec_spire_remote_search_receive_contract`: `test result: ok. 1 passed; 0 failed`.
- `git diff --check` produced no output and exited successfully.
