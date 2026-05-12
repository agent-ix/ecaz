---
topic: spire-custom-private-metadata
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30940
stage: phase-12.3
status: open
---

# Review Request: SPIRE DML Custom Private Metadata

## Scope

Please review commit `6d8a544b4fced947c981f86905c1dca359cd4f70`
(`Replace SPIRE DML custom private JSON metadata`).

This slice closes the Phase 12.3 `custom_private` cleanup item:

- Replaces DML CustomScan updated/projected column metadata encoded as JSON
  string payloads with a flat PostgreSQL node layout.
- The new layout stores each column list as a count `String` node followed by
  one `String` node per column. Empty lists are represented by a zero count.
- Keeps mode, index OID, and PK column as string nodes, avoiding the earlier
  mixed OidList/string-list copyObject issue while removing JSON parsing from
  plan-private column metadata.
- Updates the Phase 12 tracker with the copyObject evidence.

## Review Focus

- Confirm the counted string-node layout is copyObject-safe and easier to
  reason about than JSON payload strings in `custom_private`.
- Confirm updated/projected column offsets remain decoded correctly when either
  list is empty or non-empty.
- Confirm the tracker wording does not overclaim broader JSON retirement; this
  only cleans DML plan-private metadata, not tuple payload transport.

## Validation

Artifacts are packet-local under `artifacts/` and described in
`artifacts/manifest.md`.

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_custom_scan_dml_plan_private_copyobject_sql`

Key result: `1 passed; 0 failed; 1687 filtered out`.
