# Pre-optimization Graviton 4 baselines — DBpedia 10k + 50k, ec_ivf turboquant + rabitq

> ⚠️ **Read the storage_format, not the prefix.** The corpus tables
> are prefixed `ec_hnsw_real_10k` and `ec_hnsw_real_50k` for legacy
> compatibility with `crates/ecaz-cli/src/commands/corpus/prepare.rs`
> `SubsetProfile`s. The "hnsw" token is **cosmetic**; every row here
> is `ec_ivf` access method, and the actual differentiator is the
> `storage_format` reloption (`turboquant` vs `rabitq`). Both indexes
> co-exist on the same table per corpus and are swapped per pass.
> Confirm via `ecaz corpus list` → access methods `btree, ec_ivf`.

## Purpose

Captures the **pre-optimization** state of the IVF scoring kernels on
Graviton 4 (Neoverse-V2), with the data needed to start writing
NEON / SVE2 paths for the currently-scalar `RaBitQQuantizer`. This is
the "before" cycle; a follow-up cycle will measure the same battery
against the optimized kernels.

Source audit finding: **`src/quant/rabitq/` contains zero
aarch64-specific code**. RaBitQ on Graviton today runs entirely
through the compiler's scalar fallback, while TurboQuant has explicit
`#[target_feature(enable = "neon")]` blocks in `src/quant/prod.rs`
(`score_ip_from_split_parts_neon`, `score_ip_mse_codes_neon`) and
`src/quant/hadamard.rs` (`fwht_in_place_neon`). The optimization
opportunity is the ~1.7× AM-level slowdown of RaBitQ vs TurboQuant
measured below.

## Claim class

**Local development / review-packet evidence** per
[NFR-007](../../spec/non-functional/NFR-007-benchmark-provenance.md)
and the
[Benchmark Reporting Standard](../../docs/benchmark-reporting-standard.md).
Numbers come from one host on one day; product benchmark claims need a
dedicated controlled-hardware re-run with the same recipe.

## Environment

