# Packet 012 — publish decision (T3) artifact manifest

Task bucket: `reviews/task-179/`; packet `012-decide-epoch-publish/`.
Surface: `ec_distann_decide_epoch_publish` + `load_build_candidate`
(`src/am/ec_distann/build_coordinator.rs`). Isolated one-index pgrx surface.

## Commit under review
- `2ada9f9a20610f0750af24738090fb6f3cae6b97` — feat(distann): add ec_distann_decide_epoch_publish (T3 publish decision).

On `task-179-ec-distann-physical-shards` (participant lifecycle integrated at 907150c03).

## Artifacts
| File | Head SHA | Command | Key result |
|---|---|---|---|
| `pgrx-decide-epoch-publish.log` | `2ada9f9a20610f0750af24738090fb6f3cae6b97` | `cargo pgrx test pg18 --no-default-features --features pg18 test_distann_build_epoch_single_node` | `test result: ok. 1 passed` — after build-to-Ready, decide returns the candidate manifest digest, persists one Pending decision (no predecessor, non-empty successor activation), leaves the active pointer unswapped, and replays idempotently |

`cargo check` + strict clippy (`pg18 pg_test`, `-D warnings`) pass at `2ada9f9a20610f0750af24738090fb6f3cae6b97`.
