# Task 39 Planner Cost Mutation Triage

Code checkpoint: `263c36de197454dbcefa387ba84200b9943f61cf`

Initial run:

- Command: `make mutants MUTANTS_MODULE=src/am/common/cost.rs MUTANTS_OUTPUT_DIR=reviews/task-39/015-planner-cost-mutation/artifacts MUTANTS_JOBS=2`
- Result: 61 mutants tested, 18 missed, 37 caught, 6 unviable.
- Raw survivor list: `artifacts/cost.rs.mutants/mutants.out/missed.txt`.

Fix:

- Imported `src/am/common/cost.rs` into `hardening/careful` with pgrx callback glue gated behind `pg17`/`pg18`, so the pure planner model and callback translation helpers are covered without live PostgreSQL.
- Added exact component tests for reltuples selection, linear-only cost, graph startup cost, graph plus linear tail total, graph-covers-index behavior, and compare-type string mapping.
- Removed the redundant `inputs.index_pages > 0.0` branch after the early `<= 0.0` gate, eliminating an equivalent comparison mutant.

Rerun:

- Command: `make mutants MUTANTS_MODULE=src/am/common/cost.rs MUTANTS_OUTPUT_DIR=reviews/task-39/015-planner-cost-mutation/artifacts/rerun MUTANTS_JOBS=2`
- Result: 58 mutants tested, 0 missed, 52 caught, 6 unviable.
- Raw outcome: `artifacts/rerun/cost.rs.mutants/mutants.out/outcomes.json`.

| Initial missed mutant | Verdict | Evidence |
| --- | --- | --- |
| `cost.rs:226:46 replace > with <` | killed | `planner_cost_model_uses_reltuples_when_stats_are_available` distinguishes `reltuples` from `index_pages * 10.0`. |
| `cost.rs:236:46 replace * with /` | killed | `planner_cost_model_uses_reltuples_when_stats_are_available` asserts the linear page cost. |
| `cost.rs:245:39 replace + with -` | killed | `planner_cost_model_accounts_for_graph_and_linear_components` asserts `tree_height + ef_search` graph pages. |
| `cost.rs:247:37 replace * with +` | killed | `planner_cost_model_accounts_for_graph_and_linear_components` asserts graph CPU contribution. |
| `cost.rs:248:48 replace - with +` | killed | `planner_cost_model_accounts_for_graph_and_linear_components` asserts the linear tail. |
| `cost.rs:248:48 replace - with /` | killed | `planner_cost_model_accounts_for_graph_and_linear_components` asserts the linear tail. |
| `cost.rs:249:40 replace * with +` | killed | `planner_cost_model_accounts_for_graph_and_linear_components` asserts linear page cost. |
| `cost.rs:249:40 replace * with /` | killed | `planner_cost_model_accounts_for_graph_and_linear_components` asserts linear page cost. |
| `cost.rs:255:53 replace > with ==` | removed | The branch was redundant after the early non-positive page gate. |
| `cost.rs:255:53 replace > with <` | removed | The branch was redundant after the early non-positive page gate. |
| `cost.rs:255:53 replace > with >=` | removed | The branch was redundant after the early non-positive page gate. |
| `cost.rs:261:79 replace * with +` | killed | `planner_cost_model_accounts_for_graph_and_linear_components` asserts linear CPU scaling. |
| `cost.rs:261:79 replace * with /` | killed | `planner_cost_model_accounts_for_graph_and_linear_components` asserts linear CPU scaling. |
| `cost.rs:261:58 replace * with +` | killed | `planner_cost_model_accounts_for_graph_and_linear_components` asserts linear CPU scaling. |
| `cost.rs:261:58 replace * with /` | killed | `planner_cost_model_accounts_for_graph_and_linear_components` asserts linear CPU scaling. |
| `cost.rs:261:28 replace * with +` | killed | `planner_cost_model_accounts_for_graph_and_linear_components` asserts tuple-count contribution. |
| `cost.rs:263:51 replace + with -` | killed | `planner_cost_model_accounts_for_graph_and_linear_components` asserts final total. |
| `cost.rs:263:37 replace + with -` | killed | `planner_cost_model_accounts_for_graph_and_linear_components` asserts startup plus tail total. |

Residual:

- Final missed mutants: none.
- Final unviable mutants: 6, all recorded in `artifacts/rerun/cost.rs.mutants/mutants.out/unviable.txt`.
- Live pgrx planner callback execution remains outside this packet; it is still governed by the pgrx feasibility decision in `reviews/task-39/013-pgrx-coverage-feasibility/`.
