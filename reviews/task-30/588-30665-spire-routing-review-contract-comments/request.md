# Review Request: SPIRE Routing Contract Comments

This small follow-up addresses non-blocking reviewer feedback from packets
30662 and 30663 before starting the next Phase 9 slice.

Code checkpoint: `2af95e89` (`Document SPIRE routing review contracts`)

## Scope

- Documents that `SpireTopGraphGreedyView` borrows the scan-owned top-graph
  snapshot only for the duration of the greedy traversal.
- Documents the current `nprobe_per_level` interaction: it remains local
  per-parent exploration, while leaf-level effective `nprobe` drives the global
  beam until an explicit beam reloption lands.
- Documents the recursive path-score contract: inner-product scores accumulate
  across routing levels, and top-graph distances are converted back to scores
  before recursive descent.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 route_recursive --lib`

## Review Focus

- Confirm these comments resolve 30662 F1 and 30663 F2/F3 without overstating
  behavior.
- Confirm no further code behavior change is expected from those findings.
