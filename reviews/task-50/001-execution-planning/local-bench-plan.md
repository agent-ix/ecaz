# Task 50 Local Bench Plan

Companion to `bench-baseline-plan.md`. Local benches are the
fast-iteration gate for Task 50 slices; AWS benches are closeout
confirmation. SIMD slices also require a local x86_64 measurement
because the AWS Graviton lane exercises the NEON path, not AVX2/FMA.

## Current local bench inventory

### Criterion microbenches (`cargo bench --features bench`)

Defined under `benches/criterion/`:

| Lane            | Target file                          | What it covers |
|-----------------|--------------------------------------|----------------|
| `quant_score`   | `benches/criterion/quant_score.rs`   | TurboQuant / RaBitQ kernel scoring |
| `quant_encode`  | `benches/criterion/quant_encode.rs`  | Quantizer encode path |
| `quant_prepare` | `benches/criterion/quant_prepare.rs` | Query preparation |
| `hadamard`      | `benches/criterion/hadamard.rs`      | FWHT SIMD dispatch |
| `codebook`      | `benches/criterion/codebook.rs`      | Codebook ops |
| `bitpack`       | `benches/criterion/bitpack.rs`       | 3-bit / 4-bit packing |
| `page_codec`    | `benches/criterion/page_codec.rs`    | Page encode/decode |
| `text_io`       | `benches/criterion/text_io.rs`       | Text I/O paths |

These are **kernel-level** measurements. Useful for catching
codegen regressions on the SIMD and encoding paths — exactly what
Slices 2 (page visitor), 7 (vector datum), and 8 (SIMD newtypes)
need a local pre-commit signal for.

### IAI instruction-count benches (`make bench-iai`)

Defined under `benches/iai/`:

- `iai_quant_score`
- `iai_hadamard`
- `iai_bitpack`

Instruction-count instead of wall-clock — more reproducible across
noisy local hosts. **The right tool for a local smoke gate** when
the change is structural (insert/remove a function boundary,
change inlining shape, add/remove a deref). A 0% instruction-count
delta is strong evidence that no codegen regression happened.

### dhat heap profiling

Defined under `benches/dhat/{encode,score}.rs`. Catches accidental
allocation introduced by a structural change — particularly
relevant for Slice 4 (heap source scorer), where the failure mode
listed in my reviewer feedback risk register is "accidental
per-candidate allocation."

### ecaz-cli AM-level lanes

`crates/ecaz-cli/src/commands/bench/` exposes:

- `build_probe`, `cross_am`, `graph`, `latency`, `overhead`,
  `recall`, `spire_pipeline`, `storage`, `suite`.

`make recall-gate RECALL_GATE_CONFIG=fixtures/gates/recall-gate-small.json`
runs the small recall suite against a local PostgreSQL.

### Functional recall integration

`make recall` runs `cargo test --features bench --release
--test recall_integration -- --ignored --nocapture`. This is
**correctness, not performance** — useful as a "did I break
recall" check, not as a regression gate.

### What's broken (Makefile rot — flag for the coder)

The following Make targets exist but reference scripts that
**don't exist in the tree:**

- `make bench-sql-latency` → `scripts/bench_sql_latency.sh`
  (missing)
- `make bench-storage` → `scripts/bench_storage.sh` (missing)

Flag for a separate cleanup packet: either restore the scripts or
remove the dead targets. Don't bundle into Task 50.

Also: `rg` (ripgrep) is not installed by default on this dev host
— required by `block-count-tooling.md`'s `make unsafe-block-count`
target. Same flag as point 8a of my reviewer feedback.

## Local smoke-gate policy by slice

| Slice                              | Local smoke lanes (pre-commit)            | Required? |
|------------------------------------|-------------------------------------------|-----------|
| 1 callback wrapper (1a/1b/1c)      | `cargo check --features pg18,bench` only  | optional  |
| 2 IVF page tuple visitor           | `bench-iai iai_quant_score` + `criterion page_codec`  | recommended |
| 3 ActiveEpochAnchor                | `cargo check` only                        | optional  |
| 4 heap source scorer               | `criterion quant_score` + `dhat score`    | required  |
| 5 reloptions wrapper               | `cargo check` only                        | optional  |
| 6 WAL + exclusive buffer pair      | `criterion page_codec`                    | recommended |
| 7 vector datum detoast wrapper     | `criterion quant_encode` + `dhat encode`  | required  |
| 8 SIMD load/store newtypes         | **see SIMD section below**                | required  |
| 9 DSM atomic field wrapper         | (deferred — HNSW priority is low)         | n/a       |

Rationale:

- **Slices 1, 3, 5 are doc-shape changes** — the failure modes
  are compile-error or correctness-error, not performance. `cargo
  check` catches the former; integration tests catch the latter.
  No bench needed locally.
