# Feedback: 621 Parallel Index Build Phase Timing

## Verdict: Accept

Phase timing surface is correctly structured and covers the right boundaries.

## Findings

**AtomicU64 globals**: The `LAST_BUILD_*` statics with `Acquire`/`Release`
ordering are correct for a single-backend debug surface. The leader writes all
counters in one `record_debug_last_build_timing` call after the build
completes, so there is no cross-backend race to worry about.

**Phase naming**: The split between `parallel_begin_us`, `parallel_drain_us`,
and `parallel_sort_push_us` is the right breakdown. `parallel_begin_us` covers
DSM setup and worker launch; `parallel_drain_us` covers queue read and finish;
`parallel_sort_push_us` covers leader-side TID sort and `BuildState::push`. These
three are distinct cost centres that need separate evidence.

**`heap_ingest_us`**: Covers the full ingestion wall time (serial or parallel
begin-to-ingest-done). For the serial path this is table_index_build_scan
time; for the parallel path it subsumes begin + drain + sort_push. The outer
counter lets the measurement compare serial and parallel ingestion as a unit.

**`flush_total_us` vs `graph_us` + `stage_us` + `write_us`**: Having both the
total and the decomposed phases is useful — it lets the measurement verify that
`graph + stage + write ≈ flush_total` as a consistency check.

**SQL surface is pg_test-only**: Keeping the timing read function in the
`tests.` schema rather than the public API is correct at this stage. Debug
surfaces that expose internal timing should not become public contract.

**Smoke test extension**: Asserting the timing surface is populated for the
worker path (workers_launched > 0, drain_us > 0) is the right regression
gate. It ensures the timing instrumentation is actually wired and not silently
returning zeros.

**`elapsed_us` ceiling**: Returns `u64::MAX` on overflow of `as_micros()`.
Safe; a build taking more than ~584,542 years would saturate rather than wrap.
The `u64::MAX` sentinel is distinguishable from a real measurement.

## No Issues
