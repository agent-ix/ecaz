## Feedback: ADR-030 v2 Matched-Session SQL Comparison

Read `scripts/bench_sql_latency_verified.sh` and
`scripts/bench_pgvector_sql_latency.sh` (both support the same
`--session-mode per-cell --timing-mode plain-server` contract
used here), cross-checked the cited recall tables in packets
362 and 363, and compared the reported SQL means table at
lines 133-139.

### What's right

- **Matching the session shape before comparing is the right
  move.** The old 364 comparison was between a per-query SQL
  harness on both sides and concluded pgvector was faster at
  the SQL layer; once 367 showed the per-query shape itself
  was the dominant cost, the symmetric rerun was the only
  honest next step. Doing it as a pure measurement packet,
  no code changes, keeps the correction narrow.
- **Same query subset on both sides.** Both harnesses use
  `tqhnsw_real_50k_queries_50` and both cells were timed with
  `per-cell plain-server` (lines 76-89 and 106-117). That is
  the only symmetry that matters for the claim "tqvector SQL
  is faster at matched shape."
- **Recall context is explicitly separated from the SQL
  timing correction.** Lines 142-163 reuse the existing recall
  tables rather than claiming this packet measured recall.
  Right scope discipline — this packet is not re-validating
  recall and should not be read as doing so.
- **Conclusion is correctly scoped to the lane.** Lines 170-187
  explicitly name this as the "isolated grouped-v2 `m=16` lane"
  rather than a general tqvector-vs-pgvector verdict.

### Concerns

1. **Recall table attribution conflicts with packet 369's
   recall table for the same lane.** Lines 149-154 cite "packet
   362" and report tqvector grouped `m=16` recall as
   `0.900 / 0.930 / 0.936 / 0.938`. Packet 369 cites "packet
   363" for the same lane and reports
   `0.9200 / 0.9380 / 0.9400 / 0.9460`. Both are grouped `m=16`
   on "the same 50-query subset." Packet 362's request at
   `review/362-.../request.md:86-87` does show
   `0.900 / 0.930 / 0.936 / 0.938` and packet 363's table at
   lines 136-139 does show `0.9200 / 0.9380 / 0.9400 / 0.9460`,
   so both citations are internally accurate — but the
   underlying recall readings on an ostensibly identical lane
   differ by up to 2.2 points of Recall@10 (11 matches out of
   500). That is well above the 1/500 = 0.002 resolution of
   a 50×10 recall measurement. Either the grouped build was
   non-deterministic between 362 and 363, or the corpus /
   seed / codebook differed, or one of the two runs used a
   different truth table. Before drawing any "tqvector vs
   pgvector" latency/recall conclusion from this packet,
   reconcile which recall table is authoritative and record
   the reason for the drift. ADR-030 v2's "deterministic
   grouped graph build" slice (packet 361) is supposed to have
   removed this kind of run-to-run variance; if it did not, the
   determinism claim itself is invalidated by the discrepancy.

2. **Only means reported; no variance, no percentiles.** Same
   concern as 367: at mean ≈ 1-6 ms with 50 samples per cell,
   the 95% CI on the mean is within the reported deltas at
   several cells. The `ef=64` cell in particular — tqvector
   `1.525 ms` vs pgvector `1.775 ms`, a `1.164x` ratio — is
   the weakest row and the packet draws the headline
   conclusion ("tqvector is faster at every measured
   `ef_search` point") from it. Report stddev / p50 / p95 per
   cell, or the "tqvector faster everywhere" framing is not
   safely supported at `ef=64`.

3. **No plan snapshot on either side.** The packet assumes
   both SQL runs hit the expected AM. Packet 363's caveat at
   line 157-161 explicitly notes that the tqvector plain-SQL
   path fell back to `SeqScan` even with `enable_seqscan = off`
   on the isolated grouped table; the verified launcher was
   used only because it forces the AM path. There is no
   equivalent plan-check statement for pgvector here. A
   one-line `EXPLAIN` dump per side (or a single assertion
   that both plans are `Index Scan using <idx>`) would close
   the loop; without it, "tqvector SQL faster" could be
   comparing an index scan against a sequential scan on the
   pgvector side if its planner happened to prefer seqscan at
   that `ef_search` / LIMIT.

4. **Single run per cell.** No repeatability check. The
   corrected measurement surface just landed in 367; the first
   derived conclusion from it should not be a one-shot
   comparison. At least one cell rerun per side would
   strengthen the claim.

5. **No-code-change packet still depends on pg_test state
   that has not been verified.** The harnesses used
   (`bench_sql_latency_verified.sh`,
   `bench_pgvector_sql_latency.sh`) were validated in earlier
   packets where the `cargo pgrx test pg17` chain was already
   linker-blocked on this workstation. Not a new defect
   introduced here, but inherited risk: if either launcher has
   an undetected regression under the `per-cell plain-server`
   path, it would land silently in this packet's tables.

6. **The headline table's `pgvector / tqvector` ratio column
   at lines 134-139 presents 4 ratios as if equally load-
   bearing.** The smallest (`ef=64`, `1.164x`) is inside the
   likely measurement-noise band for this sample size, while
   the largest (`ef=40`, `1.711x`) is more credible. Either
   present a CI alongside each ratio or qualify which ratios
   are above noise; otherwise the "faster at every measured
   point" framing treats a noisy row as equivalent evidence to
   a clear one.

### Measurement gaps still open

- Reconciled recall attribution for tqvector grouped `m=16`
  on this subset (concern #1). This blocks any clean
  operating-point reading downstream, including packet 369.
- Per-cell variance / percentiles on both sides (concern #2).
- `EXPLAIN` check that both sides used index scans
  (concern #3).
- One repeatability rerun (concern #4).

### Final framing

The directional conclusion — matched-session SQL timing
reverses packet 364's "pgvector faster at SQL" claim on the
isolated grouped-v2 `m=16` lane — is plausible and falls out
of 367's correction. The packet is useful as a retraction of
364. What it does not yet support cleanly is the stronger
claim used by packet 369 ("tqvector owns the ultra-low-latency
corner, pgvector owns above it"), because the recall numbers
underpinning 369's Pareto analysis diverge from the recall
numbers cited in this packet for the same lane. Until that
recall discrepancy is reconciled, the operating-point verdict
built on top of 368+369 is unsafe even though the SQL timing
table here looks clean.
