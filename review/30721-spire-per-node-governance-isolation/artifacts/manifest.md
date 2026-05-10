# Artifacts Manifest: 30721 SPIRE Per-Node Governance Isolation

Head SHA: `0ff350cf5319c855428c12f6afed808a15cf92bf`
Packet: `review/30721-spire-per-node-governance-isolation`
Timestamp: `2026-05-10T01:39:05-07:00`

## Artifacts

| Artifact | Lane | Fixture / Surface | Storage Format | Rerank Mode | Isolated Surface | Command | Key Result |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `cargo-check-pg18.log` | PG18 compile | extension compile | n/a | n/a | n/a | `cargo check --no-default-features --features pg18` | `Finished dev profile ... target(s) in 0.12s` |
| `cargo-fmt-check.log` | format | repository formatting | n/a | n/a | n/a | `cargo fmt --check` | command exited `0`; stable rustfmt emitted existing unstable-option warnings |
| `cargo-pgrx-pg18-per-node-governance.log` | PG18 pgrx | per-node advisory-lock governance fixture | `rabitq` | degraded receive-attempt diagnostics | one coordinator table with two remote node descriptors | `cargo pgrx test pg18 test_ec_spire_libpq_executor_per_node_governance_isolated` | `test tests::pg_test_ec_spire_libpq_executor_per_node_governance_isolated ... ok`; `1 passed; 0 failed; 1524 filtered out` |
| `git-diff-check.log` | diff hygiene | working diff | n/a | n/a | n/a | `git diff --check` | command exited `0` |
