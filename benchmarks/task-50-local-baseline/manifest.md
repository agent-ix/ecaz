# Task 50 Local Baseline Manifest

This packet captures the local "before" baseline for Task 50 unsafe-block
structural reduction. It runs once on the developer host before Slice 1a lands.
Per-slice packets cite their same-host before/after numbers against the
matching rows here. AWS m8g.2xlarge closeout confirmation lives elsewhere
(see `reviews/task-50/001-execution-planning/bench-baseline-plan.md`).

## Head and host

| Field | Value |
| --- | --- |
| HEAD SHA | `cc06046177ce63f969da51150d66a83260efe4d7` |
| Captured | `2026-05-19T09:49:59-07:00` (America/Los_Angeles) |
| Host | `DESKTOP-BMB4AFO` (WSL2 / Linux 6.6.114.1-microsoft-standard-WSL2) |
| CPU | Intel(R) Core(TM) i9-10900K @ 3.70GHz, 10 cores / 20 threads |
| Target features | `avx2 fma bmi1 bmi2 sse4_2 aes pclmulqdq rdrand` — **no AVX512** |
| Memory | 62 GiB total |
| OS | Ubuntu 22.04 (under WSL2 on Windows host) |
| Rust toolchain | (recorded inline in each bench log) |
| PostgreSQL | 18.3 (pgrx local install, socket `/home/peter/.pgrx`, port 28818) |
| Criterion sample size | default (100); `iai-callgrind` uses instruction-count |

## Corpus inventory

Canonical Task 50 profiles per `crates/ecaz-cli/src/commands/corpus/prepare.rs`:

| Profile | Corpus rows | Query rows | Local TSV state |
| --- | ---: | ---: | --- |
| `ec_real_10k` | 10,000 | 200 | reused from `/home/peter/dev/datasets/tqhnsw_real_10k/`; symlinked + canonical manifest under `target/real-corpus/staged-task50/` |
| `ec_real_25k` | 25,000 | 500 | newly prepared this packet (`corpus-prepare-ec_real_25k.log`) |
| `ec_real_50k` | 50,000 | 1,000 | reused from `/home/peter/dev/datasets/tqhnsw_real_50k/`; symlinked + canonical manifest |
| `ec_real_100k` | 100,000 | 1,000 | newly prepared this packet (`corpus-prepare-ec_real_100k.log`) |
| `ec_real_ann_benchmarks_anchor` | 990,000 | 10,000 | reused from `/home/peter/dev/datasets/tqhnsw_real_ann_benchmarks_anchor/`; symlinked + canonical manifest |

The reused prefixes (`tqhnsw_real_10k`, `tqhnsw_real_50k`,
`ec_hnsw_real_ann_benchmarks_anchor`) have identical selection rules
(corpus_rows, query_rows, query_start, sort_key) to the canonical
`ec_real_*` profiles. The staging directory carries symlinks for the TSVs
and rewritten manifests with the canonical prefix and filenames; the TSV
sha256s in each manifest are unchanged from the original capture.

## AM/storage matrix

Each profile is loaded under an isolated PG prefix per AM/storage to honor
the index-isolation rule
(`memory/feedback_index_isolation_rule.md`): one corpus table per variant
so the planner cannot pick across competing indexes.

| Surface label | Load args | PG prefix template |
| --- | --- | --- |
| `ec_ivf_rabitq` | `--profile ec_ivf --storage-format rabitq` | `<profile>_ivfrabitq` |
| `ec_spire_rabitq` | `--profile ec_spire --storage-format rabitq` | `<profile>_spirerabitq` |
| `ec_hnsw` | `--profile ec_hnsw` | `<profile>_hnsw` |
| `ec_diskann` | `--profile ec_diskann` | `<profile>_diskann` |

## Artifacts

