---
task: 229
packet: 002-format-and-lifecycle
agent: Codex
role: coder
model: gpt-5
date: 2026-08-26
seq: 02
---

# Task 229 format/lifecycle — checkpoint 2 review

Review source commit `255081d74aa6ce430a2a21ee5555e9569c0a0fa7`
against exact main `3419c9c758bea7d9940b27d9afbcf9e627e84879`.
Checkpoint 1 is review-closed DONE in
`feedback/2026-08-26-01-reviewer.md`; its carried items are dispositioned in
`artifacts/seq01-disposition.md`.

This is the second narrow checkpoint of packet 002, not a claim that the
complete format/lifecycle packet is finished. It implements the canonical V1
cover descriptor, compact entry codec, key/identity checks, and the required
registration binding. Generation-descriptor embedding and physical relation
ownership remain subsequent packet-002 work.

## Implemented

- `DistannPayloadCoverDescriptorV1` canonically encodes and strictly decodes
  entry version 1, the format cap, exact sorted attributes and widths, full
  type/typmod/collation/send/receive identity, and row-schema fingerprint.
  Its sidecar-specific domain digest is the registration identity input; no
  existing digest domain changes.
- Descriptor validation rejects version/count/order, NUL/incomplete identity,
  nonempty collation on the fixed noncollatable allowlist, width/type mismatch,
  truncation, and trailing bytes. `validate_row_schema` requires fingerprint
  and per-attribute identity agreement with the frozen row schema.
- The compact payload is exactly `null_bitmap | concatenated_non_null_values`.
  Encoding and decoding enforce count, fixed widths, zero padding bits, exact
  derived length, no truncation/trailing bytes, and the 258-byte ceiling.
- `decode_row` rejects invalid requested TIDs plus row-TID or `vec_id` echo
  mismatch before exposing decoded values.
- Build registration remains byte-identical V1 when no cover is declared—the
  existing golden digest is unchanged. Covered registrations use conditional
  V2 canonical bytes containing the cover-descriptor digest. T1 writes and T1
  replay/T2 replay recompute it, so reloption drift between registration and
  construction fails the durable registration digest check.
- No sidecar relation, generation-descriptor field, receipt, manifest, catalog,
  SQL, read path, or DML path is added in this checkpoint. The heap/index pair
  will land with durable catalog OIDs and lifecycle ownership, never orphaned.

## Validation

- `cargo fmt --all -- --check` — pass.
- `cargo check --lib --no-default-features --features pg18` — pass.
- Focused payload-sidecar tests — 5 passed; descriptor round trip/schema bind,
  compact NULL/width/echo corruption checks, allowlist, exact 258-byte maximum,
  and resolution exclusions.
- Focused registration golden — 1 passed; no-cover digest unchanged and cover
  digest demonstrably moves the registration identity.
- Full strict clippy reports only the four pre-existing main failures in
  `ambuild.rs`, `generation_descriptor.rs`, `head_sample.rs`, and
  `remote_endpoint.rs`; an exact diff proves those files unchanged in this
  checkpoint. Re-running with only those four lint names allowed passes all
  targets under `-D warnings`, so this Task 229 slice adds no lint finding.
- No PostgreSQL, `cargo pgrx test`, fixture, or benchmark command was run.

Durable output and command provenance are in `artifacts/manifest.md`.

## Review questions

1. Is the V1 descriptor canonical, bounded, and sufficiently bound to the
   exact frozen row schema under its sidecar-specific digest domain without
   changing an existing domain?
2. Does the entry codec implement exactly the accepted compact NULL/value
   layout and fail closed on all malformed length, bitmap, TID, and `vec_id`
   cases without adding per-entry metadata?
3. Does conditional registration V1/V2 preserve no-cover bytes while closing
   seq-01's required T1/T2 reloption-drift hole?
4. May checkpoint 3 proceed to generation descriptor V2/V3 plus dual-version
   receipt/manifest/fingerprint/lifecycle-wire persistence?
