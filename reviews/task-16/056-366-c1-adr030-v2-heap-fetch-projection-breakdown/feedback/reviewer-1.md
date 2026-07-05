## Feedback: ADR-030 v2 Heap Fetch / Projection Breakdown

Read `debug_profile_ordered_scan_with_heap_fetch(...)` at
`scan_debug.rs:846`, the generic-scan-API sequence
(`index_beginscan` + `index_rescan` + `index_getnext_slot` +
`slot_getattr`) at lines 893-939, and the harness extensions
in `scripts/bench_tqvector_sql_overhead_breakdown.sh`.

### What's right

- **Generic index scan API is the correct implementation.**
  `index_beginscan` → `index_rescan` → `index_getnext_slot` is
  the same path Postgres's executor uses for ordered index
  scans. By running through the same API, this helper measures
  what the executor would do per tuple, not what an alternative
  hand-rolled path would do. That's what makes the executor-like
  total actually executor-like — otherwise the measurement
  would be diagnostic-only and not comparable to real SQL
  latency. Right choice.
- **Snapshot handling.** Lines 861-873: reuses the active
  snapshot if present, registers a snapshot if not. The
  `pushed_registered_snapshot` tracking flag ensures the
  registered snapshot is unregistered on cleanup. That's
  correct snapshot discipline — a pg_test called outside a
  transaction block would otherwise crash or leak snapshot
  refcounts.
- **Tuple slot allocation.** `MakeSingleTupleTableSlot` with
  `table_slot_callbacks(heap_relation)` at lines 875-879 — the
  right per-relation callback table, not a default. If the
  heap relation uses a non-heap AM (zheap, TableAM extension),
  the callbacks are still correct. Defensive and right.
- **The `index_fetch_heap` crash learning is the real
  contribution.** The implementation note at lines 80-84: the
  first local attempt mixed `tqhnsw_amgettuple(...)` (the AM's
  own gettuple entry point) with `index_fetch_heap(...)` (a
  Postgres internal that expects to be called *by* the
  `index_getnext_slot` path, not alongside a parallel
  gettuple). Going through the generic API fixes the
  double-call. Worth preserving this as a note — anyone
  writing a future debug probe for another AM extension will
  hit the same footgun. The lesson is "don't combine AM
  entries with `index_fetch_heap` manually; go through
  `index_getnext_slot` which routes both internally."
- **Executor-like total tracks internal AM total within
  noise.** Lines 169-172:
  - ef=40: internal 1.141 vs executor-like 0.964
  - ef=64: internal 1.257 vs executor-like 1.346
  - ef=128: internal 2.006 vs executor-like 2.062
  - ef=320: internal 4.105 vs executor-like 4.030
  The deltas (-0.177 / +0.089 / +0.056 / -0.076 ms) are within
  measurement noise. That's the load-bearing finding: the
  executor-like path is not measurably more expensive than the
  bare AM probe at the first-10-rows granularity.
- **Slot fetch and projection are bounded.** Slot fetch:
  0.008-0.033ms. Projection: 0.000ms at every cell. These are
  within-AM-scan costs and they're tiny. Packet's conclusion:
  heap fetch and simple tuple projection aren't the SQL
  residual source.
- **Ruling out more of the stack.** Lines 186-193: encoding,
  grouped traversal scoring, heap-slot fetch, simple slot
  projection — all excluded by this and preceding packets.
  What's left: executor node callbacks, SQL tuple
  materialization, and possibly planner path differences.
  Narrower search space for the next packet.

### Concerns

1. **The executor-like path reuses the `ScanKeyData` argument
   shape.** Lines 909-913. Works in isolation but: in the real
   SQL path, `amrescan` receives `ScanKeyData` that was set up
   by the executor via `ExecIndexBuildScanKeys`, which does a
   few things the direct helper doesn't:
   - it evaluates the order-by expression to a datum (the
     helper does that too, by using `IntoDatum::into_datum`)
   - it may wrap the ScanKey with additional scan-key flags
     (SK_ORDER_BY, SK_ISNULL) that the AM's `amrescan` code
     reads
   This helper sets `sk_argument` and defaults the rest.
   Grepping for `SK_ORDER_BY` and `SK_ISNULL` in tqvector's
   `amrescan` would confirm whether the AM relies on those
   flags being set correctly by the caller; if it does, the
   helper's defaulted scan-key flags could cause it to exercise
   a subtly different path than the real executor. Worth
   verifying — otherwise the "executor-like total tracks
   internal AM total" conclusion is contingent on the scan-key
   shape being equivalent at the AM-observed level.

2. **The `scan->heapRelation` binding pattern is correct for
   index-only cost paths but may not be for all SQL plan
   shapes.** The helper uses `index_beginscan(heap_relation,
   index_relation, ...)` which is the standard bitmap-or-
   ordered scan path. A real SQL `ORDER BY ... LIMIT` on this
   corpus likely goes through the same path. But if the
   planner elects an Index-Only Scan (unlikely given the
   order-by operator, but not impossible), the executor uses
   `index_getbitmap` or a different slot-fetch path. One plan
   snapshot from the harness would confirm the helper is
   measuring the right shape. Packet 365's concern #3 about
   plan snapshots applies here too — without seeing the real
   plan, we're assuming the helper and the SQL query use the
   same code path.

