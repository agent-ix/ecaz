# Review Request: bench / vacuum scripts — ecvector rename

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `scripts/vacuum_concurrency_scratch.sh`
- `scripts/bench_sql_latency.sh`

## What this packet is

Two of the pre-rename scripts flagged by packet 11049's follow-up list get
updated in place so their embedded SQL uses the current ecvector type,
encoder, and operator class. The third flagged script
(`scripts/bench_tqvector_sql_overhead_breakdown.sh`) is intentionally left
for a separate packet — see Follow-ups.

These scripts are still HNSW-only harnesses; they are not being ported to
the AM-generic ecaz CLI here. The scope is purely "make them run on
post-rename main" so the existing lanes stay functional while the DiskANN
measurement lane lands alongside.

## What changed

### `scripts/vacuum_concurrency_scratch.sh`

Five embedded-SQL occurrences updated:

- Column type `tqvector` → `ecvector` in the harness corpus DDL (line ~137).
- Encoder calls `encode_to_tqvector(...)` → `encode_to_ecvector(...)` for
  both the seed insert and the concurrent insert worker (lines ~140, ~161).
- Operator class `tqvector_ip_ops` → `ecvector_ip_ops` on both HNSW index
  creations (lines ~152, ~262).

No behavior change: the script's concurrency harness shape, vacuum worker,
and assertion logic are identical. It simply uses the type/encoder names
the extension ships today.

### `scripts/bench_sql_latency.sh`

Synthetic-fixture mode and the two banner prints updated:

- Two banner strings `=== tqvector SQL ... ===` → `=== ecvector SQL ... ===`.
- Synthetic-mode encode step: `encode_to_tqvector(...)` → `encode_to_ecvector(...)`.
- Synthetic-mode index DDL: `USING ec_hnsw (vec tqvector_ip_ops)` →
  `USING ec_hnsw (vec ecvector_ip_ops)`.

The real-corpus mode path was already schema-clean (it reads tables
produced by `load_real_corpus.py`, which is on ecvector already via packet
11049). The `PGDATABASE=tqvector_bench` default and the `PGDATABASE`
example in the header comment are left as-is: that is a conventional
database name, not a schema object, and changing it would churn docs in
`RECALL_REAL_CORPUS.md` that still use the same convention.

## Why this slice

Packet 11049 made the loader AM-generic but explicitly left these scripts
for follow-up because each needs a per-script "port to AM-generic CLI vs.
update in place" judgement call. For these two the call is "update in
place": both are HNSW-specific harnesses (vacuum concurrency, synthetic
HNSW latency) and have no DiskANN equivalent today, so rewriting them
against the ecaz CLI's profile layer would be scope creep without a
user-visible win. The rename makes them runnable on current main — which
is what blocks using them to cross-check DiskANN work against the existing
HNSW baseline.

## Test evidence

Static — both scripts require a running scratch pg17 cluster and the
ecaz extension. Grepped post-edit:

```
$ rg 'tqvector|encode_to_tqvector' scripts/vacuum_concurrency_scratch.sh
(no matches)

$ rg 'tqvector' scripts/bench_sql_latency.sh
11:#       PGDATABASE=tqvector_bench bash scripts/bench_sql_latency.sh
594:PGDATABASE="${PGDATABASE:-tqvector_bench}"
```

Remaining matches are the `tqvector_bench` database-name convention,
intentional per "What changed" above.

Manually cross-checked:

- `ecvector` type, `encode_to_ecvector` SQL function, and
  `ecvector_ip_ops` opclass are the current names registered by
  `sql/bootstrap.sql`.
- `scripts/load_real_corpus.py` emits the same `ecvector_ip_ops` opclass
  via the `ec_hnsw` profile (packet 11049), so harnesses sharing a corpus
  with the loader now line up.

## Follow-ups intentionally not in this packet

- `scripts/bench_tqvector_sql_overhead_breakdown.sh` (691 lines) still
  references `encode_to_tqvector` / `tqvector`-named tables. Deferred to
  its own packet because (a) the filename itself bakes the old name into
  the repo and the rename question is larger than just the embedded SQL,
  and (b) the script's "encode + internal-scan + residual" decomposition
  may want to grow a DiskANN counterpart anyway — mechanical rename now
  would churn the file a second time shortly.
- No CLI port. A DiskANN vacuum / overhead harness is a separate task-17
  slice; cutting over these HNSW-only scripts here would delete working
  lanes without a replacement.
