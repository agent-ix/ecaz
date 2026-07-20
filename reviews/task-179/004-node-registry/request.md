# Review request — Task 179 authenticated node registry

**Status:** review requested; checkpoint 004, not Task 179 closeout.

**Branch:** `task-179-ec-distann-physical-shards`  
**Code checkpoint:** `17bb92b2759cd4f879d23e1c35b01d3aca2cad22`

## Outcome

This checkpoint replaces caller-asserted registry identity with authenticated,
durable participant identity and separates the mutable desired roster from
immutable per-build transport bindings. It also closes the stale-snapshot and
mid-replacement deadlock paths found during the registry review.

## Implementation under review

- Add insert-only participant identity configuration and extend the secured
  control-identity endpoint with the compatibility digest, configured endpoint,
  and server-canonical index locator.
- Restrict endpoint, secret-reference, and locator text to injective canonical
  v1 grammars. Resolved conninfo remains in memory; stored rows contain only the
  secret reference and authenticated returned identity.
- Bind compatibility to reloptions, row-schema fingerprint, indexed vector
  attnum, exact extension-owned IP opclass/kind, identity attnum/type/NOT-NULL,
  and index validity/readiness/liveness.
- Treat `ec_distann_node_descriptor` as the desired next-build roster, add a
  monotonically increasing registry revision, and add private immutable
  build-participant bindings plus unambiguous build identity on publication and
  active-pointer catalogs.
- Retain the coordinator `ShareRowExclusiveLock` to transaction end so one
  `unregister → register` replacement cannot deadlock with a waiter. All
  registry operations use relation-lock then registry-row-lock order.
- Make unregister one guarded mutation: an in-progress build binding blocks it,
  while Published/retained bindings survive desired-roster removal. Conditional
  revision advancement prevents stale success.
- Harden every current DistANN `SECURITY DEFINER` path as
  `pg_catalog, extension schema, pg_temp`, with `pg_temp` explicit and last.
- Make captured row-tier columns physically nullable, while preserving source
  type/typmod/collation/attnum/dropped-slot identity and enforcing NOT NULL on
  the distributed source-identity attribute itself.

The accompanying FR-078/FR-082/NFR-014 revisions also freeze the missing owner
digest, empty-owner sequence-zero, physical receipt digest/byte-accounting, and
active build-binding contracts needed by checkpoint 005. Those are normative
pre-work; this packet does not claim stage/seal implementation.

## Failure and race coverage

- Exact participant configuration replay and conflicting reconfiguration.
- Duplicate ordinal/node/endpoint/participant/local entries.
- Raw conninfo and former blocklist counterexamples, provider aliases, missing
  secrets, endpoint mismatch, noncanonical locator, key-attnum drift, custom
  opclass, nullable identity, and reloption incompatibility.
- Sanitized connection and remote-query failures plus real libpq loopback UUID
  provenance.
- In-progress unregister rejection, Published-binding preservation, desired
  replacement, and registry-revision accounting.
- A real two-backend mid-replacement waiter that would deadlock under the old
  early-unlock behavior; the waiter instead observes the committed roster.
- Repeatable Read stale mutation returns SQLSTATE `40001` and preserves the
  committed replacement.
- Actual unprivileged endpoint/catalog denial and a granted positive call with
  attacker-schema function plus temporary relation/type shadows.

## Validation

- Full filtered DistANN PG18 run: 161 passed, 0 failed, 1 explicit golden
  emitter ignored.
- Independent DistANN persisted/wire fixture slice: 12 passed, 0 failed.
- Strict PG18 all-target clippy: clean with warnings denied.
- Specification grammar: 244/244 clean.
- Traceability/error-category audit: zero missing/duplicate/unexpected rows;
  implementation matrix remains honestly partial.

## Reviewer focus

1. Verify relation-lock retention and registry-row serialization across
   multi-call replacement transactions, including error/subtransaction paths.
2. Verify unregister cannot delete a captured in-progress binding or report
   success after a stale/concurrent delete, while Published bindings remain
   usable.
3. Verify compatibility identity and canonical locator/endpoint provenance are
   sufficient and do not persist caller-only or secret transport material.
4. Verify `SECURITY DEFINER` search-path, ACL, and sanitized remote failures are
   fail-closed under the attacker/temp fixtures.
5. Verify desired-roster edits cannot alter active/retained build routing and
   that the new catalog foreign keys bind active identity consistently.

## Deliberately open

No owner batch is staged or sealed by this checkpoint. Publication, physical
read path, true three-instance topology, and required 10k/50k/100k A/B evidence
remain open under later Task 179 packets.
