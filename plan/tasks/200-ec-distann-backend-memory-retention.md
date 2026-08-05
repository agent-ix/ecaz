# Task 200: ec_distann Backend Memory Retention

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

Status: **complete — root cause fixed, executable integration gate, unattended
PG18 regression, and Task 188 no-reconnect follow-up pass** (2026-07-29).
Priority: P2 after the production A1 and Task 188 follow-up came back bounded.
Phases 1 and 2 have run and several premises in the original filing were wrong —
see *Established* below before doing any work.

## Why

During the Task 188 packet-006 rerun, a PostgreSQL backend serving the repeated
physical query phase grew to approximately **52 GB RSS** and was still climbing
when the run was terminated to protect the host
(`reviews/task-188/006-batch10-stage-counters/artifacts/run/rerun-20260727/outcome.md`).
Task 200's own Phase 1 hit the same wall at **14.5 GB** and was also killed.

Two runs have now been destroyed by this. That is the reason the task exists,
and it remains true regardless of which code path turns out to be responsible.

Task 188 worked around it by reconnecting the benchmark backend every five
queries, which was the right call to finish the diagnostic but leaves the defect
unexamined.

## Established (2026-07-28) — do not re-derive

Evidence: `reviews/task-200/001-reproduction/`, `002-attribution/`,
`003-fix-and-regression/`.

**The production read path is flat under autocommit.** 100k three-owner physical
generation, one backend, `worker_batch_size=0`, 300 timed queries, 250 ms
sampling, both stage-counter settings on the *same* generation:

| arm | samples | elapsed | RSS first→last | slope |
| --- | ---: | ---: | ---: | ---: |
| counters off | 33 | 8067 ms | 260104→261028 KB | 114.54 KB/s |
| counters on | 32 | 7817 ms | 260024→261024 KB | 127.93 KB/s |

**The growth reproduces in a benchmark-only function.** It is
`ec_distann_physical_seed_coverage_benchmark`
(`src/am/ec_distann/generation_read.rs`), which is
`#[cfg(feature = "distann-head-attribution-benchmark")]` and absent from
production builds. The benchmark fixture issues it as a 200-way
`CROSS JOIN LATERAL` before the latency phase whenever `head_policy` is set and
neither `--stage-counter-only` nor `--skip-recall` is passed
(`crates/ecaz-cli/src/commands/dev/distann_multicluster.rs:4229`). Task 188's
config satisfied that gate, so its 52 GB is plausibly this same statement.

**The memory is in `TopTransactionContext`.** From
`pg_log_backend_memory_contexts` during the coverage statement
(`001-reproduction/artifacts/run-latency-rerun/diagnostic-node1.log:127`):

```
level: 2; TopTransactionContext: 8321499136 total in 1002 blocks;
          8603128 free (6895 chunks); 8312896008 used
Grand total: 8323959872 bytes
```

99.97% of the total, in one context. Every other context is trivial
(`CacheMemoryContext` 1.07 MB, `ExecutorState` 82 KB). 1002 blocks for 8.32 GB is
~8.3 MB per block — on the order of **five large allocations per lateral
invocation**, not many small ones.

**Consequences of that context, which drive everything below:**

- The memory is **PostgreSQL-allocated and PostgreSQL-visible**. The original
  filing claimed this class of growth lives in Rust `static`/`thread_local` state
  outside PostgreSQL's contexts and that `pg_backend_memory_contexts` would be
  blind to it. **That was wrong.** `pg_log_backend_memory_contexts(pid)` found it
  in one call. Allocator tooling was not needed.
- `TopTransactionContext` is **transaction**-lifetime. It is reset at
  commit/abort. This is why the autocommit loop above is flat — one transaction
  per query, discarded 300 times — and why one long statement grows without
  bound.
- It is therefore **not** "a gigabyte retained per query" as originally filed.
  It is unbounded accumulation *within a transaction*. Still a defect: a
  transaction that reaches 52 GB kills the host long before it can commit.