3. **`ExecClearTuple(slot)` at line 945 resets the slot
   between iterations, which is required, but doesn't mirror
   what a `Limit` node above the index scan would do.** In a
   real SQL query with `LIMIT 10`, the Limit node reads 10
   tuples from its child and stops — the child index scan's
   slot state on iteration 11 never happens. The helper
   approximates this by ending the loop at `result_limit`, but
   if the AM's internal state has any kind of
   look-ahead-then-emit pattern (which grouped-v2's live rerank
   window might, per 353/360), the helper and the SQL query
   could trigger different AM work even at the same "10 rows
   emitted" boundary. Not blocking — the numbers line up within
   noise, so whatever lookahead happens isn't dominating — but
   worth a sentence naming that the helper and the SQL-Limit
   boundary aren't identical.

4. **Residual over executor-like (3.733 / 4.214 / 5.126 /
   6.781 ms) is still very large.** The packet's phrasing
   "strongly suggests the remaining cost is higher in the
   stack" (line 205) is right, but the magnitude matters. At
   ef=320, 63% of the SQL latency is *outside* the AM +
   executor-like path. That's a lot of SQL overhead to live
   above the index scan. Likely candidates:
   - ExecutorRun startup/teardown
   - SeqScan-to-IndexScan plan conversion cost
   - `LIMIT` node + order-by sort (if present)
   - result row materialization to the client
   - ExecutorEnd / portal cleanup
   A profiler trace (perf, bpftrace) of one SQL query would
   probably surface the dominant remaining cost in a way that
   another narrower probe can't. That's the escape hatch if
   the next probe-driven packet doesn't find the answer.

5. **No pgvector comparison on the executor-like path.** Same
   as packet 365's concern #6. This packet's ruling-out work
   is internal to tqvector — it tells us heap fetch isn't
   slow on *our* AM, but not whether heap fetch is equally
   fast on pgvector's AM. If both AMs show similar
   executor-like totals, then the ~1ms SQL-level gap from 364
   is explained by something not on the executor-like path
   (which is the packet's conclusion anyway). But confirming
   it *symmetrically* would close the loop.

6. **The fake-psql regression tests are fine but don't
   exercise the new generic-scan path directly.** The pg_test
   added here ("the helper stops at the requested limit and
   projects the requested heap attribute on a small fixture")
   is the load-bearing correctness test for the new probe. Per
   the checkpoint section, it didn't run in the required
   `cargo test` / `cargo pgrx test pg17` commands — same
   linker-block as 359-365. That's now six packets running on
   clippy + `cargo check` only. Any one of those packets
   adding pg test coverage that hasn't been run in a green
   required checkpoint is a cumulative risk. If CI has the
   green run, a pointer would close this. Otherwise, the
   "fix-the-linker" packet is overdue.

### Observation

Two meta-points:

1. **The diagnostic arc 365-366 is the cleanest sequence on
   the branch.** 365 adds the encode/AM/residual
   decomposition and rules out encoding + AM. 366 adds the
   executor-like layer and rules out heap fetch + projection.
   Each packet narrows the search space, doesn't chase
   assumptions, and reports findings that are immediately
   actionable for the next packet. This is what the "strategy
   enum + experiment" arc in 355-358 was *trying* to be but
   landed less cleanly. Worth naming as the template for any
   future performance-investigation packets on this repo.

2. **The "don't mix amgettuple with index_fetch_heap" crash
   learning belongs in a developer-facing note.** At lines
   80-84 the packet documents the crash and the fix briefly.
   That's the kind of tribal-knowledge trap that would
   otherwise be re-discovered painfully by the next developer
   writing a pg_test helper. Appending a one-paragraph note
   to either `CONTRIBUTING.md` or a `HACKING.md` under
   "writing pg_test debug probes" would prevent the
   re-discovery.

### Measurement gap still open

- **Plan snapshots for both SQL paths (tqvector-SQL and
  pgvector-SQL).** Concern #2, applicable across 365 and 366.
- **Profiler trace of one SQL query.** Concern #4. Would
  pinpoint the remaining cost without requiring another
  probe-driven packet.
- **Symmetric pgvector executor-like breakdown.** Concern #5.
- **pg_test green run on the required checkpoint.** Concern
  #6. Applies to everything 359-366.

### Final framing

After 365-366, the SQL-level gap has a definitive localization:
the remaining 3.7-6.8ms per query is *above* the executor's
slot-fetch path and *not* explained by anything at or below
the AM. That's a useful result even though it's a negative
result — it means the next profitable investigation isn't more
tqvector-internal work but instead a direct SQL-layer probe
(profiler trace, plan snapshot comparison with pgvector, or a
pgrx-side measurement of the amrescan-to-executor bridge).
That conclusion should be carried forward into the ADR's
"known limitations" section as "tqvector's SQL-layer overhead
is ~4-7ms/query *above* the index-scan cost, dominated by
executor-node and result-shaping work that is not specific to
the tqhnsw AM." Any product conversation about grouped-v2's
latency budget has to include that constant.
