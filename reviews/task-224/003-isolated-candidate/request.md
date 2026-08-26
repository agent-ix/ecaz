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

Reviewer seq02 also returned **NOT DONE**: the candidate command still paired a
reused fixture with the crash/restart correctness matrix, the matrix would have
cold-biased the candidate by running after control A, the attribution equation
did not name executable artifact fields/normalization, and one manifest hash
had an extra hex character
(`feedback/2026-08-25-02-reviewer.md`). Checkpoint
`b834b7fb3715b8fea27d78bbf577c2b47b55d220` addresses those findings. The
screen remains prohibited until seq03 rereview accepts this revision.

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

Timing config:
`crates/ecaz-cli/suites/task224-mat26-fast-real-array-100k.json`
(`47234e2880271108685c49114c92ab12b2d792cea9542153a622e668a25abff2`).

Semantic config:
`crates/ecaz-cli/suites/task224-mat26-semantics-10k.json`
(`d3a57b8b6d93bdf8d41bf5c9b31f9be6d5e6204b9cfb8d7a8ffd8cd714e09cb2`).

The amended suite has four ordered steps on one immutable fixture:

1. unprofiled production control A;
2. the unprofiled-SQL fast-sender candidate;
3. unprofiled production control B, bounding run-to-run noise around the
   candidate; and
4. a nonconforming profiled-control context arm, bounding the candidate's
   timing-shim handicap and exposing native `typsend` send-region work.

No timing step runs `materialization_correctness`; therefore no owner
crash/restart occurs between control A, candidate, control B, and profiled
context. A second suite runs control and candidate semantic matrices on two
independent, non-reused 10k fixtures with distinct run directories and ports.
Those matrix steps are correctness-only context: recall is skipped and their
one-iteration latency output has no decision weight.

Control A and candidate use:

- the vector-bearing projection;
- `skip_owner_locality_profile=true`, so both arms run unprofiled production
  payload SQL;
- the same two eager/lazy-10 runtime variants in the same order;
- one external run directory and exact fixture reuse for every subsequent step;
- 200 frozen queries, 20 warmups, and 200 measured iterations;
- recall prediction output and full stage/work counters.

The headline cross-step runtime difference is
`owner_fast_real_array_send=false/true`. Candidate latency runs fail unless
fast-path values are nonzero and both generic-array fallbacks and ineligible
requests are zero. Timing-step recall keeps the session switch enabled, so its
id-only query exercises visible native-send degradation instead of aborting.
The isolated candidate semantic step keeps the same switch enabled across all
seven correctness/failure scenarios. The timing reuse invariant attests the
exact epoch fingerprint at runtime.

`allow_debug_extension` is absent from all six steps. Before either run, all
nodes must receive a release, non-`pg_test`, attribution-feature extension; the
CLI must also be a release build from exact checkpoint
`b834b7fb3715b8fea27d78bbf577c2b47b55d220`. The fixture preflight must attest
one unanimous release SHA/profile/features tuple; every normalized row must
then report that exact SHA and `extension_build_profile=release`, otherwise the
screen fails closed.

Amended decision gate, fixed before measurement:

- Let `C` be the arithmetic mean of control A and control B's matched lazy-10
  `physical_benchmark_latency.values.mean_ms`, and let
  `N = abs(A-B) / C`. `N` is a conservative control-envelope floor, not a pure
  repeat-noise estimate: B is a lazy-10-only `stage_counter_only` step and runs
  later. Those protocol/position differences can only inflate the bar.
- The candidate must improve on `C` by at least 5% **and** at least `2*N`.
- Candidate p95 and p99 must each be no more than 5% above the arithmetic mean
  of the corresponding control `physical_benchmark_latency.values.p95_ms` and
  `p99_ms` fields.
