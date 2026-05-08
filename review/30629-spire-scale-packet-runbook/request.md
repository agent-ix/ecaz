# SPIRE Scale Packet Runbook

## Scope

This packet prepares the controlled scale-packet runbook and artifact manifest.
It does not make a product-scale claim.

Code checkpoint: `2ce7e477` (`Add SPIRE old epoch cleanup packet`)

## Runbook

1. Provision an AWS/RDS-class environment and record instance/storage settings
   in `artifacts/manifest.md`.
2. Use a clean database per AM or an isolated one-index-per-table layout.
3. Run the configured SPIRE suite from
   `crates/ecaz-cli/suites/task30-spire-real10k.json`, or a larger checked-in
   scale config if one is added before the measurement run.
4. Store raw logs under this packet's `artifacts/` directory.
5. Update `artifacts/manifest.md` with head SHA, command lines, timestamps, and
   key result lines before citing results in `request.md`.

## Required Commands

The exact command set depends on the target host and configured dataset, but
the packet must include these lanes:

- load
- storage
- explain/planner cost
- latency
- recall

## Files

- `plan/tasks/30-spire-ivf-foundation.md`
- `review/30629-spire-scale-packet-runbook/artifacts/manifest.md`

## Validation

- `git diff --check`

## Notes

The Phase 8 scale-packet task remains open. This packet exists so the eventual
measurement run has the required packet-local artifact structure before any
claim is made.