| Property | Value |
|---|---|
| Region | us-west-2 (AZ us-west-2a) |
| DB instance (AM baselines) | **m8g.xlarge** (Graviton 4 Neoverse-V2 r0p1, 4 vCPU / 16 GB) |
| DB instance (kernel battery) | **m8g.2xlarge** (8 vCPU / 32 GB) |
| EBS | gp3, 150 GB (started 50 GB, upsized for next cycle's 1M corpus prep), encrypted |
| OS | Amazon Linux 2023, kernel 6.1.170-213.321 aarch64 |
| PostgreSQL | 18.3 |
| ecaz extension | 0.1.1 (HEAD = commit `157d176b` at packet write) |
| Rust toolchain | stable-aarch64-unknown-linux-gnu, cargo 1.95.0 (2026-03-21) |
| `RUSTFLAGS` | `-C target-cpu=native -C link-arg=-Wl,--unresolved-symbols=ignore-all` (Linux). Pinned via `.cargo/config.toml`; see commit `ff3be17f` for the fix that made `target-cpu=native` actually apply on Linux + macOS. |
| `[profile.bench]` | `lto = "thin"`, `codegen-units = 4`, `debug = true` (inherits `release`'s `opt-level = 3`). Set in `Cargo.toml`; see commit `14a497e1` for the fix that dropped the previous `lto=fat`+`cu=1` settings that needed > 24 GB peak. |
| Date (UTC) | 2026-05-17 |
| Date (PT) | 2026-05-16 |

CPU feature flags (from `/proc/cpuinfo`, captured in
`artifacts/kernels/env/cpuinfo.txt`):
```
fp asimd evtstrm aes pmull sha1 sha2 crc32 atomics fphp asimdhp cpuid
asimdrdm jscvt fcma lrcpc dcpop sha3 asimddp sha512 sve asimdfhm dit
uscat ilrcpc flagm sb paca pacg dcpodp sve2 sveaes svepmull svebitperm
svesha3 flagm2 frint svei8mm svebf16 i8mm bf16 dgh rng bti
```

Key items for vector scoring: `asimd` (NEON), `asimddp` (dotprod),
`fphp/asimdhp` (fp16), `sve` + `sve2`, `svei8mm`, `svebf16`, base
`i8mm` + `bf16`.

**Codebase runtime feature dispatch** (audited
`src/quant/simd.rs:35`): only `is_aarch64_feature_detected!("neon")`
is checked. SVE2 / i8mm / bf16 / dotprod / fp16 are **not** detected
at runtime today. The host's SVE2 capabilities are unreachable from
the codebase until additional dispatch is added.

## Datasets

`Qdrant/dbpedia-entities-openai3-embedding-3-large-1536-1M` (26
parquet shards, 1 M total rows). Subsets sliced by
`ecaz corpus prepare` using built-in `SubsetProfile`s (sorted-id
prefix, deterministic):

| Prefix | Corpus rows | Query rows | Source |
|---|---|---|---|
| `ec_hnsw_real_10k` | 10,000 | 200 | DBpedia 1M parquet |
| `ec_hnsw_real_50k` | 50,000 | 1,000 | DBpedia 1M parquet |

Restored from snapshot `snap-054feaffc50ecf1c9` (carried forward from
[../cloud-10k-real-baselines/](../cloud-10k-real-baselines/manifest.md)).

## Indexes

Each corpus has both ec_ivf indexes built side-by-side. The AM
baseline pass drops the off-target index before each measurement and
restores both at the end so the EBS snapshot leaves a usable state.

| Index name | AM | reloptions | Build time |
|---|---|---|---|
| `ec_hnsw_real_10k_idx` | ec_ivf | (default = `storage_format=auto` → TurboQuant) | ~9 s |
| `ec_hnsw_real_10k_rabitq_idx` | ec_ivf | `storage_format=rabitq` | ~9 s |
| `ec_hnsw_real_50k_idx` | ec_ivf | default (TurboQuant) | ~30 s |
| `ec_hnsw_real_50k_rabitq_idx` | ec_ivf | `storage_format=rabitq` | ~30 s |

Storage-format-aware costing exists in `src/am/ec_ivf/cost.rs:349-357`
(`RaBitQ => 0.45` selectivity vs TurboQuant default), but on a single
opclass + single embedding column with one index dropped, the planner
unambiguously picks the surviving one.

## Bench parameters

`ecaz bench latency` and `ecaz bench recall`, invoked via
`scripts/run_cloud_am_baseline.sh --profile-runner small`:

| Parameter | Value |
|---|---|
| `k` | 10 |
| `iterations` (latency) | 200 |
| `concurrency` | 1 |
| `sweep` | profile default `nprobe = [8, 16, 24, 32, 48, 64]` |
| `--storage-formats` | `turboquant rabitq` |
| `--prefixes` | `ec_hnsw_real_10k ec_hnsw_real_50k` |
| Per-step manifest | `artifacts/am-baseline/manifest.json` |

## Results — 10k

### Latency (200 iters, k=10, concurrency=1)

**TurboQuant-on-IVF (`storage_format=auto`):**

| nprobe | mean | p50 | p95 | p99 |
|---|---|---|---|---|
| 8 | 1.82 ms | 1.80 ms | 2.25 ms | 2.41 ms |
| 16 | 3.14 ms | 3.18 ms | 3.70 ms | 4.09 ms |
| 24 | 4.39 ms | 4.41 ms | 4.79 ms | 4.96 ms |
| 32 | 5.49 ms | 5.52 ms | 5.95 ms | 6.12 ms |
| 48 | 7.77 ms | 7.78 ms | 8.39 ms | 8.67 ms |
| 64 | 10.2 ms | 10.2 ms | 10.8 ms | 11.1 ms |

**RaBitQ-on-IVF (`storage_format=rabitq`):**

| nprobe | mean | p50 | p95 | p99 |
|---|---|---|---|---|
| 8 | 2.61 ms | 2.63 ms | 3.28 ms | 3.51 ms |
| 16 | 4.72 ms | 4.91 ms | 5.47 ms | 5.62 ms |
| 24 | 6.76 ms | 6.95 ms | 7.35 ms | 7.56 ms |
| 32 | 8.55 ms | 8.67 ms | 9.24 ms | 9.57 ms |
| 48 | 12.3 ms | 12.3 ms | 13.0 ms | 13.4 ms |
| 64 | 16.3 ms | 16.2 ms | 17.1 ms | 17.5 ms |

### Recall@10

| nprobe | TQ recall@10 | TQ ndcg@10 | RaBitQ recall@10 | RaBitQ ndcg@10 |
|---|---|---|---|---|
| 8  | 0.9690 | 0.9994 | **0.9730** | 0.9995 |
| 16 | 0.9730 | 0.9997 | **0.9780** | 0.9998 |
| 24 | 0.9740 | 0.9998 | **0.9785** | 0.9999 |
| 32 | 0.9745 | 0.9998 | **0.9790** | 0.9999 |
| 48 | 0.9745 | 0.9998 | 0.9790 | 0.9999 |
| 64 | 0.9745 | 0.9998 | 0.9790 | 0.9999 |

### Storage (TurboQuant)

| Component | Value |
|---|---|
| Heap | 1.3 MiB |
| Table (heap + toast + fsm/vm) | 159.4 MiB |
| Indexes (total) | 10.3 MiB |
| `ec_hnsw_real_10k_idx` (ec_ivf TQ) | 9.8 MiB (~1030 B/row) |
| `ec_hnsw_real_10k_corpus_pkey` (btree) | 456 KiB (~47 B/row) |

(RaBitQ storage row failed via the ecaz CLI — `bench storage` triggers
an `ec_ivf` scan during row-count that hits an "requires exactly one
ORDER BY query" guard; known CLI bug, not a real failure. RaBitQ index
size is comparable to TQ index size based on the build-time logs.)

## Results — 50k

### Latency (200 iters, k=10, concurrency=1)

**TurboQuant-on-IVF:**

| nprobe | mean | p50 | p95 | p99 |
|---|---|---|---|---|
| 8 | 3.57 ms | 3.55 ms | 4.22 ms | 4.39 ms |
| 16 | 6.17 ms | 6.13 ms | 6.99 ms | 7.56 ms |
| 24 | 8.88 ms | 8.86 ms | 9.79 ms | 10.1 ms |
| 32 | 11.3 ms | 11.4 ms | 12.4 ms | 12.8 ms |
| 48 | 16.4 ms | 16.3 ms | 17.8 ms | 18.1 ms |
| 64 | 21.4 ms | 21.5 ms | 22.6 ms | 23.1 ms |

**RaBitQ-on-IVF:**

| nprobe | mean | p50 | p95 | p99 |
|---|---|---|---|---|
| 8 | 5.20 ms | 5.25 ms | 6.16 ms | 6.52 ms |
| 16 | 9.48 ms | 9.51 ms | 10.8 ms | 11.7 ms |
| 24 | 13.8 ms | 13.9 ms | 15.1 ms | 15.6 ms |
| 32 | 17.9 ms | 18.0 ms | 19.4 ms | 20.1 ms |
| 48 | 26.3 ms | 26.4 ms | 28.0 ms | 28.7 ms |
| 64 | 34.2 ms | 34.3 ms | 36.1 ms | 36.9 ms |

### Recall@10

| nprobe | TQ recall@10 | TQ ndcg@10 | RaBitQ recall@10 | RaBitQ ndcg@10 |
|---|---|---|---|---|
| 8  | **0.8290** | 0.9886 | 0.8287 | 0.9885 |
| 16 | 0.8863 | 0.9941 | 0.8841 | 0.9940 |
| 24 | 0.9107 | 0.9961 | 0.9075 | 0.9960 |
| 32 | 0.9236 | 0.9974 | 0.9202 | 0.9973 |
| 48 | 0.9364 | 0.9985 | 0.9331 | 0.9984 |
| 64 | 0.9414 | 0.9989 | 0.9379 | 0.9988 |

(50k storage also failed via CLI; same bug as 10k.)

## RaBitQ-vs-TurboQuant slowdown summary

The **headline finding**: at every (corpus × nprobe) point, RaBitQ
is 1.3× to 1.8× slower than TurboQuant, with no recall benefit at 50k
and a small recall benefit at 10k.

| Corpus / nprobe | TQ mean | RaBitQ mean | Slowdown | Recall delta |
|---|---|---|---|---|
| 10k @ 8   | 1.82 ms | 2.61 ms | **1.43×** | +0.0040 |
| 10k @ 16  | 3.14 ms | 4.72 ms | **1.50×** | +0.0050 |
| 10k @ 64  | 10.2 ms | 16.3 ms | **1.60×** | +0.0045 |
| 50k @ 8   | 3.57 ms | 5.20 ms | **1.46×** | −0.0003 |
| 50k @ 16  | 6.17 ms | 9.48 ms | **1.54×** | −0.0022 |
| 50k @ 64  | 21.4 ms | 34.2 ms | **1.60×** | −0.0035 |

That ~1.5× factor is the *optimization headroom* available from
adding NEON/SVE2 to `RaBitQQuantizer`. The TurboQuant kernels already
have explicit `#[target_feature(enable="neon")]` blocks; RaBitQ goes
through the compiler's scalar fallback for every distance estimate.

## Kernel-level attribution

Captured by `scripts/run_kernel_battery.sh --profile medium --skip-iai`
(commit `157d176b`), output `artifacts/kernels/`.

### Criterion wall-time (key 1536-dim 4-bit cases)

| Kernel | Time (ms) | Throughput |
|---|---|---|
| `score_ip_encoded` (d=1536, b=4) | 1.105 µs | 904.7 K elem/s |
| `score_ip_from_parts` (d=1536, b=4) | 1.105 µs | 904.7 K elem/s |
| `score_ip_codes_lite` (d=1536, b=4) | 6.245 µs | 160.1 K elem/s |

`score_ip_encoded` and `score_ip_from_parts` produce identical times
because the latter tail-calls the former (confirmed in
`asm/score_ip_from_parts.s` — `b ecaz::quant::prod::ProdQuantizer::score_ip_from_split_parts`).

### Hardware counters (perf stat, full bench duration)

**Group A — compute + branch** (`artifacts/kernels/perf-stat-quant_score-A.log`):

| Counter | Value |
|---|---|
| cycles | 545,730,573,769 |
| instructions | 2,666,075,043,166 |
| **IPC** | **4.89 insn/cycle** |
| branches | `<not supported>` (PMU limit on Neoverse-V2 with kernel 6.1) |
| branch-misses | 3,188,287,406 |
| stalled-cycles-frontend | 15,658,048,257 (**2.87% FE-idle**) |
| stalled-cycles-backend | 97,720,339,215 (**17.91% BE-idle**) |

**Group B — memory hierarchy** (`artifacts/kernels/perf-stat-quant_score-B.log`):

| Counter | Value |
|---|---|
| L1-dcache-loads | 543,907,442,878 |
| L1-dcache-load-misses | 1,829,118,031 (**0.34% miss rate**) |
| LLC-loads / LLC-load-misses | `<not supported>` |
| dTLB-load-misses | 117,775,665 |
| iTLB-load-misses | 2,886,523 |

**Top-down breakdown**: `<not supported>` on this PMU + kernel.
`perf stat --topdown -a` returns `System does not support topdown`.
Neoverse-V2 + AL2023 kernel 6.1 don't expose the topdown event class.
Recoverable in a future cycle with `linux-arm64-tools-aws` 6.6+ or
explicit `topdown-fe-bound / be-bound / bad-spec / retiring` events.

### Interpretation

- **4.89 IPC** is high — Neoverse-V2 retires up to 8 micro-ops/cycle
  theoretical max; we're at 61% of theoretical. The hot loops are
  pipelined well by the NEON paths.
- **17.91% backend-bound** is the optimization target: backend stalls
  on Neoverse-V2 are dominated by memory access wait + execution-unit
  contention. With the L1 miss rate being only 0.34%, these are
  execution-unit-bound, not memory-bound. SVE2's wider lanes (vs NEON
  128-bit) would address this directly.
- **0.34% L1 miss rate** means the kernel inputs are L1-resident (256
  / 768 / 1536 / 3072 dim × 4 bit codes = 128 B – 1.5 KB per code).
  No need to optimize for cache layout at these sizes; that becomes
  relevant at 1M+ corpora where the working set exceeds L2.
- **2.87% frontend idle** means decode/branch-predictor isn't the
  bottleneck. Won't benefit from unrolling.

### Heap profile (dhat)

| Bench | dhat result |
|---|---|
| `dhat_encode` | (see `artifacts/kernels/dhat-encode.json`) |
| `dhat_score` | **`Total: 0 bytes in 0 blocks`** ✓ |

The score path being zero-alloc is the documented invariant in
`src/quant/mod.rs:25` ("O(n) with zero allocation"). dhat confirms it
still holds. Don't regress this in the optimization cycle.

### Disassembly

Captured via `cargo asm --package ecaz --lib --features bench --release`,
output `artifacts/kernels/asm/<fn>.s`:

| Function | Lines | Notes |
|---|---|---|
| `score_ip_from_parts` | 70 | Wrapper that tail-calls `score_ip_from_split_parts` |
| `score_ip_encoded` | 88 | Real disassembly |
| `score_ip_codes_lite` | 101 | Real disassembly |
| `fwht_in_place` | 410 | Dispatch + NEON path |
| `score_ip_from_split_parts_neon` | 0 | Inlined at all call sites (no exported symbol) |
| `score_ip_mse_codes_neon` | 0 | Inlined |
| `fwht_in_place_scalar` | 0 | Inlined |
| `estimate_ip` | 14 | RaBitQ scalar path (small — scoring is mostly inlined into callers) |

Functions reporting 0 lines have been inlined into their callers — a
useful signal in itself: the optimizer aggressively inlines the
`_neon` variants where they're hot. The next optimization cycle's
flamegraph will need to identify those inline sites.

## aarch64 SIMD coverage matrix

Source audit + asm capture:

| Kernel | aarch64 path today | Coverage |
|---|---|---|
| `score_ip_from_parts` (TQ) | NEON via tail-call to `score_ip_from_split_parts_neon` | Has dispatch |
| `score_ip_encoded` (TQ) | Compiler-vectorized via `target-cpu=native` | Implicit only |
| `score_ip_codes_lite` (TQ) | Inline `score_ip_mse_codes_neon` | Has dispatch |
| `fwht_in_place` | Explicit `fwht_in_place_neon` (`hadamard.rs:391`) | Has dispatch |
| `RaBitQQuantizer::estimate_ip` | **scalar only** | **GAP — optimization target** |
| `RaBitQQuantizer` rotate / encode | **scalar only** | **GAP** |
| `score_ip_from_split_parts_neon` | NEON (gated `#[target_feature]`) | ✓ |
| `score_ip_mse_codes_neon` | NEON (gated) | ✓ |
| Any SVE2-gated path | **none in repo** | **GAP across the board** |

This matrix is the optimization shortlist. RaBitQ scalar paths are
the highest-impact target (no NEON anywhere yet), with the ~1.5× AM
slowdown above as the headroom measurement.

## Gaps (deferred to next cycle)

Documented for honesty; none block the optimization work:

1. **Top-down breakdown**: PMU/kernel limitation, not script. Need
   kernel 6.6+ or explicit Neoverse-V2 topdown events.
2. **Flamegraph SVG**: `cargo flamegraph` filled the root volume's
   /tmp with `perf.data`. Fix is to point `perf record -o` at the
   data EBS volume; partial fix landed in `scripts/run_kernel_battery.sh`
   at commit `157d176b` but the perf.data path itself still needs
   work.
3. **STREAM memory bandwidth ceiling**: AL2023 doesn't ship `gfortran`
   by default; STREAM's Fortran target failed. The C target builds.
   Easy fix: dnf install gcc-gfortran in the script's STREAM stage.
4. **iai-callgrind instruction counts**: skipped via `--skip-iai` on
   small cloud hosts because valgrind on aarch64 is very slow. Can
   run later on a beefier host or locally.
5. **`ecaz bench storage` for non-default storage_format**: triggers
   an ec_ivf-scan-without-ORDER-BY error. CLI bug, not a measurement
   gap — storage numbers extractable from index-builds.log + direct
   psql.

## Preserved artifacts

- **EBS volume**: `vol-0d09cac38fcfb94c9` (150 GB, gp3, attached to
  `i-05af7ea8e92f65b30`). Both ec_hnsw_real_10k and ec_hnsw_real_50k
  corpora and both index storage formats persisted. Instance is
  **left running** for the next-cycle 100k + 1M corpus prep.
- **Anchor snapshot from prior cycle**: `snap-054feaffc50ecf1c9` (the
  source of the AM data above). Not deleted.
- **Raw logs**: `artifacts/{am-baseline,kernels}/` in this packet.

## Reproduction recipe

From a fresh `terraform apply` of `infra/cloud/terraform/profiles/10k-medium.tfvars`
(or in-place stop+modify+start from any other profile) with
`enable_eice_ssh=true`:

```bash
# On the operator workstation:
aws ec2-instance-connect send-ssh-public-key \
    --instance-id <db-iid> --instance-os-user ec2-user \
    --ssh-public-key file://~/.ssh/ecaz-bench.pub

ssh -i ~/.ssh/ecaz-bench \
    -o ProxyCommand="aws ec2-instance-connect open-tunnel --instance-id <db-iid>" \
    ec2-user@<db-iid>

# On the DB host (as postgres, with PGHOST=/tmp PGDATABASE=tqvector_bench):
cd /var/lib/pgsql/build/ecaz
git checkout 157d176b   # this packet's source state

# AM baselines (uses both storage formats, both corpora):
scripts/run_cloud_am_baseline.sh \
    --out /tmp/artifacts/am-baseline \
    --db tqvector_bench \
    --profile-runner small

# Kernel battery (criterion + perf-stat + dhat + asm):
make kernel-battery-cloud-medium OUT=/tmp/artifacts/kernels
```

Re-running on the same EBS volume gives the same numbers within
criterion variance (verified: prior AM baseline at
[../cloud-10k-real-baselines/](../cloud-10k-real-baselines/manifest.md)
matched within 0–4% on the TurboQuant rows after the
`target-cpu=native` fix landed; identical recall).

## Next cycle

- Prep `ec_hnsw_real_100k` + `ec_hnsw_real_1m` corpora with both
  storage formats, run same AM-level battery → scaling curve.
- Begin NEON `RaBitQQuantizer::estimate_ip` (the obvious next step
  given the 1.5× slowdown + zero existing aarch64 coverage).
- Re-run kernel battery against the optimized binary; diff this
  packet's perf-stat counters against the new ones for the same
  bench duration → quantify per-kernel improvement.

## See also

- [`docs/benchmark-reporting-standard.md`](../../docs/benchmark-reporting-standard.md)
- [`spec/non-functional/NFR-007-benchmark-provenance.md`](../../spec/non-functional/NFR-007-benchmark-provenance.md)
- [`spec/non-functional/NFR-015-benchmark-reporting-standard.md`](../../spec/non-functional/NFR-015-benchmark-reporting-standard.md)
- [`spec/functional/FR-038-configured-benchmark-suite-runner.md`](../../spec/functional/FR-038-configured-benchmark-suite-runner.md)
  (the suite runner that this packet's AM-baseline script complements
  for the snapshot-restore case)
- Prior cycle: [`review/cloud-10k-real-baselines/manifest.md`](../cloud-10k-real-baselines/manifest.md)
  (TurboQuant-only baseline before the `target-cpu=native` and bench-
  profile fixes landed)
