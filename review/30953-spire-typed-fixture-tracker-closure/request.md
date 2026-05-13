# Review Request: SPIRE Typed Fixture Tracker Closure

Code checkpoint: `abcc295b` (`Close SPIRE typed fixture tracker row`)

## Scope

- Marks the Phase 12.2 typed-transport fixture parent row complete.
- Adds evidence text pointing to packets `30915`, `30916`, `30917`, and
  `30918`.
- Leaves the remaining Phase 12.2 rows open for endpoint negotiation,
  CustomScan typed binary receive, tuple-heavy throughput measurement, and
  production JSON retirement.

## Existing Evidence Cited

- `30915`: scalar JSON-parity fixture for `bigint` and `text` typed payload
  bytes through `ec_spire_remote_search_tuple_payload_typed(...)`.
- `30916`: NULL and `text[]` fixture using out-of-band `payload_nulls` and
  PostgreSQL `array_send(...)`.
- `30917`: domain and named composite fixture using domain metadata, base
  `textsend(...)` bytes, and `record_send(...)` for named composites.
- `30918`: empty requested projection fixture proving typed metadata/value
  arrays remain aligned without JSON fallback.

## Validation

- `git diff --check abcc295b^ abcc295b`

Packet-local log is under `artifacts/`; see `artifacts/manifest.md` for the
command and result line.

## Review Focus

- Confirm the cited packets are sufficient to close only the fixture parent
  row.
- Confirm the tracker wording does not imply negotiation, CustomScan receive,
  throughput measurement, or JSON retirement are complete.
