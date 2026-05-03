# 30353 SPIRE Diagnostic Status Labels

## Request

Review the docs-only follow-up that makes SPIRE diagnostic label strings
explicitly operator-facing and stable.

## Scope

- Added a `Stable Labels` section to `docs/SPIRE_DIAGNOSTICS.md`.
- Documented the current `assignment_payload_status` values from
  `ec_spire_index_options_snapshot(index_oid)`.
- Updated Task 30 status to point at the stable-label documentation.

## Decision

Diagnostic labels should not be reused for a different meaning. Operator tools
can currently treat these assignment payload status values as stable:

- `supported`: current SPIRE scans can score the configured payload format.
- `deferred_model_metadata`: the configured format is recognized, but SPIRE
  still needs additional grouped-PQ model metadata before it can scan that
  format.

Today `supported` covers TurboQuant and RaBitQ; `deferred_model_metadata`
covers PQ-FastScan.

## Validation

- `git diff --check`

Docs-only change; no tests were run.

