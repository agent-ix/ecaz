# Review Request: IVF Scratch SoA Batch Decode

## Scope

Code commit under review:

- `a22ca84531379581855613a2968a2ca8aca14a5b` - opt-in IVF scratch SoA batch decode

Benchmark packet:

- `benchmarks/task51-local-ivf-scratch-soa/`

This slice adds an opt-in local scratch-SoA path for IVF/RaBitQ bits=1 scans.
The default scan path is unchanged unless `ec_ivf.scratch_soa_batch_decode` is
enabled. The CLI and suite runner can enable the GUC via
`--ivf-scratch-soa-batch-decode` / `ivf_scratch_soa_batch_decode`.

Files changed by the code commit:

- `src/am/ec_ivf/options.rs`
- `src/am/ec_ivf/scan.rs`
- `crates/ecaz-cli/src/commands/bench/mod.rs`
- `crates/ecaz-cli/src/commands/bench/recall.rs`
- `crates/ecaz-cli/src/commands/bench/latency.rs`
- `crates/ecaz-cli/src/commands/bench/suite.rs`

## Result

Local validation:

```text
cargo check --no-default-features --features pg18
cargo test -p ecaz-cli expands_recall_with_defaults
cargo test posting_scratch_soa --no-default-features --features pg18 --no-run
git diff --check
```

All completed successfully. The `--no-run` coverage builds the new unit tests;
running the library test binary directly in this local pgrx-linked session is
blocked by the existing `pg_re_throw` dynamic-symbol issue, so the packet uses
compile validation plus the PG18 benchmark suite as runtime evidence.

Benchmark status:

```text
[suite:task51-local-ivf-scratch-soa] completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Main benchmark finding:

- Recall parity held: both static and scratch SoA report recall@10 `0.9750`,
  recall p10 `0.9000`, and NDCG@10 `0.9986`.
- Latency p50 improved `603.7 ms -> 590.5 ms`, about 2.2%.
- EXPLAIN execution improved `586.336 ms -> 570.902 ms`, about 2.6%, with
  identical posting/candidate counts.

## Decision

Do not promote this prototype as a Task 51 win. It is useful infrastructure and
an honest experiment, but the current implementation does not meet Exp 3's
local gate of at least 20% candidates/sec improvement. It should remain opt-in.

This packet also rejects using this scratch-SoA result as a trigger for Posting
Layout v2 in this round. The Task 51 Layout v2 gate requires a clear scratch
SoA or counter signal that posting decode/scan is the primary bottleneck; this
run shows only a small local improvement.

## Notes For Review

- The code path is gated by session GUC and by storage shape:
  `StorageFormat::RaBitQ && quant_bits == 1`.
- The scratch buffer is scan-owned and reused; no per-candidate heap allocation
  is introduced on the hot path.
- The current prototype copies tuple fields into SoA buffers before using the
  existing scalar scoring path. It is not a true chunked/vectorized bits=1
  scoring kernel.
- No AWS, vchord, or pgvectorscale runs were performed.

See `artifacts/manifest.md` for packet-local validation artifacts.
