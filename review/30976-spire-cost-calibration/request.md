# Review Request: SPIRE CustomScan Cost Calibration

- coder: coder1
- code/evidence commit: `515731b40134cfed18a92bab47a84216511c06c5`
- tracker rows: Phase 12.3 CustomScan cost calibration and packet-local
  benchmark logs

## Scope

This packet calibrates the distributed SPIRE CustomScan cost constants against
the local PG18 loopback fixture from packet `30975`.

Changed constants in `src/am/ec_spire/custom_scan.rs`:

- remote dispatch CPU units: `32.0` -> `1024.0`
- merge CPU units: `4.0` -> `0.5`
- added projected tuple-width cost: `0.001` CPU units per projected byte

The goal is conservative planner costing, not a product benchmark claim. The
new model keeps fixed remote dispatch as the dominant term for a one-remote
loopback read, while preserving monotonic cost growth for remote fanout, remote
placements, output rows, and projected tuple width.

## Evidence

Artifact manifest:
`review/30976-spire-cost-calibration/artifacts/manifest.md`

Final latency matrix:
`review/30976-spire-cost-calibration/artifacts/calibrate-spire-cost-final.log`

Post-change modeled-cost rows:
`review/30976-spire-cost-calibration/artifacts/customscan-cost-model-after.log`

Key measured rows:

- `id_only 1 10 ... p50_ms=31.674`
- `id_only 16 100 ... p50_ms=42.521`
- `title_body 1 10 ... p50_ms=31.775`
- `title_body 16 100 ... p50_ms=42.454`

Key modeled rows:

- `id_only k=10`: startup `2.600000`, total `2.837700`
- `id_only k=100`: startup `2.600000`, total `4.977000`
- `title_body k=10`: startup `2.600000`, total `2.841875`
- `title_body k=100`: startup `2.600000`, total `5.018750`

Validation:

- `cargo test custom_scan_cost`
- `cargo fmt --check`
- `git diff --check`

## Reviewer Focus

- Do the constants match the local measurement evidence without overclaiming
  beyond this fixture?
- Is the projected-width term appropriately small relative to fixed dispatch?
- Are the Phase 12.3 tracker rows closed with enough packet-local evidence?
