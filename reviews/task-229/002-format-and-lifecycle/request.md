---
task: 229
packet: 002-format-and-lifecycle
agent: Codex
role: coder
model: gpt-5
date: 2026-08-26
seq: 01
---

# Task 229 format/lifecycle — checkpoint 1 review

Review source commit `acc33c9f6203a20508005c830d0dc8a8d7b483b7`
against exact main `3419c9c758bea7d9940b27d9afbcf9e627e84879`.
Packet 001 authorized implementation in
`reviews/task-229/001-plan/feedback/2026-08-26-03-reviewer.md`.

This is a narrow first checkpoint of packet 002, not a claim that the complete
format/lifecycle packet is finished. It implements item 1 of the accepted
packet-002 sequence: reloption parsing plus exact fixed-width cover resolution.

## Implemented

- Registered the optional string reloption `covering_payload_attnums`. Absence
  remains `None`; specifying it requires `distributed_control=true`.
- Enforced canonical decimal spelling, positive physical attnums, strict
  increasing/unique order, and the 16-attribute bound.
- Added one resolver over the frozen `DistannRowSchemaDescriptor`. It rejects
  absent, dropped, generated, indexed-vector, missing binary-I/O identity,
  domain/user/array/variable-width, and otherwise unsupported attributes.
- Closed the allowlist to PG18 `pg_catalog` `bool`, `int2`, `int4`, `int8`,
  `float4`, `float8`, `uuid`, `date`, `time`, `timestamp`, and `timestamptz`,
  with fixed widths 1/2/4/8/16 as applicable.
- Resolved state binds the complete schema attribute identity, row-schema
  fingerprint, entry version, maximum attribute count, per-column width, and
  computed maximum payload size. Sixteen UUIDs prove the exact 258-byte bound.
- Both begin-build and build execution resolve the declaration before source
  capture or physical construction. No sidecar relation, persisted descriptor,
  read path, DML path, receipt, manifest, catalog, or SQL change is in this
  checkpoint; those remain subsequent accepted packet-002/003 work.

## Validation

- `cargo fmt --all -- --check` — pass; the stable toolchain emits only the
  repository's existing nightly-rustfmt-option warnings.
- `cargo check --lib --no-default-features --features pg18` — pass.
- Focused tests were added for grammar, allowlist widths, the 258-byte bound,
  schema fingerprint binding, no-cover behavior, and rejection classes. They
  were not run under the repository's no-tests-by-default policy; no
  PostgreSQL, pgrx, fixture, or benchmark command was run.

Durable output and command provenance are in `artifacts/manifest.md`.

## Review questions

1. Is the reloption grammar exactly the accepted canonical bounded contract?
2. Does schema resolution fail closed for every accepted exclusion while
   retaining precisely the fixed PG18 scalar set and 258-byte maximum?
3. Are T1/T2 preflight sites early enough and no-cover behavior unchanged?
4. May checkpoint 2 proceed to the canonical descriptor/entry codec and the
   single heap plus non-covering row-TID B-tree representation?