| Artifact | Command | Status |
| --- | --- | --- |
| `unsafe-block-count-baseline.log` | `rg --count-matches 'unsafe\s*\{' src \| awk -F: '{printf "%4d %s\n", $2, $1}' \| sort -nr \| head -50` | captured; current total **3202** blocks across `src/`, top-15 unchanged from Task 35 close |
| `iai-quant-score.log` | `cargo bench --features bench --bench iai_quant_score` | _running_ |
| `iai-hadamard.log` | `cargo bench --features bench --bench iai_hadamard` | _pending_ |
| `iai-bitpack.log` | `cargo bench --features bench --bench iai_bitpack` | _pending_ |
| `criterion-quant-score.log` | `cargo bench --features bench --bench quant_score` | _pending_ |
| `criterion-hadamard.log` | `cargo bench --features bench --bench hadamard` | _pending_ |
| `criterion-page-codec.log` | `cargo bench --features bench --bench page_codec` | _pending_ |
| `corpus-prepare-ec_real_25k.log` | `ecaz corpus prepare --profile ec_real_25k ...` | _running_ |
| `corpus-prepare-ec_real_100k.log` | `ecaz corpus prepare --profile ec_real_100k ...` | _pending_ |
| `corpus-load-<profile>-<surface>.log` | `ecaz corpus load --prefix <profile>_<surface> --profile <am> [--storage-format <fmt>] --corpus-file ... --queries-file ... --manifest-file ...` | _pending (5 profiles × 4 surfaces = 20 loads)_ |
| `recall-<profile>-<surface>.log` | `ecaz bench recall --prefix <profile>_<surface> --profile <am>` | _pending_ |
| `latency-<profile>-<surface>.log` | `ecaz bench latency --prefix <profile>_<surface> --profile <am>` | _pending_ |
| `storage-<profile>-<surface>.log` | `ecaz bench storage --prefix <profile>_<surface>` (one per loaded prefix) | _pending_ |

This manifest is updated as each row completes. Any row that is unsupported
or operationally blocked is recorded here with the reason rather than
silently dropped, per the Task 50 local bench plan.

## SIMD note

Host exposes AVX2 + FMA but **no AVX512**. The `#[target_feature(enable =
"avx2,fma")]` paths in `src/quant/hadamard.rs` and `src/quant/prod.rs` are
exercised by this baseline; AVX512 paths (if any are added later) would
need a different host.

## Re-run

```sh
# kernel microbenches
cargo bench --features bench --bench iai_quant_score 2>&1 | tee benchmarks/task-50-local-baseline/artifacts/iai-quant-score.log
cargo bench --features bench --bench iai_hadamard    2>&1 | tee benchmarks/task-50-local-baseline/artifacts/iai-hadamard.log
cargo bench --features bench --bench iai_bitpack     2>&1 | tee benchmarks/task-50-local-baseline/artifacts/iai-bitpack.log
cargo bench --features bench --bench quant_score     2>&1 | tee benchmarks/task-50-local-baseline/artifacts/criterion-quant-score.log
cargo bench --features bench --bench hadamard        2>&1 | tee benchmarks/task-50-local-baseline/artifacts/criterion-hadamard.log
cargo bench --features bench --bench page_codec      2>&1 | tee benchmarks/task-50-local-baseline/artifacts/criterion-page-codec.log

# corpus prepare for missing profiles (10k/50k/990k reused via staged symlinks)
ecaz corpus prepare --profile ec_real_25k  --parquet /home/peter/dev/datasets/qdrant-dbpedia-entities-openai3-text-embedding-3-large-1536-1M/data --output-dir target/real-corpus/staged-task50
ecaz corpus prepare --profile ec_real_100k --parquet /home/peter/dev/datasets/qdrant-dbpedia-entities-openai3-text-embedding-3-large-1536-1M/data --output-dir target/real-corpus/staged-task50

# per-profile per-surface load + recall + latency + storage
# see this packet's artifacts/ for exact commands per row
```
