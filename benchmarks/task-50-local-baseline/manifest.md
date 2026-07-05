# Task 50 Local Baseline Manifest

Local "before" baseline for Task 50 unsafe-block structural reduction. Per-slice
packets cite their same-host before/after numbers against the matching rows
here. AWS m8g.2xlarge closeout confirmation lives elsewhere (see
`reviews/task-50/001-execution-planning/bench-baseline-plan.md`).

## Head and host

| Field | Value |
| --- | --- |
| HEAD SHA | `cc06046177ce63f969da51150d66a83260efe4d7` |
| Captured | `2026-05-19` (America/Los_Angeles) |
| Host | `DESKTOP-BMB4AFO` (WSL2 / Linux 6.6.114.1-microsoft-standard-WSL2) |
| CPU | Intel(R) Core(TM) i9-10900K @ 3.70GHz, 10 cores / 20 threads |
| Target features | `avx2 fma bmi1 bmi2 sse4_2 aes pclmulqdq rdrand` — **no AVX512** |
| Memory | 62 GiB |
| OS | Ubuntu 22.04 (under WSL2) |
| PostgreSQL | 18.3 (pgrx local install, socket `/home/peter/.pgrx`, port 28818) |
| Criterion sample size | default (100) |
| Valgrind | 3.18.1 (user-prefix install, shimmed via `~/.local/bin/valgrind`) |

## Re-run

The canonical runner is **`ecaz bench suite`** with the checked-in config:

```sh
ecaz bench suite run --config benchmarks/task-50-local-baseline/suite.json
```

The suite expands to **73 steps** across `corpus-prepare`, `load`, `recall`,
`latency`, `storage`. Kernel microbenches and unsafe-block-count are captured
separately by the commands listed below; they are reproducible without the
suite runner. The suite runner replaces the throwaway `run-matrix.sh` that was
used for the first capture (now removed).

Kernel microbenches:

```sh
PATH=$HOME/.local/bin:$HOME/.cargo/bin:$PATH \
  cargo bench --features bench --bench iai_quant_score 2>&1 | tee benchmarks/task-50-local-baseline/artifacts/iai-quant-score.log
PATH=$HOME/.local/bin:$HOME/.cargo/bin:$PATH \
  cargo bench --features bench --bench iai_hadamard    2>&1 | tee benchmarks/task-50-local-baseline/artifacts/iai-hadamard.log
PATH=$HOME/.local/bin:$HOME/.cargo/bin:$PATH \
  cargo bench --features bench --bench iai_bitpack     2>&1 | tee benchmarks/task-50-local-baseline/artifacts/iai-bitpack.log
cargo bench --features bench --bench quant_score 2>&1 | tee benchmarks/task-50-local-baseline/artifacts/criterion-quant-score.log
cargo bench --features bench --bench hadamard    2>&1 | tee benchmarks/task-50-local-baseline/artifacts/criterion-hadamard.log
cargo bench --features bench --bench page_codec  2>&1 | tee benchmarks/task-50-local-baseline/artifacts/criterion-page-codec.log
```

Unsafe-block-count baseline:

```sh
rg --count-matches 'unsafe\s*\{' src \
  | awk -F: '{printf "%4d %s\n", $2, $1}' | sort -nr | head -50 \
  > benchmarks/task-50-local-baseline/artifacts/unsafe-block-count-baseline.log
```

## Corpus inventory

Canonical Task 50 profiles per `crates/ecaz-cli/src/commands/corpus/prepare.rs`:

| Profile | Corpus rows | Query rows | Local TSV |
| --- | ---: | ---: | --- |
| `ec_real_10k` | 10,000 | 200 | reused from `/home/peter/dev/datasets/tqhnsw_real_10k/`; symlinked + canonical-name manifest in `target/real-corpus/staged-task50/` |
| `ec_real_25k` | 25,000 | 500 | newly prepared this packet from DBpedia parquet (`corpus-prepare-ec_real_25k.log`) |
| `ec_real_50k` | 50,000 | 1,000 | reused from `/home/peter/dev/datasets/tqhnsw_real_50k/`; symlinked + canonical-name manifest |
| `ec_real_100k` | 100,000 | 1,000 | newly prepared this packet from DBpedia parquet (`corpus-prepare-ec_real_100k.log`) |
| `ec_real_ann_benchmarks_anchor` | 990,000 | 10,000 | reused from `/home/peter/dev/datasets/tqhnsw_real_ann_benchmarks_anchor/`; symlinked + canonical-name manifest |

