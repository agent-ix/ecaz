# Review Request: SPIRE Libpq Operations Runbook

Code checkpoint: `fd4ea9d3` (`Add SPIRE libpq operations runbook`)

## Scope

- Adds `docs/SPIRE_LIBPQ_RUNBOOK.md`.
- Covers the Phase 12.9 libpq security and operations checklist:
  - `sslmode` and libpq security option preservation;
  - raw-conninfo non-exposure;
  - sanitized auth/certificate/conninfo failure handling in strict and degraded
    modes;
  - `max_prepared_transactions` readiness;
  - orphaned prepared xact recovery;
  - credential-rotation deferral;
  - audit-log schema deferral.
- Links the runbook from `docs/SPIRE_DIAGNOSTICS.md` and
  `docs/SPIRE_LOCAL_READINESS.md`.
- Marks the Phase 12.9 libpq security/ops runbook row complete.

## Validation

- `git diff --check fd4ea9d3^ fd4ea9d3`

Packet-local log is under `artifacts/`; see `artifacts/manifest.md` for the
command and result line.

## Review Focus

- Confirm the runbook names each required Phase 12.9 security/ops item without
  overclaiming a full TLS, credential-rotation, or audit subsystem.
- Confirm the strict/degraded failure wording is compatible with existing
  sanitized diagnostic labels and does not expose raw conninfo or remote error
  text.
