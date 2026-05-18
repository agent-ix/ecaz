## Feedback: ADR-030 v2 pgvector SQL Latency Harness

Read `scripts/bench_pgvector_sql_latency.sh`, the scratch
wrapper `bench_pgvector_sql_latency_scratch.sh`, and the Python
regression test. Measurement surface at lines 100-155.

### What's right

- **Harness parity with the existing tqvector launcher.** The
  new pgvector launcher follows the same shape as
  `bench_sql_latency_verified_scratch.sh`: `--corpus-table`,
  `--query-table`, `--index-name`, `--ef-search` list,
  per-cell or per-query backend reuse, `EXPLAIN ANALYZE` or
  plain-server timing. Symmetric harness design is what makes
  363's size comparison turn into a clean apples-to-apples
  runtime comparison here. No asymmetric hidden
  instrumentation between lanes.
- **Planner verification per cell.** Mirrors the tqvector
  launcher's "abort-before-timing on planner fallback"
  behavior. That's the right guardrail for a benchmark that
  switches `ef_search` between cells — if the planner
  regresses to seqscan for any single ef, the harness notices
  and exits instead of silently producing meaningless
  timings. This has been a recurring footgun across this
  branch; reusing the pattern for pgvector closes it for the
  external baseline too.
- **Python regression coverage of abort-before-timing.**
  `test_bench_pgvector_sql_latency.py` exercises both the
  planner-fallback abort path and a successful multi-ef
  timing run with warmup. That's the right regression
  surface; it means future changes to the launcher can't
  silently break either control flow.
- **First real SQL apples-to-apples read.** Lines 160-168:
  pgvector ef=128 7.465ms vs tqvector grouped ef=128 8.523ms
  is a ~1ms SQL-level gap *in favor of pgvector*, despite
  tqvector winning on direct-harness at the same operating
  point by ~1.5ms. That's the packet's load-bearing finding
  and it's surfaced directly — no hedging.
- **Rerun of isolated tqvector grouped m=16 on the same
  cluster state.** Not just a reuse of 363's numbers; an
  actual rerun of the verified SQL harness on the same
  scratch state as the pgvector run. That controls for
  cluster-state drift between the two lanes and makes the
  comparison cleaner. Right discipline.

### Concerns

1. **Linker-block still open, still accumulating.** Required
   checkpoints for `cargo test` and `cargo pgrx test pg17`
   failed at the same pg14-vs-pg17 symbol layer as 359, 360,
   361. That's now five packets in a row without the required
   pg test surface. This packet doesn't add new pg-test
   coverage, so the incremental gap is small, but the
   cumulative gap on the branch is real — pg-test coverage
   added across 359-363 has not been verified in a green
   checkpoint on this workstation. If the pg tests are
   running in CI, a pointer to that green run in the packet
   (URL, timestamp) would let readers trust the coverage
   claim. Otherwise, the "install the linker fix" packet
   proposed in 359's feedback is getting harder to defer.

2. **The SQL gap reversing the direct-harness story is the
   key finding, and the packet's interpretation understates
   it.** Line 171-172 says "tqvector's current direct-harness
   latency advantage is being eaten by end-to-end SQL/operator
   overhead." That phrasing is correct but soft. The hard
   version is: *tqvector is slower than pgvector at the
   user-visible SQL level across the interesting ef range*.
   That's the shippable-readout answer to "does the grouped
   compressed ANN beat pgvector?" — and right now the answer
   is no. Packet 365 picks this up as the next diagnostic,
   which is the right response, but this packet should frame
   the SQL-level outcome as the current user-visible result,
   not just as a measurement anomaly to explain.