Reused prefixes (`tqhnsw_real_*`, `ec_hnsw_real_ann_benchmarks_anchor`) have
identical selection rules (corpus_rows, query_rows, query_start, sort_key) to
the canonical `ec_real_*` profiles. The staging directory carries symlinks for
the TSVs and rewritten manifests with the canonical prefix and filenames; the
TSV sha256s recorded in each manifest are unchanged.

## AM/storage matrix

Each profile is loaded under an isolated PG prefix per AM/storage to honor the
index-isolation rule (one corpus table per variant so the planner cannot pick
across competing indexes).

| Surface label | Load args | PG prefix template |
| --- | --- | --- |
| `ec_ivf_rabitq` (`ivfrabitq`)   | `--profile ec_ivf --storage-format rabitq`   | `<profile>_ivfrabitq` |
| `ec_spire_rabitq` (`spirerabitq`) | `--profile ec_spire --storage-format rabitq` | `<profile>_spirerabitq` |
| `ec_hnsw` (`hnsw`)              | `--profile ec_hnsw --m 8,16 --ef-construction 128` | `<profile>_hnsw` |
| `ec_diskann` (`diskann`)        | `--profile ec_diskann`                       | `<profile>_diskann` |

## Per-cell status

Legend: `✓` captured ok; `✗` failed and recorded; `—` not applicable.

| profile | surface | load | recall | latency | storage | note |
| --- | --- | :-: | :-: | :-: | :-: | --- |
| `ec_real_10k`   | ivfrabitq   | ✓ | ✓ | ✓ | ✓ | |
| `ec_real_10k`   | spirerabitq | ✓ | ✓ | ✓ | ✓ | |
| `ec_real_10k`   | hnsw        | ✓ | ✓ | ✓ | ✓ | |
| `ec_real_10k`   | diskann     | ✓ | ✓ | ✓ | ✓ | |
| `ec_real_25k`   | ivfrabitq   | ✓ | ✓ | ✓ | ✓ | storage backfilled after fix |
| `ec_real_25k`   | spirerabitq | ✓ | ✓ | ✓ | ✓ | storage backfilled after fix |
| `ec_real_25k`   | hnsw        | ✓ | ✓ | ✓ | ✓ | |
| `ec_real_25k`   | diskann     | ✓ | ✓ | ✓ | ✓ | |
| `ec_real_50k`   | ivfrabitq   | ✓ | ✓ | ✓ | ✓ | storage backfilled after fix |
| `ec_real_50k`   | spirerabitq | ✗ | — | — | — | **deferred (known)** — `ec_spire object tuple payload 11270 exceeds page size 8192`, SPIRE rabitq ambuild bug at 50k+; out of Task 50 scope |
| `ec_real_50k`   | hnsw        | ✓ | ✓ | ✓ | ✓ | |
| `ec_real_50k`   | diskann     | ✓ | ✓ | ✓ | ✓ | |
| `ec_real_100k`  | ivfrabitq   | ✓ | ✓ | ✓ | ✓ | |
| `ec_real_100k`  | spirerabitq | ✗ | — | — | — | **deferred (known)** — same SPIRE rabitq ambuild bug |
| `ec_real_100k`  | hnsw        | ✓ | ✓ | ✓ | ✓ | |
| `ec_real_100k`  | diskann     | ✓ | ✓ | ✓ | ✓ | |
| `ec_real_ann_benchmarks_anchor` (990k) | ivfrabitq   | ✓ | ✓† | ✓ | ✓ | †recall initially OOM with 10k queries; rerun with `--queries-limit 1000` |
| `ec_real_ann_benchmarks_anchor` (990k) | spirerabitq | ✗ | — | — | — | **deferred (known)** — same SPIRE rabitq ambuild bug, hit after long codebook training |
| `ec_real_ann_benchmarks_anchor` (990k) | hnsw        | ✓ | ✓† | ✓ | ✓ | †same `--queries-limit 1000` rerun |
| `ec_real_ann_benchmarks_anchor` (990k) | diskann     | ✗ | — | — | — | **deferred (slow)** — diskann ambuild was canceled via `pg_cancel_backend` after 4h00m at 93% CPU; not blocking Task 50 since smaller-corpus diskann rows captured cleanly |

### Deferred / known issues

1. **SPIRE rabitq ambuild fails at 50k+ rows** with
   `ec_spire object tuple payload 11270 exceeds page size 8192`. Known product
   bug, deferred per coder-reviewer chat. Not a baseline blocker; the SPIRE
   row at each profile size ≥50k is recorded as deferred-known in the table
   above. The 10k and 25k SPIRE rows captured cleanly and are the durable
   SPIRE-RaBitQ baseline for Task 50 closeout comparisons.

