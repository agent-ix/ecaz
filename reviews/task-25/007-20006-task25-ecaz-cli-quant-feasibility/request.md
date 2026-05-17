# Review Request: Task 25 Slice 7 — Move Feasibility Harness into `ecaz quant feasibility`

Scope:
- `crates/ecaz-cli/Cargo.toml` — adds `ecaz = { path = "../.." }` so
  the CLI can reach `ecaz::bench_api::{Quantizer, RaBitQQuantizer,
  ProdQuantizer, payload_len}`. The `ecaz` crate already declared
  `crate-type = ["cdylib", "lib"]`, so no library-crate change was
  needed.
- `crates/ecaz-cli/src/commands/quant/mod.rs` — new `QuantCommand`
  subcommand group. Peer to `bench`, `corpus`, `compare`, `dev`,
  `stress`. Intentionally separate from `bench` because feasibility
  runs offline against TSV files, not against a loaded-corpus
  Postgres table.
- `crates/ecaz-cli/src/commands/quant/feasibility.rs` — the
  harness itself. `QuantizerKind` is a `ValueEnum` with one variant
  (`Rabitq`) today; the match in `run` is the single
  extension point for future quantizers. Loads via
  `crate::tsv::iter_rows` so it consumes the canonical ecaz-cli
  `<id>\t<json_array>` shape — same format `ecaz corpus generate`
  writes and `ecaz corpus load` reads.
- `crates/ecaz-cli/src/cli.rs` + `crates/ecaz-cli/src/commands/mod.rs`
  — register the new subcommand.
- `src/bin/rabitq_feasibility.rs` — **deleted**. Per the "deprecate
  = delete" project rule; the harness now lives in ecaz-cli
  canonically, the library-level binary was a scaffolding stop.

Task: `plan/tasks/25-rabitq-quantizer.md` (slice 7, added during
Phase 2 work). Unblocks slice 8 (real gate-verdict run).

Branch: `task25-rabitq-stage1-phase0` (slice 7 builds on `b3b40fc`).

## Problem

Slice 5 shipped `src/bin/rabitq_feasibility.rs`. The problem the
user flagged: every subsequent quantizer on the roadmap — OPQ
(task 20), additive residual (task 22), LSQ (task 23), Symphony
Stage 2 (task 27) — will want the same recall-vs-exact + error-bound
study before promotion. A one-off `src/bin/` target would force each
new quantizer to clone the harness.

## Approach

### QuantizerKind is a one-arm enum today

Only `Rabitq` is wired up. The shape exists so that when task 20 or
task 22 lands, the cost of adding a feasibility entry point is:

1. One `QuantizerKind` variant.
2. One `run_<name>` function that wraps the quantizer's construction
   and its `estimate_ip` equivalent (or `score` if it has no bound).
3. One match arm in `run`.

The recall / bound / tightness summary and the PASS / MARGINAL / FAIL
verdict printer are shared across variants.

### Offline-only

The command does not take a `--database` connection path — it
reads the TSV fixtures directly via `tsv::iter_rows` and computes
brute-force exact top-K per query in-process. `--database` still
exists as a global flag (inherited from `Cli`) but is unused by
this command; that is consistent with `ecaz corpus generate`, which
also ignores it.

### TSV shape

The ecaz-cli canonical format is `<id>\t<json_array>`. That is what
`ecaz corpus generate`, `ecaz corpus prepare`, and the
`tsv::iter_rows` parser all speak. Slice 5's standalone binary had
to carry its own dual-format parser (JSON-in-brackets + whitespace)
to bridge the difference; slice 7 drops that complexity because we
only speak the canonical format here.

### Parity with deleted binary

Same `--dim`, `--top-k`, `--seed` defaults. New optional flags
`--corpus-limit` / `--query-limit` let operators cap rows cheaply
during iteration (10k × 500 at D=1536 runs in ~47 s; capping to
10k × 200 — what the gate-ready smoke run uses — is ~19 s).