**Also corrected in the original filing:**

- "The growth occurred while executing the ordinary semantic ANN SQL through the
  coordinator's CustomScan — the production read path — not a benchmark-only
  `pg_extern`." Wrong on both counts, per the above. It *is* a benchmark-only
  `pg_extern`, and by the same reasoning it is closer to the Task 185 class
  (`df89b5726`, `c83ea6ea8`) than the filing allowed.
- `remote_transport.rs` was a "suggested starting point." **Ruled out.**
  `DistannTransportState` is a single `thread_local` holding one current-thread
  tokio runtime plus connections pooled by `lifecycle_connection_key(conninfo)`;
  `PooledConnection::Drop` aborts its task and `with_transport_state` clears the
  map on an observed interrupt. Nothing accumulates across calls.
- `head_cache.rs` bounded, as originally stated. Confirmed:
  `HashMap<u32, Arc<…>>` keyed on `index_oid`, `insert` replaces.

**One diagnostic result must not be cited.** `coverage-separate-200.log` was run
as 200 statements in a **single simple-query protocol message**
(`diagnostic-node1.log:279`), which PostgreSQL executes in one implicit
transaction. Separate statements, one transaction — so it cannot discriminate
statement-scoped from backend-scoped retention, and its 1.9→3.6 GB figure is
exactly what transaction-scoped accumulation predicts. Re-run per Phase 3 A.

## Goal

Bound the allocation, prove the production read path is unaffected under a held
transaction, and leave a regression test that fails if either regresses.

Diagnosis is done. What remains is the fix and the proof.

## Phase 3 A: close the two open questions (complete)

These were run on the retained fixture at
`/home/peter/.ecaz/clusters/task200-counters-off-100k` (100k, three owners,
published, 6.8 GB). **Do not rebuild it** — `ecaz bench suite` gained
`--reuse-fixture` in `9de8b4fa2`; rebuild remains the default, so pass it
explicitly. Preserve that directory until Phase 3 B lands.

**A1 — production read path safe inside a held transaction: bounded.**
This is the gate on this task's priority and on any claim that production is
unaffected. The Phase 1 A/B ran in autocommit, which resets
`TopTransactionContext` between queries and therefore **cannot detect this class
of defect**. Run, one backend, sampler attached:

```
BEGIN;
  -- 300 ordinary ANN queries through the CustomScan, identical to the
  -- Phase 1 latency arm
COMMIT;
```

- Flat → production is unaffected, priority drops to P2, and the closeout takes
  the instrumentation branch below.
- Grows → this is a production defect, priority stays P1, and closeout requires
  the read-path branch including the 10k/50k/100k A/B.

Report the series, not a peak. A batch/ETL path holding a transaction across many
ANN queries is an ordinary production shape; this is not a hypothetical.

**A2 — allocating call site: identified and fixed.**
Attribute to a **call site**, not a function. "The coverage helper" is where
packet 002 already stands and is not sufficient.

Constraints that narrow the search — use them, don't re-derive them:

- There is **no `MemoryContextSwitchTo` anywhere in production `src/`** (only in
  `src/tests/ec_distann_basic.rs`). Nothing deliberately targets
  `TopTransactionContext`; the allocations land there because
  `CurrentMemoryContext` *is* that context when they run.
- ~5 allocations of ~8 MB per invocation.
- No SPI contexts appear in the dump, so SPI result retention is not the holder.

Leading candidate: the helper calls
`PhysicalGenerationScan::open(index_regclass.oid())` on **every** lateral
invocation (`generation_read.rs:1735`) was tested directly and remained flat.
The retention was then isolated to `owner_scan_seed_candidates`, specifically
the pgrx `value::<Vec<u8>>()` conversion of `graph_record`.

`pg_log_backend_memory_contexts` is the working tool here; heaptrack/massif are
**not** required and the earlier instruction to expect them is withdrawn.

