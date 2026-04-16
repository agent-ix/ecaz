## Feedback: ADR-030 v2 Corrected SQL Overhead With Session Reuse

Read `scripts/bench_tqvector_sql_overhead_breakdown.sh` usage
at lines 17-51, mode validation at 233-240, and the
`plain-server` timing templates at 369-381, 415-427, and
436-446. Read the regression harness
`scripts/tests/test_bench_tqvector_sql_overhead_breakdown.py`.

### What's right

- **Fixing session shape is the right correction.** The
  per-query SQL leg vs. per-cell internal helpers mismatch was
  the real measurement bias in 365/366. Naming that and
  retracting the "3.7-6.8ms residual" framing at lines 136-157
  is the right editorial move; negative results are
  load-bearing here.
- **`plain-server` template is sound in shape.**
  `clock_timestamp()` around a `MATERIALIZED` ordered CTE
  (lines 369-381) forces the ordered scan under one bracketed
  timer, bypasses EXPLAIN JSON parsing, and keeps timing inside
  the same backend as the per-cell internal probes. Reusing
  the verified-launcher per-cell SQL-file strategy keeps the
  two harnesses comparable rather than forking.
- **Defaults preserved.** `per-query` + `explain` remain the
  defaults (lines 233-240), so existing callers keep their
  output shape and the new modes are strictly additive.
- **Fake-psql regression covers the new parsing paths.** The
  dedicated `per-cell plain-server` test plus `\o` redirection
  support in the fake `psql` is the right coverage level, given
  that end-to-end pg_test remains linker-blocked on the
  workstation.

### Concerns

1. **`ef=64` row shows a negative residual (`-0.056 ms`,
   lines 125-128).** At these sample sizes that is a noise
   reading, not evidence that SQL is faster than the AM-internal
   path. The packet's "essentially disappears" framing (line
   141) overstates what the table supports. Report a stddev or
   min/max column across the 50 per-cell samples, or qualify the
   conclusion as "residual is within measurement noise" instead
   of claiming parity. Without variance, the current table is
   equally consistent with "residual is zero" and "residual is
   ~0.1 ms drowned in per-query jitter," and those imply
   different follow-up work.

2. **`plain-server` timer brackets work the internal helper
   does not do.** The MATERIALIZED CTE forces a tuplestore
   write plus an outer `SELECT ... FROM started, finished`
   (lines 377-381 and 423-427), and the timer includes parse,
   rewrite, and planner cost for the wrapper. For ~ms-level
   residuals none of these are obviously zero. The
   "timed server-side" phrasing at line 56 should be qualified
   with a one-sentence note that the outer CTE is still inside
   the timed interval, otherwise the corrected residual is
   already an upper bound rather than a pure SQL-minus-AM delta.

3. **Sample size (50 queries) is borderline for sub-ms
   conclusions.** With typical PostgreSQL backend jitter in the
   0.1-0.3 ms range, the 95% CI on a 50-sample mean near 1 ms
   is roughly ±0.05-0.1 ms, which is the same magnitude as the
   "residual" rows. The retraction is directionally correct
   (the old multi-ms residual was not real), but the new
   positive claim — "SQL ≈ internal" — needs either a larger
   sample or an explicit variance report before 368/369 build
   on it.

4. **No repeatability check across runs.** One run per cell
   after the last correction; nothing in the packet shows the
   corrected numbers are stable across reruns on the same
   cluster. The branch already got surprised once by session
   shape; a second independent run on at least one cell would
   protect against a second-round revision.

5. **`--timing-mode` does not propagate to the internal or
   slot-fetch probes.** Those stay in their existing shape
   (lines 41-50 describe the switch only for the full SQL
   leg). That is fine for the correction goal, but the
   interpretation section should state it once so a reader
   does not assume all three probes now use the same timer.

6. **Required checkpoint still fails at the workstation
   linker layer** (lines 86-92). `cargo test` and
   `cargo pgrx test pg17` both fail with unresolved pgrx /
   Postgres symbols. This is now seven consecutive packets
   running green only on clippy plus harness-level Python
   tests. None of the recent scan / search changes
   (`src/am/scan.rs`, `src/am/search.rs`) have had a green
   pg_test run locally. Either a pointer to a green CI run or
   a dedicated linker-unblock packet is overdue; a real scan
   regression would currently not be caught.

### Measurement gaps still open

- Variance / percentile columns on the corrected cells
  (concern #1).
- Repeatability check on at least one cell (concern #4).
- pg_test green run on the required checkpoint (concern #6),
  carried forward from 359-366.

### Final framing

The packet's load-bearing claim — the large SQL residual from
366 was a per-query measurement artifact rather than server-side
overhead — is directionally well-supported and the retraction
language is honest. What is not yet shown is that the *positive*
claim ("SQL and internal are now equal") is statistically
robust. Means-only tables with one run per cell and 50 samples
are enough to invalidate the old 3.7-6.8 ms framing, but not
enough to support new architectural conclusions downstream.
Packets 368 and 369 should either cite variance from a rerun
or explicitly bound their own conclusions by the same
measurement-noise caveat.
