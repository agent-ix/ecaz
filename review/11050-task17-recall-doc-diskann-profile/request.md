# Review Request: RECALL_REAL_CORPUS doc — DiskANN profile + ecvector rename

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `docs/RECALL_REAL_CORPUS.md`

## What this packet is

The explicit follow-up deferred by packet
`review/11049-task17-loader-am-generic`: bring the real-corpus recall doc in
line with (a) the `tqvector` → `ecvector` / `ecaz` rename that landed on
`main`, and (b) the new `--index-profile` surface on
`scripts/load_real_corpus.py` so operators know DiskANN shares the same
staged corpus.

## What changed

### Schema section

- Corpus table's `embedding` column retyped from `tqvector` → `ecvector`.
- Population expression updated from `encode_to_tqvector(source, 4, 42)` →
  `encode_to_ecvector(source, 4, 42)`.
- Added a one-liner noting that the same `<prefix>_corpus` also serves
  `ec_diskann` indexes because both AMs share the `ecvector` embedding
  type.

### Legacy / coexisting index examples

- All four `USING ec_hnsw (embedding tqvector_ip_ops)` lines updated to
  `USING ec_hnsw (embedding ecvector_ip_ops)` — this matches the operator
  class actually registered by `sql/bootstrap.sql` in the current tree.

### New "Access-Method Profiles" section

Inserted between "How to Use" and "Diagnostics". Content:

- Documents `--index-profile {ec_hnsw,ec_diskann}` on
  `scripts/load_real_corpus.py` and the `--reloption key=value`
  passthrough added in packet 11049.
- Shows a worked example that reuses an already-staged
  `ec_hnsw_real_10k` corpus to build a DiskANN index with explicit
  `graph_degree` / `alpha` reloptions.
- Notes that `--m` is rejected on non-HNSW profiles (loader-side guard
  from packet 11049) and points at `list_size` /
  `ec_diskann.list_size` as the DiskANN tuning axis.
- Documents that `ecaz bench recall --profile ec_diskann` and
  `ecaz bench latency --profile ec_diskann` are the DiskANN recall /
  latency measurement lane, backed by the CLI's profile registry
  (`crates/ecaz-cli/src/profiles.rs`).
- Explicitly calls out that the existing A4 SQL gate surfaces
  (`ec_hnsw_graph_scan_recall_external_*`) remain HNSW-only so readers
  do not try to point them at an `ec_diskann` index.

### Troubleshooting stale-name fix

The "Scratch DB missing functions" section referred to `tqvector`
"installed from an older pg_test build". The fix block already used
`DROP EXTENSION IF EXISTS ecaz`, so the leading prose was the only stale
bit — updated to `ecaz` to match.

## Why this slice

The next task-17 user deliverable is DiskANN recall/latency evaluation on
the same staged corpora HNSW uses. Packet 11049 made the loader AM-generic
but intentionally left the doc stale because a DiskANN recall doc note made
more sense to land alongside the DiskANN story than as a trailing edit.
This is that note. After this packet, an operator can read
`docs/RECALL_REAL_CORPUS.md` front-to-back and follow it all the way from
parquet → staged TSV → `<prefix>_corpus` → both HNSW and DiskANN indexes →
`ecaz bench recall` / `ecaz bench latency` without consulting another doc.

## Test evidence

Doc-only change; no executable surface to run. Manually cross-checked
claims against:

- `crates/ecaz-cli/src/profiles.rs` — `ef_search_guc` values (`ec_hnsw.ef_search`,
  `ec_diskann.list_size`), operator classes, encoder function.
- `crates/ecaz-cli/src/commands/bench/recall.rs` — `ecaz bench recall`
  accepts `--profile ec_diskann` and sweeps the profile's `ef_search_guc`.
- `scripts/load_real_corpus.py` — `--index-profile`, `--reloption`, and
  the "no `--m` on non-HNSW profiles" guard from packet 11049.
- `sql/bootstrap.sql` — `ecvector_ip_ops` / `ecvector_diskann_ip_ops` are
  the opclass names in the current tree.

## Follow-ups intentionally not in this packet

- `scripts/bench_sql_latency.sh`, `scripts/vacuum_concurrency_scratch.sh`,
  `scripts/bench_tqvector_sql_overhead_breakdown.sh` still carry
  pre-rename `tqvector` / `encode_to_tqvector` references. Still deferred
  — a per-script port/update decision (keep HNSW-only vs. port to
  AM-generic via the ecaz CLI) is large enough to own its own packet.
- The A4 SQL gate surface for DiskANN (an equivalent of
  `ec_hnsw_graph_scan_recall_external_gate_report` for `ec_diskann`) is a
  separate lane and not in scope here; the doc currently directs DiskANN
  recall evaluation through `ecaz bench recall` because that CLI verb
  already exists on `main`.
