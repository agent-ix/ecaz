---
agent: Codex
role: coder
model: gpt-5
date: 2026-08-25
seq: 01
---

# Task 224 packet 003 — MAT-26 implementation and screen preregistration

Initial review was requested at code HEAD
`0ad5d63930bb021114585f64da5ab3622e4ddf7b` before the live 100k A/B.
Packet 002 authorized exactly one MAT-26 candidate under instrumentation parity;
this packet implements that candidate and preregisters the isolated screen.

Reviewer seq01 returned **NOT DONE** and did not authorize the screen. The
follow-up code checkpoint and amended preregistration below address all four
blockers at corrected code HEAD
`7cafbd2027b05365afd47c6f8b34c0415e6b78fc`; a live run remains prohibited
until outside rereview accepts them.

## Candidate

The measured 6.967996 ms/scan bucket was PostgreSQL `array_send` serializing a
projected 1,536-element `real[]`: 1,536 element-length writes and 1,536
`float4send` fmgr calls per value. The candidate preserves PostgreSQL's binary
array wire format but emits null-free `real[]` values directly from one
detoasted flat array:

- dimension count, null flag, element OID, dimensions, and lower bounds are
  emitted in network order;
- every float retains its exact IEEE-754 bits, including negative zero and NaN;
- arrays carrying a NULL bitmap fall back to PostgreSQL `array_send`, including
  the reachable bitmap-without-NULL shape, so the flags word is byte-identical;
- all other SQL types fail closed;
- only frozen-schema attributes identified as `pg_catalog._float4` are
  substituted; requests with no such projection use their frozen native send
  functions and increment an explicit ineligible-request counter.

The path is absent from normal builds. It requires the feature-only, default-off
`ec_distann.benchmark_fast_real_array_send` GUC, which the coordinator transmits
as an explicit owner-endpoint parameter. It cannot combine with the Task 224
locality profiler or Task 220 packed payload. Existing production SQL remains
the control shape.

## Screen preregistration

Config:
`crates/ecaz-cli/suites/task224-mat26-fast-real-array-100k.json`
(`d9b086cc4664390dd8833e2ff8db8965e98a41a35965159cce14feda7834e941`).

The amended suite has four ordered steps on one immutable fixture:

1. unprofiled production control A;
2. the unprofiled-SQL fast-sender candidate;
3. unprofiled production control B, bounding run-to-run noise around the
   candidate; and
4. a nonconforming profiled-control context arm, bounding the candidate's
   timing-shim handicap and exposing native `typsend` send-region work.

All decision-bearing steps use:

- the vector-bearing projection;
- `skip_owner_locality_profile=true`, so both arms run unprofiled production
  payload SQL;
- the same two eager/lazy-10 runtime variants in the same order;
- one external run directory and exact fixture reuse for every subsequent step;
- 200 frozen queries, 20 warmups, and 200 measured iterations;
- recall prediction output, stage/work counters, and the full materialization
  semantic/failure matrix.

The headline cross-step runtime difference is
`owner_fast_real_array_send=false/true`. Candidate latency runs fail unless
fast-path values are nonzero and both generic-array fallbacks and ineligible
requests are zero. Recall and the correctness matrix keep the session switch
enabled; id-only/narrow queries exercise the visible native-send degradation
instead of aborting. The reuse invariant attests the exact epoch fingerprint
at runtime.

`allow_debug_extension` is absent from every step. Before the run, all nodes
must receive a release, non-`pg_test`, attribution-feature build at the reviewed
HEAD; the fixture preflight must independently attest the unanimous release
profile and exact git SHA.

Amended decision gate, fixed before measurement:

- Let `C` be the arithmetic mean of control A and control B's matched lazy-10
  warm means, and `N = abs(A-B) / C` be the measured noise floor.
- The candidate must improve on `C` by at least 5% **and** at least `2*N`.
- Candidate p95 and p99 must each be no more than 5% above the arithmetic mean
  of the corresponding control percentiles.
- The matched send-region saving (`profiled-control owner_binary_send_ns` minus
  candidate `owner_binary_send_ns`) must be positive and at least 50% of the
  end-to-end warm-mean saving. This makes a flat or contradictory send bucket
  an attribution failure.
- Candidate fast-path values must be nonzero; fallback and ineligible counters
  must be zero in the vector-bearing latency arm; cross-step prediction files
  must be byte-identical; both decision-bearing semantic matrices must pass.

The profiled-control minus `C` warm-mean delta is reported as a conservative
upper bound on the candidate's asymmetric timing-shim cost: the context arm
profiles both projected values while the candidate wrapper instruments only
the `real[]`. A passing observed candidate delta is therefore a lower bound on
the underlying sender win. Advance to packet 004 only if every gate passes;
otherwise Task 224 STOPs after this screen. The expected improvement remains
bounded by packet 002's 5.148990 ms / 18.258830% endpoint critical path, not the
24.709206% summed-owner bucket.

## Validation at HEAD

- normal PG18 `cargo check`: pass;
- feature PG18 `cargo check`: pass;
- pure wire encoder tests: 2 pass;
- focused CLI/suite tests: 5 pass;
- focused PG18 byte-equivalence and wrong-type tests: 2 pass;
- `cargo fmt --all -- --check`: pass.

The corrected SQL byte-equivalence test covers empty arrays, NaN/negative zero,
multidimensional arrays with non-default lower bounds, NULL-element fallback,
and a NULL bitmap whose NULL slot was overwritten against PostgreSQL
`array_send`. The empty-array path also avoids constructing a zero-length slice
past the allocation. See `artifacts/manifest.md` and the packet-local validation
logs.

## Response to reviewer seq01

1. **Session-GUC blast radius:** fixed by native-send degradation for
   projections without `real[]`; new outcome telemetry counts fast values,
   generic-array fallbacks, and ineligible requests.
2. **Debug extension:** fixed in preregistration by removing every debug
   override. The release reinstall and exact-SHA preflight are mandatory run
   prerequisites, not assertions supplied by the packet.
3. **Bitmap parity:** fixed by falling back on `ArrayType.dataoffset != 0`; the
   reviewer's bitmap-without-NULL reproducer is now a PG18 regression case.
4. **Instrumentation/noise/attribution:** the two extra fixture-reuse steps
   establish control-repeat noise and a profiled native-send context. The exact
   usefulness and attribution gates above were fixed before seeing results.

Non-blocking items (a), (b), and (c) are also closed by explicit outcome
counters with fail-closed CLI assertions, a pinned provenance-suffix unit test,
and the empty-array early return. Item (d)'s volatility/parallel-safety
difference is retained and disclosed: this lateral per-row payload expression
cannot be folded or parallelized in either arm, and any residual planner effect
is conservative for the candidate.

## Review questions

1. Is the exact-byte sender implementation safe and semantically complete for
   the registered `real[]` projection boundary?
2. Does the explicit coordinator-to-owner flag keep the production path and
   the A/B isolation honest?
3. Is the preregistered 5% usefulness gate appropriate, or should it be amended
   before the 100k run?
4. May the live same-generation 100k screen proceed at this HEAD?
