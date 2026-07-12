# Packet 014 — epoch topology (by fingerprint) artifact manifest

Task bucket: `reviews/task-179/`; packet `014-epoch-topology/`.
Surface: `ec_distann_epoch_topology` + `build_topology_row` refactor
(`src/am/ec_distann/handoff.rs`). Isolated one-index pgrx surface.

## Commit under review
- `d284190b03360af5a18424e14c44fd0478944751` — feat(distann): add ec_distann_epoch_topology by-fingerprint inspection.

## Artifacts
| File | Head SHA | Command | Key result |
|---|---|---|---|
| `pgrx-epoch-topology.log` | `d284190b03360af5a18424e14c44fd0478944751` | `cargo pgrx test pg18 --no-default-features --features pg18 test_distann_build_epoch_single_node` | `test result: ok. 1 passed` — Published epoch resolved by fingerprint reports the generation (3 records); unknown valid-version fingerprint -> EC_GENERATION_MISSING; bad version -> EC_EPOCH_FINGERPRINT_VERSION |

`cargo check` + strict clippy (`pg18 pg_test`, `-D warnings`) pass at `d284190b03360af5a18424e14c44fd0478944751`.
