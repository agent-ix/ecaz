# 31142: Standards, Coverage, and Hardening Spec Baseline

## Scope

This packet covers the spec-only checkpoint in commit `97046241`.

Changed files:

- `spec/spec.md`
- `spec/tests.md`
- `spec/non-functional/NFR-004-safety-and-stability.md`
- `spec/functional/spire/storage/FR-050-spire-leaf-v2-format.md`
- `spec/functional/spire/storage/FR-051-spire-routing-delta-topgraph-formats.md`
- `spec/functional/spire/distributed/FR-056-spire-remote-endpoint-typed-transport.md`
- `spec/functional/spire/distributed/FR-059-spire-coordinator-dml-2pc.md`

## What Changed

- Added standards alignment targets for ISO/IEC/IEEE 42010, ISO/IEC 25010,
  and ISO/IEC/IEEE 15939 alongside the existing 29148 and IEEE 828 targets.
- Added the local interpretation of those standards, including AC-level
  traceability, relationship-frontmatter migration expectations, and ID
  tombstone/supersession policy.
- Expanded `NFR-004` from a small safety requirement into the Task 34 hardening
  baseline, with general analysis-requirement rules and Ecaz-specific
  application across supply-chain, unsafe/static hygiene, Miri/careful, fuzz,
  model-checking, formal, sanitizer, SQLsmith, Rudra, MIRAI, and PG18 lanes.
- Updated `spec/tests.md` to treat the generated inventory as a correctness
  baseline, not proof by name alone.
- Added `TC-034` for the Task 34 hardening and analysis lanes.
- Downgraded over-broad SPIRE coverage claims from complete to partial where
  AC-level mapping, CustomScan lifecycle proof, Stage E live coverage, or
  product-scale evidence is still missing.
- Tightened SPIRE Leaf V2, routing/delta/top-graph, typed transport, and 2PC
  GID requirements so byte layouts, payload shape, transport identity, and
  repeated prepared-branch uniqueness are specified.

## Review Focus

- Confirm the standards targets are framed as alignment targets rather than
  certification claims.
- Confirm the analysis-requirement rules are general enough to reuse but still
  bind Task 34 to concrete Ecaz evidence lanes.
- Confirm `spec/tests.md` no longer overclaims SPIRE completeness where the
  coverage audit identified partial or indirect proof.
- Confirm the SPIRE format and wire-protocol additions are precise enough for a
  faithful implementation without contradicting the existing implementation.
- Confirm `TC-034` correctly separates pure Rust hardening evidence from PG18,
  pgrx, SPI, libpq, sanitizer, and SQLsmith evidence.

## Validation

- `git diff --check`

No tests were run. This is a docs/spec checkpoint; the matrix now explicitly
records the relevant hardening and live-test gaps instead of treating this
checkpoint as execution proof.
