# SPIRE Unsafe-Comment Burndown Coverage

- Closeout head: `f12dd9816f63068e5f8b56e8e2d76fa5dddaceb6`
- Production SPIRE source cleared: `870` entries across `37` SPIRE source packets.
- Remaining SPIRE source baseline entries: `16`, all in test/helper modules:
  - `src/am/ec_spire/custom_scan/tests.rs`: `7`
  - `src/am/ec_spire/dml_frontdoor/tests.rs`: `9`
- SPIRE module total tracked by the redirect reviews: `886` entries (`870` cleared + `16` test/helper residual).
- Related cross-cutting SPIRE entrypoint packet outside `src/am/ec_spire`: packet 007 cleared `69` `src/lib.rs` relation-boundary entries.

| Packet | File | Entries Cleared |
|---|---:|---:|
| 043 | `src/am/ec_spire/custom_scan/plan_private.rs` | 23 |
| 044 | `src/am/ec_spire/cost/mod.rs` | 22 |
| 045 | `src/am/ec_spire/insert.rs` | 21 |
| 046 | `src/am/ec_spire/coordinator/maintenance.rs` | 20 |
| 047 | `src/am/ec_spire/custom_scan/cost_helpers.rs` | 19 |
| 048 | `src/am/ec_spire/build/drafts.rs` | 19 |
| 049 | `src/am/ec_spire/custom_scan/dml.rs` | 17 |
| 050 | `src/am/ec_spire/custom_scan/begin_exec.rs` | 13 |
| 051 | `src/am/ec_spire/options/mod.rs` | 13 |
| 052 | `src/am/ec_spire/storage/relation_plan.rs` | 14 |
| 053 | `src/am/ec_spire/build/tuples.rs` | 14 |
| 054 | `src/am/ec_spire/coordinator/remote_candidates/fanout.rs` | 12 |
| 055 | `src/am/ec_spire/coordinator/remote_candidates/executor_receive.rs` | 12 |
| 056 | `src/am/ec_spire/update/publish/relation.rs` | 9 |
| 057 | `src/am/ec_spire/coordinator/remote_candidates/write_payload.rs` | 9 |
| 058 | `src/am/ec_spire/coordinator/diagnostics.rs` | 9 |
| 059 | `src/am/ec_spire/build/publish.rs` | 9 |
| 060 | `src/am/ec_spire/custom_scan/tuple_payload.rs` | 6 |
| 061 | `src/am/ec_spire/build/recursive.rs` | 3 |
| 063 | `src/am/ec_spire/coordinator/remote_candidates/fault_matrix.rs` | 3 |
| 064 | `src/am/ec_spire/scan/callbacks.rs` | 4 |
| 066 | `src/am/ec_spire/coordinator/remote_candidates/endpoint_identity.rs` | 4 |
| 067 | `src/am/ec_spire/coordinator/remote_candidates/operator.rs` | 5 |
| 068 | `src/am/ec_spire/coordinator/remote_candidates/pipeline.rs` | 5 |
| 070 | `src/am/ec_spire/coordinator/lifecycle.rs` | 6 |
| 071 | `src/am/ec_spire/coordinator/remote_candidates/libpq_plan.rs` | 7 |
| 072 | `src/am/ec_spire/coordinator/remote_candidates/dispatch.rs` | 7 |
| 073 | `src/am/ec_spire/coordinator/remote_candidates/scan_output.rs` | 24 |
| 074 | `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` | 71 |
| 075 | `src/am/ec_spire/coordinator/snapshots.rs` | 62 |
| 076 | `src/am/ec_spire/coordinator/debug.rs` | 38 |
| 077 | `src/am/ec_spire/dml_frontdoor/mod.rs` | 159 |
| 078 | `src/am/ec_spire/page.rs` | 58 |
| 079 | `src/am/ec_spire/storage/relation_store.rs` | 51 |
| 080 | `src/am/ec_spire/custom_scan/planner.rs` | 37 |
| 081 | `src/am/ec_spire/vacuum/mod.rs` | 34 |
| 082 | `src/am/ec_spire/scan/relation.rs` | 31 |
| **Total** | **SPIRE production source** | **870** |

## Remaining SPIRE Baseline

The remaining `src/am/ec_spire` baseline entries are test/helper-only:

```text
src/am/ec_spire/custom_scan/tests.rs:431
src/am/ec_spire/custom_scan/tests.rs:432
src/am/ec_spire/custom_scan/tests.rs:433
src/am/ec_spire/custom_scan/tests.rs:434
src/am/ec_spire/custom_scan/tests.rs:435
src/am/ec_spire/custom_scan/tests.rs:436
src/am/ec_spire/custom_scan/tests.rs:437
src/am/ec_spire/dml_frontdoor/tests.rs:209
src/am/ec_spire/dml_frontdoor/tests.rs:219
src/am/ec_spire/dml_frontdoor/tests.rs:231
src/am/ec_spire/dml_frontdoor/tests.rs:251
src/am/ec_spire/dml_frontdoor/tests.rs:269
src/am/ec_spire/dml_frontdoor/tests.rs:296
src/am/ec_spire/dml_frontdoor/tests.rs:306
src/am/ec_spire/dml_frontdoor/tests.rs:347
src/am/ec_spire/dml_frontdoor/tests.rs:359
```
