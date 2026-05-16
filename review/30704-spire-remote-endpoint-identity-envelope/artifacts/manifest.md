# Artifact Manifest: 30704 SPIRE Remote Endpoint Identity Envelope

Head SHA: `c134e4b859c35225edb280ba1558476f48afbad3`
Packet: `review/30704-spire-remote-endpoint-identity-envelope`
Timestamp: `2026-05-09T20:19:47-07:00`

## Scope

- Lane: Task 30 Phase 11.3 remote endpoint candidate-row envelope.
- Fixture: focused PG18 pg_tests for endpoint row shape, libpq receive
  contract, and libpq request planning.
- Storage format: default/auto endpoint rows expose non-ready endpoint status;
  RaBitQ remains the Phase 11 ready serving profile.
- Rerank mode: not a rerank measurement packet.
- Surface: `ec_spire_remote_search`, libpq result contract, endpoint contract,
  and remote-node-model fingerprint documentation.
- Index isolation: not a benchmark packet; no shared-table or multi-index
  performance claim.

## Validation Commands

| Command | Result |
| --- | --- |
| `cargo fmt` | Passed; existing rustfmt unstable import-grouping warnings printed |
| `cargo test remote_search_sql_scores_selected_leaf_pids --lib` | Passed: 1 passed, 0 failed |
| `cargo test remote_search_receive_contract --lib` | Passed: 1 passed, 0 failed |
| `cargo test remote_search_libpq_req --lib` | Passed: 2 passed, 0 failed |
| `git diff --check` | Passed |

## Key Result Lines Cited By Request

- `cargo test remote_search_sql_scores_selected_leaf_pids --lib`: `test result: ok. 1 passed; 0 failed`.
- `cargo test remote_search_receive_contract --lib`: `test result: ok. 1 passed; 0 failed`.
- `cargo test remote_search_libpq_req --lib`: `test result: ok. 2 passed; 0 failed`.
- `git diff --check` produced no output and exited successfully.
