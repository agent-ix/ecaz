---
agent: codex
role: coder
model: GPT-5
date: 2026-07-28
seq: 2
---

# Task 200 fix and regression

The root cause is corrected in code checkpoint `fa84ff3b0`. In
`RetainedGenerationScan::seed_candidates`, the old
`value::<Vec<u8>>()` bytea conversion allocated detoast copies in
`TopTransactionContext` and retained them for the transaction. The fixed path
reads the raw SPI datum with `SpiTupleTable::get_datum_by_name`, decodes through
`DetoastedVarlena`, and frees each copy at the end of the row.

Definitive regression: 200 coverage calls in one `BEGIN`/`COMMIT` transaction
completed successfully on the reused 100k fixture. RSS stayed between 402,780
and 403,300 KB over 7m24s, and the final memory dump reported
`TopTransactionContext: 142606336 total`; the unfixed 20-call owner arm had
reached 5,595,201,536 bytes. The packet-local RSS series and SQL/context logs
are cited in the manifest.

The production A1 held-transaction run is also flat, so no 10/50/100k recall/
latency/storage matrix is required: the fix is in the benchmark-only
diagnostic owner seed path and leaves the production read path unchanged.