## Verification

- `cargo build --release -p ecaz-cli` clean.
- `ecaz quant feasibility --help` lists the new flags.
- Battle-test run on the 10k synthetic corpus that was generated
  earlier in this session via `ecaz corpus generate`:

  ```
  $ time ./target/release/ecaz quant feasibility \
      --quantizer rabitq \
      --corpus-file data/rabitq-feasibility/corpus.tsv \
      --queries-file data/rabitq-feasibility/queries.tsv \
      --dim 1536 --top-k 10 --query-limit 200

  # ecaz quant feasibility
  # quantizer: Rabitq  corpus_file: … queries_file: …
  # loaded: 10000 corpus vectors, 200 queries (dim=1536, top_k=10, seed=42)
  # storage: RaBitQ code 200 B, PQ4 code 768 B (parity ratio 3.84x)
  #   query 0: recall@10 = 2/10
  #   query 1: recall@10 = 2/10
  #   query 2: recall@10 = 4/10

  recall@10 mean: 0.2535
  bound  mean=0.602  p50=0.602  p99=0.619
  error  mean=0.012  p50=0.010  p99=0.040
  tightness (error / bound) mean: 0.020

  GATE: FAIL (recall gap 74.650 pp > 2.0 pp)

  real    0m19.027s
  ```

  The FAIL verdict is **not a gate signal**. The corpus is
  `ecaz corpus generate`'s unit-sphere iid Gaussian — adversarial
  for any approximate IP method at D=1536 (true top-K IP values
  concentrate at ~0.08-0.10, and the estimator's p99 error is 0.04,
  so noise is at the same scale as signal). The result proves:
  - the end-to-end pipeline works (load → encode → score → summary);
  - the bound calibration is healthy (`tightness = 0.020` — bound
    is an envelope 50× looser than realized mean error, as
    Cauchy-Schwarz should be on random inputs);
  - the harness matches the standalone binary's numbers to within
    sampling noise (the binary at 500 queries got 0.2488 vs.
    0.2535 here at 200 queries; seed-level reproducibility
    preserved).

  The real gate-verdict run (slice 8) still needs a structured
  corpus (DBpedia-via-`corpus prepare`, or any real embedding
  TSV). On that corpus the expectation per the ADR-045 Stage 1
  design is recall@10 ≥ 0.99 at the same storage ratio.

## What this slice does NOT do

- No additional quantizer variants. OPQ / AQ / LSQ land through
  their own tasks; this slice provides the seam.
- No summary JSON output. `ecaz bench` commands already have a
  JSON-summary convention via `comfy-table` + `serde_json`; if
  reviewers want `ecaz quant feasibility --json out.json` for
  CI diff-gating, happy to add it in slice 8 once the output
  shape is confirmed against the real-corpus run.
- No verdict threshold flags. The 1 pp / 2 pp boundaries come from
  the ADR-045 Stage 1 gate directly. Overriding them per-invocation
  would invite moving the gate post.

## Open questions for reviewer

1. Placement under `ecaz quant` rather than `ecaz bench quant` or
   `ecaz compare quant`. I chose a peer group because (a) `bench`
   wants a loaded DB corpus and (b) `compare` is cross-engine.
   Moving it under `bench offline-quant` or similar is fine if you
   prefer fewer top-level groups.
2. The `--quantizer` default is `rabitq`. Once there is a second
   variant, we should probably require the flag explicitly rather
   than silently defaulting — but that is a cross-cutting decision
   (same question applies to `--engine` in `compare`). Keeping the
   default for now to match task-25's scope.
3. `FeasibilityArgs::seed` currently seeds only `ProdQuantizer`'s
   codebook / SRHT signs (the argument to `RaBitQQuantizer::with_srht`).
   It does **not** seed any RNG in the harness itself (the brute-force
   loop is deterministic). Flagging so reviewers don't expect
   `--seed` to shift query order or anything else.
