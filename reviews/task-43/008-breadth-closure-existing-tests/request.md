# Task 43 Review Request: Breadth Closure for Existing Pure Tests

## Summary

This packet closes the campaign tracker rows that were already backed by small
pure tests but were not first-class Miri coverage:

- DiskANN Vamana build helper coverage now has bounded Miri tests for graph
  connectivity/stats and pass-1 extra candidate pools.
- DiskANN vacuum now runs mark-delete, strip-dead-primary, fully-dead, repair,
  and deletion state-machine tests under the `miri_vc_` prefix.
- HNSW search now covers stale-frontier removal, fully stale frontier
  exhaustion, forget/reseed ordering, and visible-frontier select/refill under
  Miri.
- SPIRE top-k now covers ranked leaf dedupe, primary tie-breaks, scored
  candidate tie ordering, rerank prefix replacement, invisible candidates, and
  non-finite rerank rejection under Miri.
- SPIRE routing now covers root/internal/top-graph/recursive routing and the
  root mismatch, internal-parent, missing-child, wrong-level, and conservative
  nprobe paths under Miri.

This is not a completion packet. The tracker remains authoritative and still
marks remote typed payload validation, SPIRE delete-delta/vacuum visibility,
SPIRE careful mirroring, mutation probes, and final audit as open.

## Code Under Review

Code commit: `2a4d09c97132ca685342a149a019f3a923a56ed2`

Changed files:

- `src/am/ec_diskann/vamana.rs`
- `src/am/ec_diskann/vacuum.rs`
- `src/am/ec_hnsw/search.rs`
- `src/am/ec_spire/scan/tests/candidates.rs`
- `src/am/ec_spire/scan/tests/runtime_state.rs`
- `src/am/ec_spire/scan/tests/routing.rs`

## Validation

Artifacts are packet-local under `artifacts/`; see
`artifacts/manifest.md` for commands and key result lines.

- `miri-diskann-build.log`: 2 passed; 0 failed.
- `miri-diskann-vacuum.log`: 8 passed; 0 failed.
- `miri-hnsw-beam.log`: 4 passed; 0 failed.
- `miri-hnsw-visible-frontier.log`: 2 passed; 0 failed.
- `miri-spire-routed-rank.log`: 2 passed; 0 failed.
- `miri-spire-scored-candidate.log`: 1 passed; 0 failed.
- `miri-spire-rerank.log`: 3 passed; 0 failed.
- `miri-spire-candidate-cursor.log`: 1 passed; 0 failed.
- `miri-spire-routing.log`: 9 passed; 0 failed.
- `careful-harness-cargo-test.log`: 69 passed; 0 failed.
- `cargo-fmt-check.log`: exit 0.
- `git-diff-check.log`: exit 0.

## Tracker Update

`reviews/task-43/001-coverage-survey-strategy/artifacts/campaign-tracker.md`
has been updated in this packet to mark the rows closed by this breadth slice
and to keep the remaining completion gates open.