- **Slices 2, 6 touch page hot paths** — `iai_quant_score` and
  `criterion page_codec` catch codegen regressions in the scoring
  / encoding path that downstream slices depend on.
- **Slices 4, 7 touch heap and detoast paths** — adding `dhat`
  heap profiling catches the "I accidentally added a `.to_vec()`"
  failure mode in <30 seconds.
- **Slice 8 is the SIMD slice** — cross-arch requirement, see
  below.

## SIMD slices: required local AVX2/FMA measurement

Slice 8 (and any SIMD-touching part of Slice 7) **requires a
local x86_64 measurement** in addition to the cloud Graviton
NEON measurement. Reasons:

1. The cloud baseline exercises only the NEON code path (Graviton
   4 is aarch64).
2. AVX2/FMA code in `src/quant/hadamard.rs` and `src/quant/prod.rs`
   has explicit `#[target_feature(enable = "avx2,fma")]` blocks
   that the cloud bench never runs.
3. `src/quant/hadamard.rs` has 62 unsafe blocks (rank 14 in the
   Task 50 top-15) — its restructuring affects the x86 path.

Local AVX2 capture for SIMD slices:

```sh
cargo bench --features bench --bench iai_hadamard
cargo bench --features bench --bench iai_quant_score
cargo bench --features bench --bench hadamard
cargo bench --features bench --bench quant_score
```

Store under packet artifacts:

- `artifacts/iai-hadamard-x86-before.log` /
  `iai-hadamard-x86-after.log`
- `artifacts/iai-quant-score-x86-before.log` /
  `iai-quant-score-x86-after.log`
- `artifacts/criterion-hadamard-x86-before/` /
  `criterion-hadamard-x86-after/` (criterion writes a directory)

Document the host CPU in `artifacts/manifest.md`: model name,
target-feature set (AVX2/FMA/AVX512), kernel, RAM. The AVX2/FMA
code path is sensitive to which feature set the CPU actually
exposes, so the manifest must record it for the measurement to be
interpretable.

## Pre-commit smoke recipe

For any Task 50 packet, the recommended pre-commit sequence:

```sh
# 1. Block count delta (always)
make unsafe-block-count > /tmp/after.log
diff /tmp/before.log /tmp/after.log

# 2. Compile + lint
cargo fmt --all
cargo check --all-targets --no-default-features --features pg18,bench
cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings

# 3. Required smoke for the slice type (per table above)
#    Example for Slice 4 heap scorer:
cargo bench --features bench --bench quant_score
cargo bench --features bench --bench dhat_score   # if added as a target

# 4. Functional recall sanity check (every slice that touches
#    scan/build/insert):
make recall
```

This sequence runs in <5 minutes on most dev hosts and catches
the failures most likely to slip through a `cargo check` —
inlining inhibition, accidental allocation, recall regression
from a wrong invariant.

## What local benches are for

- **Fast iteration and packet-level evidence.** Same-host local
  before/after numbers are the expected per-slice signal for hot
  scoring, traversal, page, heap, vector-datum, and SIMD changes.
- **Same-host before/after comparison.** Different hardware,
  different SIMD path, and different page-cache behavior make
  local-after vs. cloud-before comparisons invalid.
- **Not final closeout by itself.** Local numbers do not replace
  AWS closeout confirmation for hot paths, especially SPIRE
  distributed-read behavior.

## Coordination with cloud lane

When a slice has both required local and cloud measurements:

1. Capture local before+after first (fast iteration).
2. Push the packet branch.
3. Capture cloud lane on the m8g.2xlarge-class Graviton host
   (per the existing AWS bench infrastructure under
   `infra/cloud/terraform/` and `ecaz cloud ...`). Smaller
   m8g.large lanes are historical context and did not complete the
   full benchmark set reliably.
4. Both before/after pairs land in the packet's `artifacts/`
   directory.
5. The packet's `request.md` cites local comparison as the
   iteration gate and AWS comparison as closeout confirmation.

This keeps the audit trail clear: local measurements guide slice
iteration, and AWS measurements confirm closeout on the scaled
hardware.

## Baselines required before code lands

The local pre-Task-50 baseline capture should run once on the dev
host before Slice 1a lands. This packet is responsible for
generating and loading any missing local corpus profiles, not just
running microbenches against whatever happens to exist already.

## Full local corpus spread

Available subset profiles in `crates/ecaz-cli/src/commands/corpus/prepare.rs`:

| Profile | Corpus rows | Query rows |
| --- | ---: | ---: |
| `ec_real_10k` | 10,000 | 200 |
| `ec_real_25k` | 25,000 | 500 |
| `ec_real_50k` | 50,000 | 1,000 |
| `ec_real_100k` | 100,000 | 1,000 |
| `ec_real_ann_benchmarks_anchor` | 990,000 | 10,000 |

Baseline generation rule:

- check whether each profile's prepared TSVs and manifest exist locally;
- run `ecaz corpus prepare` for missing profiles from the canonical parquet
  source;
