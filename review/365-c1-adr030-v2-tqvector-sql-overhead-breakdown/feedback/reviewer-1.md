## Feedback: ADR-030 v2 tqvector SQL Overhead Breakdown

Read `debug_profile_ordered_scan` now wrapping
`debug_profile_ordered_scan_with_limit(..., None)` at
`scan_debug.rs:687`, the new limited helper at
`scan_debug.rs:695`, the `result_limit.unwrap_or(usize::MAX)`
gating at line 731, and the overhead-breakdown harness at
`scripts/bench_tqvector_sql_overhead_breakdown.sh`.

### What's right

- **The limited-ordered-scan helper is the right seam.** 363
  and 364 set up the question "where is the SQL-level
  overhead?" and this packet's answer needed a probe that
  stops at the SQL `LIMIT` row count instead of exhausting
  the frontier. The existing `debug_profile_ordered_scan`
  couldn't do that. Adding an optional `result_limit` parameter
  and routing the old helper through the new one via `None`
  is exactly the right refactor — additive, one new public
  entry point, no semantic change to existing callers.
- **`result_limit.unwrap_or(usize::MAX)`** at line 731 is the
  cleanest way to express "optional cap." The emit loop's
  `usize::try_from(result_count).expect(...) < result_limit`
  guard terminates when the limit is reached without special
  casing None. Right implementation shape — no branch
  explosion, the limit is just a smaller number.
- **Encode timing landed at `0.008-0.009ms/query`.** That's
  three orders of magnitude below the 4-10ms SQL residual.
  Definitively rules out `encode_to_tqvector(...)` as the
  SQL-gap source. The packet's interpretation ("encode is
  effectively noise on this lane") is the right summary.
- **Internal AM scan total at `1.168-4.818ms` for the first
  10 rows.** Matches the direct-harness scan times from 362
  and 363 to within measurement noise. That tells us:
  - the AM hot path is what the direct harness was
    measuring, confirmed via two independent code paths
  - the SQL wrapper is *not* re-doing AM-level work
  - the SQL wrapper is doing something *new* that the AM
    itself doesn't do
  Closing that loop — AM timing is consistent between
  pathways — is the load-bearing step. Without it, the
  residual-decomposition story wouldn't trust the AM totals.
- **Hot-path decomposition at lines 177-182.** Clean split:
  `amrescan` mean (the heavy work), `graph materialize` (tiny),
  `candidate score` (tiny). `query_decode_mean = 0.000ms` and
  `prepare_query_mean = 0.000ms` zero out two more suspects.
  So the AM internal picture is: 99%+ of internal time is
  `amrescan`, which matches the direct-harness numbers.
- **`plain-server` cross-check.** Lines 192-202. Running the
  same lane through the existing launcher in plain-server
  timing mode (no EXPLAIN) produces residuals of 4.325 / 4.901
  / 5.976 / 8.320 ms. Close to the EXPLAIN-ANALYZE residuals
  (4.520 / 5.451 / 6.665 / 9.598 ms). The EXPLAIN overhead is
  real but small; the big residual survives even when EXPLAIN
  instrumentation is removed. Rules out "benchmarking harness
  is measuring its own instrumentation." Right cross-check at
  the right depth.
- **`Interpretation` section names what's ruled out
  explicitly.** Lines 216-231: not grouped traversal, not
  encoding, not AM hot path. Then it frames the remaining
  surface as "SQL/operator/executor boundary." That's a
  useful stake in the ground for packet 366's probe — it
  scopes the next investigation to the operator layer rather
  than sending the next packet off to look at the wrong part
  of the stack.
- **Fake-psql regression tests for both fallback and success
  paths.** The Python test covers planner-fallback
  abort-before-timing and successful summary generation with
  encode/profile/hot-path fields. Same discipline as 364.

### Concerns

