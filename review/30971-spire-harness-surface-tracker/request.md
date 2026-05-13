# Review Request: SPIRE Harness Surface Tracker Closure

Code checkpoint: `0b88b9ff65ef56632c53b6101fb8a00cac481658` (`Close SPIRE phase 12 harness surface rows`)

## Scope

- Closes the Phase 12.9 tracker parent row for `ecaz dev
  spire-multicluster` fixture wrappers.
- Closes the Phase 12.9 tracker parent row for `ecaz bench spire-pipeline`
  distributed recall, latency, and counter capture.
- Updates the `ecaz-cli` README command tree to include the accepted
  `smoke-pg18` and `insert-read-after-customscan-pg18` wrappers alongside the
  existing CustomScan read, transport-overlap, fault, and lifecycle wrappers.
- Leaves the two remaining Phase 12.9 rows open: live packet-local artifact
  capture and the final production-readiness bundle.

## Validation

- `git diff --check 0b88b9ff^ 0b88b9ff`

Packet-local log is under `artifacts/`; see `artifacts/manifest.md` for the
command and result line.

## Review Focus

- Confirm reviewer feedback from packets 30967 and 30968 is sufficient to close
  the CLI and bench harness surface parent rows.
- Confirm the tracker still keeps live artifact capture and final bundle work
  visibly open.
