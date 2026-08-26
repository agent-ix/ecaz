---
agent: Codex
role: coder
model: gpt-5
date: 2026-08-25
seq: 01
---

# Task 224 packet 003 — MAT-26 implementation and screen preregistration

Review requested at code HEAD
`0ad5d63930bb021114585f64da5ab3622e4ddf7b` before the live 100k A/B.
Packet 002 authorized exactly one MAT-26 candidate under instrumentation parity;
this packet implements that candidate and preregisters the isolated screen.

## Candidate

The measured 6.967996 ms/scan bucket was PostgreSQL `array_send` serializing a
projected 1,536-element `real[]`: 1,536 element-length writes and 1,536
`float4send` fmgr calls per value. The candidate preserves PostgreSQL's binary
array wire format but emits null-free `real[]` values directly from one
detoasted flat array:

- dimension count, null flag, element OID, dimensions, and lower bounds are
  emitted in network order;
- every float retains its exact IEEE-754 bits, including negative zero and NaN;
- arrays containing NULL elements fall back to PostgreSQL `array_send`;
- all other SQL types fail closed;
- only frozen-schema attributes identified as `pg_catalog._float4` are
  substituted, and a candidate request with no such projection fails closed.

The path is absent from normal builds. It requires the feature-only, default-off
`ec_distann.benchmark_fast_real_array_send` GUC, which the coordinator transmits
as an explicit owner-endpoint parameter. It cannot combine with the Task 224
locality profiler or Task 220 packed payload. Existing production SQL remains
the control shape.

## Screen preregistration

Config:
`crates/ecaz-cli/suites/task224-mat26-fast-real-array-100k.json`
(`ddad71b7f8d92b9ec3e061e2622ff09820d4edfb3ea400c196f7dbcbe8746d57`).

Both steps use:

- the vector-bearing projection;
- `skip_owner_locality_profile=true`, so both arms run unprofiled production
  payload SQL;
- the same two eager/lazy-10 runtime variants in the same order;
- one external run directory and exact fixture reuse for the candidate step;
- 200 frozen queries, 20 warmups, and 200 measured iterations;
- recall prediction output, stage/work counters, and the full materialization
  semantic/failure matrix.

The only cross-step runtime difference is
`owner_fast_real_array_send=false/true`. Candidate runs additionally fail if
owner activation telemetry reports zero projected values or binary-send bytes.
The reuse invariant attests the exact epoch fingerprint at runtime.

Proposed decision gate: advance to packet 004 only if the matched production
lazy-10 candidate improves warm mean by at least 5%, does not regress p95 or p99
by more than 5%, produces byte-identical predictions, and passes both semantic
matrices. Otherwise Task 224 STOPs after this screen. The expected improvement
remains bounded by packet 002's 5.148990 ms / 18.258830% endpoint critical path,
not the 24.709206% summed-owner bucket.

## Validation at HEAD

- normal PG18 `cargo check`: pass;
- feature PG18 `cargo check`: pass;
- pure wire encoder tests: 2 pass;
- focused CLI/suite tests: 4 pass;
- focused PG18 byte-equivalence and wrong-type tests: 2 pass;
- `cargo fmt --all -- --check`: pass.

The SQL byte-equivalence test covers empty arrays, NaN/negative zero,
multidimensional arrays with non-default lower bounds, and NULL-element fallback
against PostgreSQL `array_send`. See `artifacts/manifest.md` and the packet-local
validation logs.

## Review questions

1. Is the exact-byte sender implementation safe and semantically complete for
   the registered `real[]` projection boundary?
2. Does the explicit coordinator-to-owner flag keep the production path and
   the A/B isolation honest?
3. Is the preregistered 5% usefulness gate appropriate, or should it be amended
   before the 100k run?
4. May the live same-generation 100k screen proceed at this HEAD?

