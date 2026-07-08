# Task 164 M2 — placement + epoch + endpoint (slices 1–2b) manifest

## Provenance

- **Branch / head SHA:** `task-164-ec-distann-m2` @ `1a14eee0e` (stacked on
  `task-163-ec-distann-m1`).
- **Nature:** code-review packet, no benchmark (M2's measured H×RTT deliverable
  lands with the transport slice 2c/2d). Test evidence only.
- **Host:** Intel desktop, PG18 (pgrx test harness spins its own temp instance).

## Files under review

| File | Slice | What |
|------|-------|------|
| `src/am/ec_distann/placement.rs` | 1 | FR-078 hash placement + topology directory |
| `src/am/ec_distann/epoch.rs` | 2a | FR-082 epoch fingerprint (M2 subset) |
| `src/am/ec_distann/roster.rs` | 2b-config | roster/epoch GUCs + pure parser |
| `src/am/ec_distann/remote_endpoint.rs` | 2b | FR-079 `ec_distann_expand_nodes` SRF + fingerprint helper |
| `src/am/ec_distann/routine.rs` | 2b | `indexed_ecvector_attnum` exposed `pub(super)` |
| `src/tests/ec_distann_basic.rs` | 2b | 3 endpoint pg_tests |

## Commands

    # pure unit/proptests
    cargo test --no-default-features --features pg18 --lib \
      am::ec_distann::placement am::ec_distann::epoch am::ec_distann::roster
    # endpoint pg_tests
    cargo pgrx test pg18 --no-default-features --features pg18 expand_nodes
    # lint
    cargo clippy --lib --no-default-features --features pg18

## Result (cited in `test-evidence.log`)

- **placement** (FR-078): 6 tests green — determinism (AC-1), within-epoch
  stability (AC-3), <10% load imbalance across 3 nodes at 100k (AC-2), grouping
  covers-all-in-order, single-node degenerate, `resolve` bounds.
- **epoch** (FR-082 subset): 5 tests green — determinism, per-field
  sensitivity (all 10 identity fields), roster-order sensitivity, length-prefix
  anti-aliasing, bytea round-trip.
- **roster**: 5 tests green — empty/whitespace, two-node parse, tolerant
  separators, malformed/duplicate rejection, placement-order preservation.
- **endpoint** (FR-079): 3 pg_tests green —
  - `..._single_node_matches_local`: one row per requested owned vec_id
    (AC-1), nearest exact_dist ≈ −1.0 for the `[1,0,0,0]` fixture row (AC-5),
    no tombstones, aligned neighbor arrays.
  - `..._rejects_epoch_mismatch`: a wrong 16-byte fingerprint yields the
    retriable epoch-mismatch error, never data (AC-2).
  - `..._rejects_nonowned_placement`: under a 2-node roster (this instance =
    node 0), a node-1-owned vec_id yields a placement error (AC-3 case b).
- **clippy**: clean (`-D warnings` posture; no warnings).

## Not yet covered here (slices 2c/2d)

- FR-079-AC-4 (neighbor code distances equal direct QuantCodec scoring): the
  endpoint delegates to `LocalNodeExpander`, which the M0 `distann` block-kernel
  tests already cover; a direct endpoint-level assertion lands with the
  two-node fixture.
- Owned-but-absent (c) / vector-missing (d) distinct structural faults: raised
  by `LocalNodeExpander` today; the two-node fault fixture (M3 territory for the
  full drill matrix) exercises the distinct codes.
- Two-node top-k identity (TC-040/041) and measured H×RTT vs the D4 reopen
  trigger: the transport slice.