## Phase 3 B: bound it, and keep it bounded

Required in every outcome except "unreproduced", which no longer applies —
including if A1 comes back flat and this is confined to instrumentation.
"Benchmark-only" is not a reason to leave it: it has destroyed two runs and it
will destroy the next one.

1. **Bound the allocation** at the call site A2 identifies. Whichever shape fits:
   hoist the per-invocation work out of the lateral, reset a per-call context, or
   free explicitly. Record which, and why it is the right one.
2. **Document the mechanism** in the packet — the call site, why it lands in
   `TopTransactionContext`, and why the production path does or does not reach
   it. Cite the Phase 1 series and the A1 series as evidence, not a code-reading
   argument.
3. **Regression test.** Several hundred coverage calls on one backend inside a
   single transaction, asserting bounded RSS. Assert on **post-warm-up slope and
   absolute working-set delta**, not an unqualified startup peak: the
   noise floor is under 1 MB across 300 queries against a ~100 MB per-invocation
   signal, so ~20 iterations is enough and a peak threshold silently passes any
   leak that stays under it. If A1 grew, the same test is required for the
   production read path under a held transaction.
4. **Remove the Task 188 reconnect workaround only after the Task 188-owned
   stage-counter step is rerun on the fixed committed build with
   `benchmark_backend_batch_size=0`, `skip_recall` absent, both seed variants,
   and backend memory sampling.** A bounded series supports removing the
   workaround; continued growth means it stays. Confirm with the Task 188
   owner before touching that lane or its config.

## Closeout

- **If A1 grows** (production defect): bound it, regression test, and a
  **10k/50k/100k A/B** showing recall exact against current numbers and no
  latency regression, per the repository closeout rule and NFR-007.
- **If A1 is flat** (instrumentation): bound it anyway, document the mechanism,
  and record why the production path is unaffected — with the Phase 1
  counters-off series *and* the A1 in-transaction series as the evidence. No
  10k/50k/100k matrix, since the read path is unchanged. State that waiver as
  conditional on the read path staying unchanged.
- Do not close on the reconnect workaround.
- Do not close with the fix outstanding. Packet 003 is
  `fix-and-regression`; diagnostic tooling alone does not satisfy it.

## Required review packets

1. `reviews/task-200/001-reproduction/` — **submitted and addressed.** Phase 1 series for both
   counter configurations, suite config checked in.
2. `reviews/task-200/002-attribution/` — **submitted and addressed.** The
   `TopTransactionContext` citation and A2 call site are recorded.
3. `reviews/task-200/003-fix-and-regression/` — **submitted and addressed.**
   It contains A1, the bound, the executable regression, fixed-green/pre-fix-red
   evidence, and the Task 188 no-reconnect follow-up.

## Provenance closeout

Historical diagnostic results report
`extension_git_sha=897c69045249a876de151c1da0544001ead82352-dirty` and remain
labeled as historical. All closeout evidence cited for the fix and regression is
from committed, provenance-attested trees; the Task 188 follow-up used clean
committed tree `d845d8e4347d59dafd2b1ed28cd252d7d7c6e134`. The extension was release and attested
(`release_profile_preflight status=passed … unanimous=true`); the `target/debug`
driver is acceptable for memory diagnostics but its `mean 27.50 ms` / `26.50 ms`
figures are not latency evidence.

## Non-goals

- BW8 promotion or any Task 188 candidate decision.
- The Task 188 benchmark-harness refactor and its pre-merge gates.
- Recall, graph, head, or codec work.
- Allocator-level tooling (heaptrack, massif, jemalloc stats) unless
  `pg_log_backend_memory_contexts` proves insufficient, which it has not.

## Closeout record (2026-07-28)

- A1 was rerun against the reused 100k fixture with the committed release
  extension `fa84ff3b06bccec2a8f202338003da489a5ca105`. Three hundred ordinary
  production ANN queries ran inside one held transaction; RSS rose during
  initial setup and plateaued at 260780 KB, with no unbounded per-query growth.
