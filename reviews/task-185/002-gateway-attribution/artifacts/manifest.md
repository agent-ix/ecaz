# Task 185 packet 002 artifact manifest

- Head SHA: `23154d722eee818df1ef4b086b1e76d1d7ceb58e`
- Task / packet: `task-185` /
  `reviews/task-185/002-gateway-attribution/`
- Recorded: `2026-07-23T07:58:44-07:00`
- Lane: PG18, `distann-head-attribution-benchmark`
- Benchmark state: no recall/latency/storage measurement is claimed by this
  packet. Packet 003 owns the suite-driven 100k screen.

## Frozen input contract

- Evaluation: rows 1-200 of
  `/home/peter/dev/ecaz/data/staged-current/ec_real_100k_queries.tsv`.
- Training: rows 201-400 of the same ordered file.
- Policy-selection validation: rows 401-600 of the same ordered file.
- File SHA-256:
  `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.
- File row count: 1,000.
- The gateway builder loads only training and validation. Evaluation outcomes
  are unavailable to it.

## Frozen policy cells

1. Control membership + control selector:
   `training_landmarks` + `head_sample_exact`.
2. Gateway membership + control selector:
   `training_gateway_set_cover` + `head_sample_exact`.
3. Control membership + diversity selector:
   `training_landmarks` + `head_basin_diverse`.
4. Gateway membership + diversity selector:
   `training_gateway_set_cover` + `head_basin_diverse`.

Every cell fixes cap 4,096, exact scoring of the persisted head, 32 returned
seeds, graph degree 32, BW4/H100, RaBitQ neighbor scoring, and exact final
ranking.

The gateway builder limits the candidate set to each training query's 32
RaBitQ-nearest owner nodes. It runs the production orchestration core from each
candidate independently at BW4/H100/top-10, records reached exact-truth pairs
and expanded-node overlap, and chooses membership by deterministic lazy greedy
set cover. Frequency membership fills any unused cap slots.

The diversity selector exact-scores the same head, freezes the nearest
`8 * seed_count` window, derives a query-conditioned 32-wide traversal basin
from the persisted head graph, and minimizes:

`exact_rank + 32 * max_jaccard_with_selected_basins`

This changes only returned-seed selection; membership and traversal remain
separately controllable.

## Validation artifacts

| Artifact | Command | Key result |
|---|---|---|
| `clippy-pg18.log` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo clippy --lib --no-default-features --features pg18,distann-head-attribution-benchmark -- -D warnings` | PASS |
| `gateway-set-cover-test.log` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test --lib --no-default-features --features pg18,distann-head-attribution-benchmark gateway_set_cover_is_deterministic_and_uses_marginal_gain` | 1 passed |
| `basin-diverse-test.log` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test --lib --no-default-features --features pg18,distann-head-attribution-benchmark basin_diverse_selector_reduces_same_component_overlap` | 1 passed |
| `cli-suite-policy-test.log` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test --manifest-path crates/ecaz-cli/Cargo.toml distann_local_multinode_expands_benchmark_training_policies` | 1 passed |
| `cli-attribution-parser-test.log` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test --manifest-path crates/ecaz-cli/Cargo.toml distann_task185_attribution_rows_are_structured` | 1 passed |

The validation commands do not create a physical index and therefore have no
one-index-per-table/shared-table distinction.
