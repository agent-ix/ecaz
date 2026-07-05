# Review Request: SPIRE Spec Backfill

## Scope

This docs checkpoint backfills the landed SPIRE design into the formal spec
tree. It replaces the earlier flat draft SPIRE FR set with a nested functional
spec organized by storage formats, local lifecycle, distributed execution, and
operations.

## Changed Spec Surface

- Adds `spec/functional/spire/FR-048..FR-060` covering the SPIRE domain model,
  partition-object formats, Leaf V2, routing/delta/top-graph objects, local
  build/search/maintenance flows, topology and placement, typed remote
  transport, production remote executor, CustomScan distributed reads,
  coordinator DML/2PC, and diagnostics.
- Adds `US-022` for local SPIRE index lifecycle operation.
- Adds `NFR-013` and `NFR-014` for local readiness/capacity boundaries and
  transport/security operations.
- Updates `spec/spec.md`, `spec/tests.md`, `StR-005`, `US-018..US-020`,
  `FR-029`, `NFR-012`, and the ADR index to point at the landed SPIRE shape.
- Removes the obsolete duplicate flat SPIRE draft files `FR-038 SPIRE` through
  `FR-043` and the duplicate SPIRE `US-017`.

## Review Focus

1. Confirm the nested split is coherent and sufficiently reproducible:
   domain, storage, local lifecycle, distributed read/write, and operations.
2. Check that wire/data formats are specific enough for an independent
   implementation to decode objects and remote tuple payloads.
3. Check that process specs include sequence/flow/state diagrams and that
   deferred SPIRE claims are explicit rather than implied.
4. Check for stale references to the superseded flat SPIRE FR IDs.

## Validation

- `rg` stale-reference checks for old SPIRE FR/US IDs: clean.
- duplicate requirement ID check: clean.
- `git diff --check`: clean.

No runtime tests were run. This checkpoint changes requirements and
traceability docs only.
