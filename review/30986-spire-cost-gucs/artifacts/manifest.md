# Artifact Manifest: SPIRE Cost GUCs

- head SHA: `01005d7995413feeb37ca0c145346634f411dfe4`
- packet/topic: `30986-spire-cost-gucs`
- lane: Phase 12a.3 cost-constant GUCs
- fixture: local PG18 pgrx database `spire_phase12_measure`, single local
  `ec_spire` index `phase12a_cost_guc_idx`
- storage format: `auto`
- rerank mode: effective `rerank_width = 0`
- isolated/shared surface: isolated one-index table for this packet
- timestamp: `2026-05-13T15:32:25Z`

## Artifacts

### `setup-cost-guc-fixture.sql`

Creates the packet-local four-row corpus, one-query table, and
`phase12a_cost_guc_idx` used by the override fixture.

Command used:

```sh
target/debug/ecaz dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --db spire_phase12_measure --raw --file review/30986-spire-cost-gucs/artifacts/setup-cost-guc-fixture.sql
```

The setup was run equivalently through `--sql` before this file was added for
durable packet replay.

### `spire-pipeline-cost-guc-override.log`

Command used:

```sh
target/debug/ecaz bench spire-pipeline --host /home/peter/.pgrx --port 28818 --database spire_phase12_measure --prefix phase12a_cost_guc --index phase12a_cost_guc_idx --queries-limit 1 --sweep 1 --rerank-width 0 --include-cost-snapshot --cost-routing-dimension-scale 0.02 --cost-leaf-dimension-scale 0.03 --cost-index-page-scale 2.0 --cost-local-store-page-fanout-scale 0.10 --cost-storage-scoring-multiplier 1.5 --cost-rerank-multiplier 2.0 --log-output review/30986-spire-cost-gucs/artifacts/spire-pipeline-cost-guc-override.log
```

Key result lines:

- `cost_snapshot: true`
- `routing_dim ... 0.020000`
- `leaf_dim ... 0.030000`
- `page ... 2.000000`
- `store_fanout ... 0.100000`
- `storage_guc ... 1.500000`
- `rerank_guc ... 2.000000`

## Static Checks

- `cargo test packet_30976_default_tuning_preserves_legacy_modeled_costs --lib`
- `cargo test non_default_tuning_changes_modeled_costs --lib`
- `cargo test storage_scoring_guc_scales_format_baseline --lib`
- `cargo test spire_pipeline_ -p ecaz-cli`
- `cargo test explain_sql_uses_spire_profile_gucs_and_cost_snapshot -p ecaz-cli`
- `cargo build -p ecaz-cli`
- `cargo fmt --check`
- `git diff --check`
