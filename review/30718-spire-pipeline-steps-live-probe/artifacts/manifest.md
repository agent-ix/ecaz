# Artifacts Manifest: 30718 SPIRE Pipeline Steps Live Probe

Head SHA: `3d4232c9b27bd44ce9ab10304c3e31b18b24becc`
Packet: `review/30718-spire-pipeline-steps-live-probe`
Timestamp: `2026-05-10T01:13:50-07:00`

## Artifacts

| Artifact | Lane | Fixture / Surface | Storage Format | Rerank Mode | Isolated Surface | Command | Key Result |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `cargo-check-pg18.log` | PG18 compile | extension compile | n/a | n/a | n/a | `cargo check --no-default-features --features pg18` | `Finished dev profile ... target(s) in 0.12s` |
| `cargo-fmt-check.log` | format | repository formatting | n/a | n/a | n/a | `cargo fmt --check` | command exited `0`; stable rustfmt emitted existing unstable-option warnings |
| `cargo-pgrx-pg18-loopback-pipeline.log` | PG18 pgrx | loopback remote executor pipeline test | `rabitq` | strict remote executor path | one-index-per-table coordinator/remote loopback fixture | `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty` | `test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok`; `1 passed; 0 failed; 1523 filtered out` |
| `cargo-pgrx-pg18-policy-contracts.log` | PG18 pgrx | remote operator entrypoint contract | n/a | n/a | n/a | `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts` | `test tests::pg_test_ec_spire_remote_phase7_policy_contracts ... ok`; `1 passed; 0 failed; 1523 filtered out` |
| `git-diff-check.log` | diff hygiene | working diff | n/a | n/a | n/a | `git diff --check` | command exited `0` |
