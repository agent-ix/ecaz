# Review Request: SPIRE Stage E CI Subset

- coder: coder1
- code commit: `003fd90608d052e5e668d9fd2af5af552c039e31`
- tracker rows: Phase 12a.4 Stage E Fault Matrix CI Wiring

## Scope

This slice wires a lightweight Stage E fault subset into CI for pull requests
touching SPIRE surfaces:

- `src/am/ec_spire/**`
- `sql/**`
- `scripts/run_spire_multicluster_*.sh`

The workflow runs four existing `ecaz dev spire-multicluster fault-pg18` cases:

- `remote_statement_timeout`
- `local_cancel`
- `epoch_mismatch`
- `version_skew`

`version_skew` is the pre-dispatch incompatible-version blocker. The remaining
fault and lifecycle cases stay operator-runnable through the same wrapper
surface and remain archived in packet `30895`.

Docs now record the CI vs operator-runnable evidence boundary in
`docs/SPIRE_LOCAL_READINESS.md`, and the Phase 12 tracker line names the exact
CI subset.

## Evidence

Artifact manifest:
`review/30987-spire-stage-e-ci-subset/artifacts/manifest.md`

Validation:

- `python3 -c 'import yaml, sys; yaml.safe_load(open(".github/workflows/ci.yml"))'`
- `cargo test cli_parses_spire_multicluster_fault_command -p ecaz-cli`
- `git diff --check`

## Reviewer Focus

- Is the selected subset the right PR-gated balance for Stage E bitrot risk?
- Does the workflow path filter match the intended SPIRE-touching surface?
- Is the documented split clear enough that the remaining 7 fault cases and
  lifecycle matrix are not accidentally treated as CI-gated?
