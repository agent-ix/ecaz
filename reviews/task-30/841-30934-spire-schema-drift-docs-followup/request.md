---
topic: spire-schema-drift-docs-followup
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30934
stage: phase-12.5
status: open
---

# Review Request: SPIRE Schema Drift Docs Follow-Up

## Scope

Please review commit `c9e3817bdc9ee114415964b50df425270a0b17cb`
(`Document landed schema drift guard`).

This docs-only follow-up updates wording left stale after commit `369c50d1`:

- ADR-069 now says the write-path shape fingerprint has landed and names
  `ec_spire_remote_node_descriptor.coordinator_insert_shape_fingerprint`.
- `docs/SPIRE_DIAGNOSTICS.md` now describes the guard as the active
  fail-closed catch-net before remote libpq dispatch, not a planned future
  safety net.
- The Phase 12.5 tracker decision row now says the descriptor-bound
  schema-drift fingerprint is the current safety net for violated DDL ordering.

This addresses the reviewer `30931` P3 note that operator docs should identify
the schema-drift fingerprint catch-net when it lands.

## Review Focus

- Confirm the docs now match the implementation from packet `30933`.
- Confirm the wording still preserves the v1 no-DDL-propagation contract:
  operators must pause writes, apply matching DDL on coordinator and remotes,
  refresh descriptors, then resume writes.

## Validation

Artifacts are packet-local under `artifacts/` and described in
`artifacts/manifest.md`.

- `git diff --check HEAD^ HEAD`