3. **`ef=320` tqvector 13.466ms vs pgvector 13.389ms is
   effectively tied (0.077ms delta).** Line 152-154. The
   per-ef gap shrinks toward zero as ef grows, which is
   consistent with the direct-harness story: tqvector's
   per-candidate work is cheaper than pgvector's, so at high
   ef the fixed overhead per query (encode, planner,
   tuple-shaping) matters less relative to the candidate
   loop. That's *good news* for tqvector at high-ef operating
   points, but the packet doesn't name it. Worth an
   interpretation line: "the SQL-level gap is concentrated at
   low-to-mid ef; by ef=320 the two lanes are tied, implying
   tqvector's SQL-level overhead is largely a fixed per-query
   cost rather than a per-candidate cost." That framing
   matters because fixed per-query costs are much easier to
   optimize (encode, planner, slot setup) than per-candidate
   costs (inner loops, memory access).

4. **The pgvector runtime is on a `vector(1536)` type with a
   single-column heap.** The tqvector grouped lane is on the
   same heap shape per 363, so the comparison is fair at the
   tuple level. But worth noting in the packet that the
   runtime comparison is between two minimally-columned
   tables (id + embedding), which is the *best case* for the
   SQL wrapper overhead — adding more columns to the heap
   doesn't change the AM cost but does change the per-tuple
   executor work. So this is the packet's lower bound on the
   SQL residual; real workloads with wider tuples will
   probably show a larger SQL residual on both lanes, and the
   direction of the relative gap is an open question.

5. **No `--warmup-passes` sensitivity check.** The harness
   uses `--warmup-passes 1`. Reasonable default, but for a
   benchmark that's being compared at sub-millisecond
   resolution (`0.077ms delta at ef=320`), the measurement
   noise floor depends on how much cache the warmup
   populates. A quick sensitivity sweep
   (`--warmup-passes 0,1,3`) on one ef cell would let readers
   see whether the reported numbers are above or below the
   noise floor. Not required in this packet — the point was
   the harness, not the noise analysis — but worth a
   follow-up if any subsequent packet cites these numbers as
   "tqvector is X% slower at SQL."

6. **No query-plan snapshot in the packet.** The harness
   verifies the planner picks the expected index per cell,
   but the packet doesn't include a sample
   `EXPLAIN (ANALYZE, FORMAT JSON)` output showing what the
   actual plan shape looks like on each lane. For a packet
   that's comparing end-to-end SQL latency and finding a
   counterintuitive gap, the plan shape is load-bearing
   context. Is there an extra sort node? Different
   startup-cost breakdown? Different parallel-mode choice?
   Packet 365 implicitly investigates this by going deeper
   into the residual decomposition, but a paste of one
   representative plan for each lane would help grounding.

### Observation

Three meta-points:

1. **The harness is a durable contribution.** The measurement
   outcome in this packet will be superseded as the SQL gap
   narrows (or doesn't), but the reusable pgvector launcher
   and its planner-verify guardrail will stay useful for
   every future external-baseline packet. Treat this as the
   infrastructure packet with measurement-as-demonstration,
   not the other way around.

2. **The direct-harness vs SQL-harness divergence is a
   recurring pattern.** 360 saw it with isolated-vs-canonical
   grouped builds; 363 saw it with pgvector-vs-tqvector
   direct-runtime; this packet sees it with pgvector-vs-tqvector
   SQL. Every time there's a gap between direct and SQL, the
   explanation has been different. That's worth naming: "do
   not assume the SQL surface will track the direct surface
   on the grouped-v2 lane" is a stable learning that should
   live in the branch notes. The operating point of a
   production workload is the SQL surface.

3. **The "compressed + fast" pitch for grouped-v2 is now
   conditional.** "Smaller and faster than pgvector" is only
   true on the direct harness. On the SQL harness it's
   "smaller but slower than pgvector at mid-ef, tied at
   high-ef, with the recall gap persisting." The packet
   naturally motivates the 365 investigation; the real story
   will depend on what that investigation finds.

### Measurement gap still open

- **What explains the SQL residual?** Packet 365 picks this
  up.
- **Noise floor at sub-ms resolution.** Concern #5.
- **Plan shape comparison.** Concern #6.
- **Wider-tuple heap comparison.** Concern #4. Not in the
  packet's scope to investigate, but should land before the
  final product readout.