2. **`ecaz bench storage` row-count failed on ec_ivf surfaces at 25k+** with
   `ec_ivf scan currently requires exactly one ORDER BY query`.
   Fixed in this packet: the storage command now runs the row count inside a
   transaction that disables index/index-only/bitmap scans, forcing the
   sequential-scan plan and bypassing the AM. See
   `crates/ecaz-cli/src/commands/bench/storage.rs`. The three previously
   failing rows (`storage-ec_real_25k-ivfrabitq`,
   `storage-ec_real_25k-spirerabitq`, `storage-ec_real_50k-ivfrabitq`) were
   backfilled after the rebuild.

3. **990k recall OOM at full 10k-query default** — exhaustive ground truth
   builds a dense distance matrix of
   `10000 queries × 990000 corpus × 4 B ≈ 40 GiB`, which on this WSL2 host
   blows past the OOM-killer threshold (anon-rss climbed to 55.7 GiB before
   kill; confirmed in `dmesg`). The 990k recall rows are captured with
   `--queries-limit 1000`, dropping peak memory to ~10 GiB (corpus + 1k × 990k
   matrix). A streaming-truth refactor of `ecaz bench recall` (per-query top-k
   heap instead of dense matrix) is the proper fix; tracked as a separate
   harness packet outside Task 50 scope.

4. **`benches/criterion/quant_score.rs` panicked on dims other than 1536** for
   the three no-QJL 4-bit groups. Fixed in this packet: those bench groups
   now iterate only `dim=1536` since the `prepare_ip_query_*_no_qjl_4bit`
   kernels assert `rotation::tile_dim(dim).is_some()`, which today is only
   true for `TILED_FWHT_COMPAT_DIM = 1536`.

5. **`ecaz bench suite` LoadStep had no `--storage-format` passthrough**.
   Fixed in this packet: added `storage_format: Option<String>` to the
   `LoadStep` config schema and emitted `--storage-format <value>` when set.
   This is what lets `suite.json` drive the IVF/SPIRE rabitq surfaces without
   forking into a script.

### Kernel microbench artifacts

| Artifact | Status |
| --- | --- |
| `iai-quant-score.log`        | ✓ captured (3 cases) |
| `iai-hadamard.log`           | ✓ captured (2 cases) |
| `iai-bitpack.log`            | ✓ captured (3 cases) |
| `criterion-quant-score.log`  | ✓ captured (22 cases, including throughput) |
| `criterion-hadamard.log`     | ✓ captured |
| `criterion-page-codec.log`   | ✓ captured |
| `unsafe-block-count-baseline.log` | ✓ captured; current total **3202** unsafe blocks across `src/`, top file `src/am/ec_hnsw/scan_debug.rs` (356) |

### Operational artifacts in `artifacts/`

- `matrix-status.tsv` — per-step status from the original script-driven capture; kept for the initial-capture audit trail.
- `suite-manifest-rerun-10-50-100k.json` and `results-rerun-10-50-100k.jsonl` — **authoritative status and result rows** for the 10k / 50k / 100k bench cells (recall + latency + storage × {ec_ivf rabitq, ec_hnsw, ec_diskann}, 27 steps, all Succeeded). Captured by `ecaz bench suite run --only ...` against the rebuilt ecaz-cli (storage fix + LoadStep storage_format extension).
- `suite-rerun-10-50-100k.driver.log` — stdout of the suite re-run.
- `corpus-prepare-<profile>.log` — capture log for the two profiles newly prepared this packet (`ec_real_25k`, `ec_real_100k`).
- `corpus-load-<profile>-<surface>.log`, `recall-<profile>-<surface>.log`, `latency-<profile>-<surface>.log`, `storage-<profile>-<surface>.log` — per-cell logs for every captured row above.

## SIMD note

Host exposes AVX2 + FMA but **no AVX512**. The `#[target_feature(enable =
"avx2,fma")]` paths in `src/quant/hadamard.rs` and `src/quant/prod.rs` are
exercised by this baseline; AVX512 lanes (if any are added later) would need a
different host.

## Open items

- 990k diskann ambuild was canceled after 4h00m at 93% CPU to stay within the
  wrap window; the row is marked deferred above. Suite re-runs that have the
  patience to wait can complete it; Task 50 closeout does not require it
  since smaller-corpus diskann rows captured cleanly.
- Streaming-truth refactor of `ecaz bench recall` (per-query top-k heap) is
  the right fix for the 990k OOM; deferred to a separate harness packet.
- The SPIRE rabitq tuple-size bug is a known-product issue and is owned by the
  SPIRE rabitq team, not Task 50.