- Attribution uses exactly one `physical_benchmark_stage` row per selected
  step, with `variant=lazy10-production`, `payload_shape=vector_bearing`, and
  `arm=physical`. Define `P_send`/`F_send` from
  `stage=materialize_owner_binary_send_work, values.mean_ms` in profiled-control
  and candidate; likewise take `P_critical`/`F_critical` from
  `materialize_owner_endpoint_critical` and `P_work`/`F_work` from
  `materialize_owner_endpoint_work`.
- Every selected stage row's `values.scans` must equal its step's
  `physical_benchmark_latency.values.count`, and all compared counts must be
  200. Require finite positive values and
  `R = min(P_critical/P_work, F_critical/F_work)` in `(0,1]`; otherwise the
  gate fails rather than choosing a post-hoc normalization.
- Packet 002 measured the profiled control's extra scalar `int4send` at
  0.005083 ms/scan (0.073% of the vector send bucket). Remove that known
  anti-conservative asymmetry and deflate summed owner work to a conservative
  serial equivalent:
  `D_attr = (P_send - F_send - 0.005083) * R`.
  Require `D_attr > 0` and `D_attr >= 0.5 * (C - candidate_mean)`.
- Candidate fast-path values must be nonzero; fallback and ineligible counters
  must be zero in the vector-bearing latency arm. Control-A and candidate
  prediction files for both eager and lazy-10 variants must be byte-identical.
  Each isolated semantic step must emit exactly seven
  `physical_materialization_correctness` rows and every row must say
  `pass=true`.

The profiled-control minus `C` warm-mean delta is also reported as a
conservative upper bound on the candidate's asymmetric timing-shim cost: the
context arm profiles both projected values while the candidate wrapper
instruments only the `real[]`. A passing observed candidate delta is therefore
a lower bound on the underlying sender win. Advance to packet 004 only if every
gate passes; otherwise Task 224 STOPs after this screen. The expected
improvement remains bounded by packet 002's 5.148990 ms / 18.258830% endpoint
critical path, not the 24.709206% summed-owner bucket.

## Validation

- At exact seq02-correction checkpoint `b834b7fb3`: normal and feature PG18
  `cargo check` pass; focused Task 224 CLI/suite tests pass 6/6; the focused
  reused-fixture exclusion test passes 1/1; and
  `cargo fmt --all -- --check` passes.
- At serializer checkpoint `7cafbd202`: pure wire encoder tests pass 2/2 and
  focused PG18 byte-equivalence/wrong-type tests pass 2/2. The subsequent
  sender edit only removes the bitmap-offset branch already proven unreachable
  by its earlier fallback; both PG18 build configurations pass after it.

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

## Response to reviewer seq02

1. **Unrunnable candidate / restart ordering:** timing config removes both
   correctness matrices. A separate suite owns two non-reuse 10k fixtures for
   native and fast-sender matrices, so neither can perturb the timing fixture.
2. **Validation gap:** suite validation now mirrors all three fixture-mutating
   drill exclusions for `reuse_fixture`; a focused test covers each flag. Dry
   runs at exact checkpoint `b834b7fb3` show no timing step carries
   `--materialization-correctness` and both semantic steps omit
   `--reuse-fixture`.
3. **Attribution execution:** the gate now names normalized result metrics,
   step/variant/shape/arm/stage selectors, scan-count equality, the observed
   critical/summed deflator, and the 0.005083 ms scalar-send correction. There
   is no discretionary normalization after results.
4. **Evidence hash:** the 65-character typo is corrected in the manifest.

Seq02 non-blocking items are also closed: debug override is asserted false for
control B, the unreachable bitmap-offset branch is removed, zero-node batches
retain the ineligible count internally, and the manifest now carries the
volatility/parallel-safety disclosure plus the scalar-send asymmetry bound.

## Review questions

1. Is the exact-byte sender implementation safe and semantically complete for
   the registered `real[]` projection boundary?
2. Does the explicit coordinator-to-owner flag keep the production path and
   the A/B isolation honest?
3. Is the preregistered 5% usefulness gate appropriate, or should it be amended
   before the 100k run?
4. May the live same-generation 100k screen proceed at this HEAD?
