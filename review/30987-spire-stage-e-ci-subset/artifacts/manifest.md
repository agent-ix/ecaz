# Artifact Manifest: SPIRE Stage E CI Subset

- head SHA: `003fd90608d052e5e668d9fd2af5af552c039e31`
- packet/topic: `30987-spire-stage-e-ci-subset`
- lane: Phase 12a.4 Stage E fault matrix CI wiring
- fixture: GitHub Actions workflow wiring; no local multicluster fault run in
  this packet
- storage format / rerank mode: not applicable
- isolated/shared surface: not applicable
- timestamp: `2026-05-13T15:32:25Z`

## Static Checks

- `python3 -c 'import yaml, sys; yaml.safe_load(open(".github/workflows/ci.yml"))'`
- `cargo test cli_parses_spire_multicluster_fault_command -p ecaz-cli`
- `git diff --check`

## CI Subset

The workflow matrix calls:

```sh
target/debug/ecaz dev spire-multicluster fault-pg18 \
  --case "$CASE" \
  --artifact-dir "target/spire-stage-e-ci/$CASE" \
  --run-id "ci-$CASE"
```

for:

- `remote_statement_timeout`
- `local_cancel`
- `epoch_mismatch`
- `version_skew`

The job only proceeds on pull requests whose changed files match
`src/am/ec_spire/**`, `sql/**`, or `scripts/run_spire_multicluster_*.sh`.
