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
The suite reused a newly bootstrapped 100k fixture from the existing staged
corpus and executed 300 coverage invocations on one coordinator backend in
one transaction. It returned 300 rows, collected 16,609 RSS samples, and
reported a fitted slope of +5.82 KB/s against the 1,024 KB/s limit. The
packet-local series and normalized suite results are committed as evidence.

The suite runner also now forwards `reuse_provenance_dir`, so reuse is
attested from the packet-local bootstrap result instead of rebuilding. The
executable gate now has a 100 KB/s default calibrated from the fixed +1.42 and
+5.82 KB/s observations. The same gate fails against `fa84ff3b0^` at
98,380.15 KB/s, proving the assertion detects the original defect. The
unattended PG18 test passes 300 one-owner conversions with a 4 MiB bounded
growth assertion; its output is in `artifacts/pg18-seed-memory-regression.log`.
The sibling conversion audit remains in `artifacts/sibling-conversion-audit.md`.

The sibling conversion audit is in
`artifacts/sibling-conversion-audit.md`; it covers the production
`remote_endpoint.rs:538` array conversion and the corresponding
`generation_read.rs:1327` site, with A1 as the boundedness evidence.