- load each prepared profile for the Task 50 AM surfaces needed locally:
  IVF/RaBitQ, SPIRE, HNSW, and DiskANN where supported;
- run recall, latency, and storage rows for each loaded AM/profile pair;
- record missing or intentionally deferred rows in `manifest.md` with a reason.

Minimum AM/storage matrix:

| Surface label | Load args | Notes |
| --- | --- | --- |
| `ec_ivf_rabitq` | `--profile ec_ivf --storage-format rabitq` | Priority IVF/RaBitQ lane. |
| `ec_spire_rabitq` | `--profile ec_spire --storage-format rabitq` | Priority SPIRE lane where local SPIRE load/search is ready. |
| `ec_hnsw` | `--profile ec_hnsw` | Required for top-15 unsafe-density follow-through. |
| `ec_diskann` | `--profile ec_diskann` | Required for top-15 unsafe-density follow-through. |

Add IVF `turboquant` / `pq_fastscan` rows when a slice touches shared
storage-format code, but do not treat them as replacements for the RaBitQ row.

Suggested profile loop skeleton:

```sh
for profile in \
  ec_real_10k \
  ec_real_25k \
  ec_real_50k \
  ec_real_100k \
  ec_real_ann_benchmarks_anchor
do
  ecaz corpus prepare \
    --profile "$profile" \
    --parquet "$DBPEDIA_PARQUET" \
    --output-dir "$TASK50_STAGED" \
    2>&1 | tee "benchmarks/task-50-local-baseline/artifacts/corpus-prepare-$profile.log"
done
```

Suggested AM/profile baseline skeleton:

```sh
for profile in ec_real_10k ec_real_25k ec_real_50k ec_real_100k ec_real_ann_benchmarks_anchor
do
  for surface in \
    "ec_ivf_rabitq:ec_ivf:rabitq" \
    "ec_spire_rabitq:ec_spire:rabitq" \
    "ec_hnsw:ec_hnsw:" \
    "ec_diskann:ec_diskann:"
  do
    label="${surface%%:*}"
    rest="${surface#*:}"
    am="${rest%%:*}"
    storage_format="${rest#*:}"

    storage_args=()
    if [ -n "$storage_format" ]; then
      storage_args=(--storage-format "$storage_format")
    fi

    ecaz corpus load \
      --prefix "$profile" \
      --profile "$am" \
      "${storage_args[@]}" \
      --corpus-file "$TASK50_STAGED/${profile}_corpus.tsv" \
      --queries-file "$TASK50_STAGED/${profile}_queries.tsv" \
      --manifest-file "$TASK50_STAGED/${profile}_manifest.json" \
      2>&1 | tee "benchmarks/task-50-local-baseline/artifacts/corpus-load-$profile-$label.log"

    ecaz bench recall --prefix "$profile" --profile "$am" \
      2>&1 | tee "benchmarks/task-50-local-baseline/artifacts/recall-$profile-$label.log"
    ecaz bench latency --prefix "$profile" --profile "$am" \
      2>&1 | tee "benchmarks/task-50-local-baseline/artifacts/latency-$profile-$label.log"
  done

  ecaz bench storage --prefix "$profile" \
    2>&1 | tee "benchmarks/task-50-local-baseline/artifacts/storage-$profile.log"
done
```

The loop is a plan, not a blind command: if a profile/AM combination is not
supported or would overwrite an existing index, use an isolated prefix and
record the exact command in the manifest. `ecaz bench storage` is prefix-based,
so it should run once per loaded corpus prefix and report every index on that
corpus table. The 990k anchor can be time-consuming; do not drop it silently.
Either run it as the local closeout-scale row or mark it as deferred with the
operational blocker.

## Kernel and microbench baseline

```sh
mkdir -p benchmarks/task-50-local-baseline/artifacts
cargo bench --features bench --bench iai_quant_score 2>&1 | tee benchmarks/task-50-local-baseline/artifacts/iai-quant-score.log
cargo bench --features bench --bench iai_hadamard    2>&1 | tee benchmarks/task-50-local-baseline/artifacts/iai-hadamard.log
cargo bench --features bench --bench iai_bitpack     2>&1 | tee benchmarks/task-50-local-baseline/artifacts/iai-bitpack.log
cargo bench --features bench --bench quant_score     2>&1 | tee benchmarks/task-50-local-baseline/artifacts/criterion-quant-score.log
cargo bench --features bench --bench hadamard        2>&1 | tee benchmarks/task-50-local-baseline/artifacts/criterion-hadamard.log
cargo bench --features bench --bench page_codec      2>&1 | tee benchmarks/task-50-local-baseline/artifacts/criterion-page-codec.log
```

Plus a `manifest.md` recording HEAD SHA, CPU model + target
features, OS, PG version, criterion sample size used. This
captures the "before" state for local smoke comparisons. ~10
minutes to run once.
