# Artifact Manifest

Packet: `30707-spire-endpoint-diagnostic-contract`
Head SHA: `16f3f781d6d536dc5eab31ab47ea24c6295d93d9`
Timestamp: `2026-05-10T04:21:15Z`

## cargo-pgrx-pg18-remote-search-receive-contract.log

- Command: `script -q -c "cargo pgrx test pg18 test_ec_spire_remote_search_receive_contract" /home/peter/dev/ecaz/review/30707-spire-endpoint-diagnostic-contract/artifacts/cargo-pgrx-pg18-remote-search-receive-contract.log`
- Lane: Phase 11 Stage B endpoint contract diagnostics.
- Fixture: PG18 SQL-visible contract surface, no shared-table remote fixture.
- Storage format: contract-only; no index storage read path.
- Rerank mode: contract-only; no rerank execution.
- Surface shape: endpoint contract and libpq result contract surfaces.
- Key result line: `test tests::pg_test_ec_spire_remote_search_receive_contract ... ok`
- Key result line: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1516 filtered out`