- A2 attributed the retention to the owner scan call site
  `RetainedGenerationScan::seed_candidates`, specifically pgrx's
  `value::<Vec<u8>>()` conversion of `graph_record`; direct `open()` calls were
  flat while repeated owner scans grew `TopTransactionContext` to
  5595201536 bytes.
- The fix in `fa84ff3b0` reads the raw SPI datum and uses the owning
  `DetoastedVarlena` guard, freeing each detoast copy after row decoding.
- The clean committed-source regression completed 300 coverage calls in one
  transaction. The fail-capable fixture uses 512 graph rows with 256-neighbor
  toasted records. The standard PG18 test passes on the fix and fails on
  `fa84ff3b0^` after 300 calls with 1,258,283,008 bytes retained against its 4 MiB
  bound.
- The 10k/50k/100k matrix is waived conditionally because the production read
  path is unchanged; the fix is confined to the benchmark-only owner-seed
  diagnostic path. The conditional waiver lapses if that changes.
- The Task 188 `benchmark_backend_batch_size=5` setting remains only in its
  immutable historical diagnostic artifact; the active default is zero, and
  both clean Task 200 A1 and regression runs used one backend without the
  reconnect workaround. The historical packet was not rewritten, preserving
  its measurement provenance.

## Reviewer follow-up (2026-07-28)

The prior 300-call result was hand-run SQL evidence, not an executable
regression test. The new
`coverage_memory_regression_iterations` suite step runs the coverage helper on
one backend inside one transaction and enforces an RSS slope threshold. The
new gate is packet-configured at 300 calls with a 100 KB/s maximum slope and a
4,096 KB p01-to-p99 absolute delta. It warms the backend six times, settles for
one second, trims 40 samples from each edge, and records the full series. The
	live gate passed against the reused 100k three-owner fixture from the clean
	committed tree: 300 coverage calls, 300 returned rows, 16,586 RSS samples,
	stable p01-to-p99 delta 952 KB, and +1.10 KB/s slope. The older 1,024 KB/s
	result and the prior dirty-tree run are historical only. The unattended PG18
	test is enabled by the standard `pg_test` feature set and has both fixed-green
	and pre-fix-red evidence.

The Task 188-owned follow-up was subsequently rerun from fixed committed tree
`d845d8e4347d59dafd2b1ed28cd252d7d7c6e134` with stage counters on, batch size
zero, coverage enabled, both seed variants, no reconnect, and backend memory
sampling. The 100k run completed with `zero_fraction=0` and topology gate
passing. Both memory series showed only a 7–8 MB startup working-set rise and
constant HWM, with no multi-GB growth. The evidence is packet-local under
`reviews/task-200/003-fix-and-regression/artifacts/task188-fixed-no-reconnect-run-r2/`.
This supports removing the workaround, subject to Task 188 owner coordination;
Task 200 did not edit the Task 188 config.

## References

- `reviews/task-200/001-reproduction/` — Phase 1 series, memory-context dumps,
  reviewer feedback `2026-07-27-{01,02,03}-reviewer.md`.
- `reviews/task-200/002-attribution/feedback/2026-07-28-01-reviewer.md` —
  `TopTransactionContext` attribution and the invalid separate-statement test.
- `reviews/task-200/003-fix-and-regression/feedback/2026-07-28-01-reviewer.md` —
  the autocommit gap in the closeout claim.
- `9de8b4fa2` — `--memory-series-output` and `--reuse-fixture` in
  `ecaz bench suite`.
- `reviews/task-188/006-batch10-stage-counters/` — the original 52 GB outcome.
- Task 185 `df89b5726` / `c83ea6ea8` — the diagnostic-only statement-context
  fixes this now appears related to.
- NFR-007 benchmark provenance; FR-080, FR-082.
