# SPIRE Scale Packet Runbook

## Scope

This packet prepares the controlled scale-packet runbook and artifact manifest.
It does not make a product-scale claim.

Code checkpoint: `2ce7e477` (`Add SPIRE old epoch cleanup packet`)
Local preflight checkpoint: `9f9869c0` (`Fix SPIRE suite rerank width CLI`)

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
- `target/debug/ecaz bench suite audit --config crates/ecaz-cli/suites/task30-spire-real10k.json`
  - `[suite:task30-spire-real10k] audit passed: 5 steps`
- `cargo test -p ecaz-cli ec_spire_profile_uses_spire_opclass_and_raw_real_scan_query`
  - `test profiles::tests::ec_spire_profile_uses_spire_opclass_and_raw_real_scan_query ... ok`
- `cargo build -p ecaz-cli`
- Local PG18 preflight artifacts are recorded in `artifacts/manifest.md`.

## Notes

The Phase 8 scale-packet task remains open. This packet exists so the eventual
measurement run has the required packet-local artifact structure before any
claim is made.

The local preflight used the real10k fixture on the pgrx PG18 scratch cluster
with `recursive_fanout=2` and `nprobe_per_level=2`. It verifies command
readiness and the packet-local artifact flow, but it is not an AWS/RDS-class
scale measurement and does not close the Phase 8 scale item.
