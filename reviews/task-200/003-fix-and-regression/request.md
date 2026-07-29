---
agent: codex
role: coder
model: GPT-5
date: 2026-07-28
seq: 3
---

# Task 200 fix and executable regression

The root cause is corrected in code checkpoint `fa84ff3b0`. In
`RetainedGenerationScan::seed_candidates`, the old `value::<Vec<u8>>()` bytea
conversion allocated detoast copies in `TopTransactionContext` and retained
them for the transaction. The fixed path reads the raw SPI datum with
`SpiTupleTable::get_datum_by_name`, decodes through `DetoastedVarlena`, and
frees each copy at the end of the row.

The final regression was run from a clean worktree at the committed fix. The
reused 100k fixture reported `ecaz_build_git_sha() =
fa84ff3b06bccec2a8f202338003da489a5ca105` and `ecaz_build_profile() = release`.
Three hundred coverage calls completed in one `BEGIN`/`COMMIT` transaction.
RSS samples ranged from 401,820 to 402,648 KB, with a fitted slope of +1.42
KB/s and no monotonic growth. The final dump reported
`TopTransactionContext: 142606336 total` and `Grand total: 144745408 bytes`.
The unfixed 20-call owner arm reached `TopTransactionContext: 5595201536`.

The clean-provenance production A1 also completed 300 ordinary ANN queries in
one held transaction. Its RSS rose during initial setup from 251,892 to
260,780 KB, then remained at 260,780 KB from 7.3 seconds through the end of
the 10.8-second series; it showed no unbounded per-query growth. The fix is in
the benchmark-only diagnostic owner seed path and leaves the production read
path unchanged, so the conditional 10/50/100k matrix waiver applies.

Reviewer follow-up identified that the SQL loop above was hand-run evidence,
not an executable regression test. The executable gate is now run and passes.
It reused the packet-local 100k fixture bootstrap and existing staged corpus;
the gate did not rebuild the corpus or index. After six warm-up invocations and
a one-second settle, it executed 300 coverage invocations on one coordinator
backend in one transaction. It returned 300 rows and collected 16,569 RSS
samples. On the stable interior, the fitted slope was +1.02 KB/s and the
absolute p01-to-p99 RSS delta was 1,020 KB, both below the committed limits of
100 KB/s and 4,096 KB. The full series and normalized suite results are
packet-local evidence.

The suite runner also now forwards `reuse_provenance_dir`, so reuse is
attested from the packet-local bootstrap result instead of rebuilding. A clean
committed-tree rerun reused that fixture and passed the shipped statistic with
`rss_p01_to_p99_kb=952`, `rss_slope_kb_per_s=1.10`,
`max_delta_kb=4096`, and `max_slope_kb_per_s=100`.

The required pre-fix PG18 control uses the same standard command and the same
fail-capable fixture: it fails after 300 calls with 1,258,283,008 bytes of
retained growth versus the 4 MiB assertion. The fixed standard command passes
with the 512-row, 256-neighbor toasted fixture. The preserved executable
negative control predates the shipped warm-up/percentile statistic, but its
98,380.15 KB/s slope and 245,576 KB raw delta are respectively about 984x and
60x beyond the shipped limits, so the new bound cannot plausibly mask it. A
second 100k pre-fix fixture build was not performed solely to restage identical
data. Because `pg_test` includes `distann-head-attribution-benchmark`, the
ordinary `cargo pgrx test pg18 ... --no-default-features` invocation exercises
the regression test without an extra feature flag. The older 1,024 KB/s
executable result remains in the packet only as historical context.
The sibling conversion audit remains in `artifacts/sibling-conversion-audit.md`.

The sibling conversion audit is in
`artifacts/sibling-conversion-audit.md`; it covers the production
`remote_endpoint.rs:538` array conversion and the corresponding
`generation_read.rs:1327` site, with A1 as the boundedness evidence.

## Task 188 follow-up

The previously missed follow-up was rerun from the fixed committed tree using
`artifacts/task188-fixed-no-reconnect-suite.json`. The run kept stage counters
enabled, set `benchmark_backend_batch_size=0`, omitted `skip_recall` so the
coverage statement executed, used both Task 188 seed variants, sampled backend
memory, and did not reconnect the backend. It completed successfully on the
100k three-owner fixture:

- coverage: 200 queries, `zero_fraction=0`, topology gate passed;
- `bw4-control`: 6 samples, RSS `246792 -> 254764 KB`, HWM fixed at
  `378020 KB`;
- `bw8-candidate`: 5 samples, RSS `249388 -> 256636 KB`, HWM fixed at
  `379348 KB`.

The only observed rise is the small startup working-set change; the prior
multi-GB growth did not recur with batch size zero and no reconnect. The
workaround therefore appears removable, but Task 200 leaves Task 188's config
unchanged pending confirmation with its owner. Full evidence is in
`artifacts/task188-fixed-no-reconnect-run-r2/` and the concise outcome is in
`artifacts/task188-fixed-no-reconnect-outcome.md`.
