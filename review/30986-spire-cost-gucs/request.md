# Review Request: SPIRE Cost Tuning GUCs

- coder: coder1
- code commit: `01005d7995413feeb37ca0c145346634f411dfe4`
- tracker rows: Phase 12a.3 Cost-Constant GUCs

## Scope

This slice converts the SPIRE planner cost constants into session-tunable GUCs
and surfaces the live values through a narrow diagnostic snapshot:

- `ec_spire.cost_routing_dimension_scale`
- `ec_spire.cost_leaf_dimension_scale`
- `ec_spire.cost_index_page_scale`
- `ec_spire.cost_local_store_page_fanout_scale`
- `ec_spire.cost_storage_scoring_multiplier`
- `ec_spire.cost_rerank_multiplier`

`ec_spire_index_cost_tuning_snapshot(index_oid)` reports the active GUC values
plus effective storage/rerank multipliers. The existing
`ec_spire_index_cost_snapshot(index_oid)` keeps its modeled-cost shape to avoid
growing an already-wide pgrx tuple. `ecaz bench spire-pipeline` gained
`--include-cost-snapshot` and matching `--cost-*` flags so the benchmark
connection can set and report the tuning values in one run.

The storage scoring GUC is a scalar over the existing calibrated
storage-format baseline, so packet `30976` modeled rows still reproduce under
defaults while AWS/RDS recalibration can adjust the active scalar.

## Evidence

Artifact manifest:
`review/30986-spire-cost-gucs/artifacts/manifest.md`

Live override fixture:
`review/30986-spire-cost-gucs/artifacts/spire-pipeline-cost-guc-override.log`

Key fixture row:

- `routing_dim=0.020000`, `leaf_dim=0.030000`, `page=2.000000`,
  `store_fanout=0.100000`, `storage_guc=1.500000`, `rerank_guc=2.000000`

Validation:

- `cargo test packet_30976_default_tuning_preserves_legacy_modeled_costs --lib`
- `cargo test non_default_tuning_changes_modeled_costs --lib`
- `cargo test storage_scoring_guc_scales_format_baseline --lib`
- `cargo test spire_pipeline_ -p ecaz-cli`
- `cargo test explain_sql_uses_spire_profile_gucs_and_cost_snapshot -p ecaz-cli`
- `cargo build -p ecaz-cli`
- `cargo fmt --check`
- `git diff --check`

## Reviewer Focus

- Does the sibling tuning snapshot satisfy the active-value requirement without
  overgrowing `ec_spire_index_cost_snapshot`?
- Is preserving the storage-format baseline plus a GUC scalar the right balance
  for "no default behavior change" and Phase 13 recalibration?
- Is the `ecaz bench spire-pipeline --include-cost-snapshot` fixture sufficient
  for the requested non-default override proof?