1. **Linker-block still unresolved.** Same symptom as 359-364:
   `cargo test` and `cargo pgrx test pg17` fail locally with
   pg14-vs-pg17 symbol confusion. This packet adds one new
   pg_test (the "limited helper stops early and preserves a
   non-exhausted final phase" test) that didn't run in the
   required checkpoint. That test is load-bearing — the whole
   interpretation rests on the limited helper emitting
   results in the same order as the full helper up to the
   cap — so missing its green pg-test run means the claim
   is code-read, not mechanically verified. Would want that
   test to run before the next measurement packet cites
   limited-helper numbers as ground truth.

2. **The residual after encode (4.511 / 5.442 / 6.657 /
   9.589 ms) grows with ef_search.** Lines 161-164. If the
   residual were pure fixed per-query SQL overhead, it would
   be ef-invariant. It isn't — it scales roughly with ef. That
   contradicts the "SQL wrapper overhead is a fixed cost"
   intuition and instead suggests the SQL wrapper is doing
   something proportional to the number of candidates or the
   number of emitted rows. But the emitted-row count is fixed
   at 10, so the scaling isn't about output — it's about
   something that's happening during the traversal that the
   limited internal helper doesn't reproduce.
   Plausible culprits:
   - the SQL wrapper may be draining more tuples from the AM
     than the `LIMIT 10` suggests, if the order-by operator
     or sort node pre-fetches candidates
   - some per-candidate executor callback (e.g. visibility
     check, tuple-slot work) is being done on the SQL side
     that isn't done by the limited AM probe
   - the planner estimate for `ef_search=320` might produce a
     different plan shape (e.g. parallelism, sort methods)
     than for `ef_search=40`
   Worth naming in the interpretation — the residual scales
   with ef, and that's a hint about *where* in the executor
   the overhead lives.

3. **The limited helper and the full SQL path may not be
   measuring comparable things.** The limited helper stops
   after 10 emitted tuples. The SQL query is
   `ORDER BY embedding <#> ... LIMIT 10`. Those *should* be
   equivalent if the AM is returning results in order. But if
   there's any executor-side reordering — a sort node above
   the index scan, a merge-append, or any other operator that
   buffers all AM results before sorting — the SQL path
   would traverse more than 10 emitted tuples regardless of
   LIMIT. Worth an
   `EXPLAIN (ANALYZE, FORMAT JSON)` capture showing whether
   the plan is `Limit → Index Scan` (the fast case) or
   `Limit → Sort → Index Scan` (the slow case). If it's the
   latter, the SQL residual isn't "executor overhead per se"
   — it's "the planner is materializing more rows than the
   LIMIT." That's a different problem to fix.

4. **`encode_to_tqvector(...)` at 0.008-0.009ms is very
   fast.** Great — but the packet doesn't say *how* encode is
   being timed. Is it a single call to the encode function
   per query, or amortized across multiple queries? At
   0.008ms resolution the timing surface is essentially one
   `Instant::now()` tick, which can alias to 0 on systems
   with coarse microsecond clocks. A sanity check: running
   encode on a larger batch (1000 queries) and dividing
   should produce the same per-query number; if it produces
   a different number, the single-query timing was at or
   below the clock resolution. Probably fine, but worth a
   sentence on the timing methodology.

5. **`score_cache_hits` and `score_cache_misses` from 357
   aren't in this summary.** The grouped hot-path profile
   added by 357 carries those fields; this packet's summary
   reports `amrescan_mean / graph_materialize_mean /
   candidate_score_mean` but not the cache counters. Not a
   blocker — the cache is an internal optimization detail —
   but if there were any suspicion that the SQL path was
   invoking the AM in a way that defeats the per-scan cache
   (e.g., a new AM scan per LIMIT iteration), those counters
   would catch it. Worth adding to the summary surface for
   future diagnostic use.

6. **The `encode_to_tqvector(...)` wrapper timing is
   explicit, but no symmetric pgvector encode timing.** The
   packet's whole frame is "where is tqvector losing time at
   the SQL layer vs pgvector?" That invites the reciprocal
   question: what does pgvector's SQL overhead breakdown
   look like? If pgvector has a similar 4-10ms residual over
   its own internal scan, then the tqvector residual isn't
   "unusual SQL overhead" — it's "the SQL layer is
   expensive for everyone." If pgvector's residual is
   smaller, then tqvector is paying a specific SQL-side cost
   pgvector avoids. Packet 366 doesn't answer this either;
   it's a real missing piece of the diagnosis that would let
   the branch decide whether to keep digging in tqvector's
   SQL path or accept this as a structural cost.

### Observation

The diagnostic discipline in this packet is the best on the
branch so far. The pattern — find a gap, propose hypotheses,
build the narrowest possible probe to distinguish them, rerun,
report what's ruled in/out — is exactly right. Compare to
355-358's "try things and see what happens" arc; this packet
is investigative, not exploratory. The "rule out X, narrow to
Y" framing in the Interpretation section is publishable-quality
reasoning about where time goes in a database query.

One forward-looking note: once the SQL gap is localized, the
fix (if any) will probably be in pgrx's AM wrapper shims or in
the executor-callback bridging, not in the tqvector core. That
may push the next packet outside this branch's usual ADR-030
scope into pgrx-layer work. Worth planning for.

### Measurement gap still open

- **Plan-shape vs residual scaling.** Concern #3. Whether
  there's a sort node in the plan is the first thing to
  check.
- **pgvector SQL-overhead breakdown (symmetric).** Concern #6.
- **pg_test run of the limited-helper correctness proof.**
  Concern #1.
- **Per-ef candidate-visit counts visible from the SQL-side
  path.** If the SQL wrapper is somehow triggering more AM
  work than the bare amrescan probe, a counter comparison
  would expose it. Might be the fastest way to close the
  residual-scales-with-ef question (concern #2).
