# Review Request: SPIRE Multicluster Smoke/Insert CLI Paths

Code checkpoint: `95054735d2ac4b59022b0c8b03ea2171b4dc66cd` (`Wrap SPIRE multicluster smoke insert CLI paths`)

## Scope

- Adds `ecaz dev spire-multicluster smoke-pg18` for the baseline PG18
  one-coordinator/one-remote smoke fixture.
- Adds `ecaz dev spire-multicluster insert-read-after-customscan-pg18` for the
  repeated INSERT followed by CustomScan read fixture.
- Carries through the shared fixture controls operators need for packet-local
  evidence: artifact directory, run/log/smoke-log paths, ports, run id, pgbin,
  PGRX home, and skip-install.
- Adds an `--insert-mode helper|trigger` CLI enum for the insert/read fixture.
- Adds parser coverage for both new subcommands and records the Phase 12.9
  tracker evidence.

## Validation

- `git diff --check 95054735^ 95054735`
- `cargo test -p ecaz-cli spire_multicluster`
- `cargo check --no-default-features --features pg18`

Packet-local logs are under `artifacts/`; see `artifacts/manifest.md` for
commands and result lines.

## Review Focus

- Confirm these are the right operator-facing names for the two repeated PG18
  multicluster workflows.
- Confirm the wrapper options match the underlying scripts without losing any
  packet-local artifact controls.
- Confirm parse-only coverage is sufficient for this non-live CLI wrapper
  slice; live fixture execution remains gated by the normal operator approval.
