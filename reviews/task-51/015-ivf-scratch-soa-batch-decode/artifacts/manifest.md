# Task 51 Review Packet 015: IVF Scratch SoA Batch Decode

- head SHA: `a22ca84531379581855613a2968a2ca8aca14a5b`
- timestamp: `2026-05-23T15:06:04Z`
- task bucket: `reviews/task-51/`
- packet path: `reviews/task-51/015-ivf-scratch-soa-batch-decode/`
- code commit: `a22ca84531379581855613a2968a2ca8aca14a5b`
- benchmark packet: `benchmarks/task51-local-ivf-scratch-soa/`
- lane: local PG18 / WSL2 only
- AWS: not used
- competitors: none; this packet is IVF/RaBitQ only
- table surface: reused preserved isolated prefix `task51_local_990k_ivf_rabitq1_n1024_w50`
- storage format: `rabitq`
- rerank mode: heap f32 rerank width 50
- isolated one-index-per-table surface: yes, inherited from packet `benchmarks/task51-local-ivf-rabitq-990k/`

## Code Scope

Commit `a22ca84531379581855613a2968a2ca8aca14a5b` adds:

- opt-in `ec_ivf.scratch_soa_batch_decode`, default `off`;
- scan-owned scratch buffers for contiguous heap TIDs, gammas, and RaBitQ
  payload bytes;
- GUC-gated scratch-SoA scoring for `StorageFormat::RaBitQ && quant_bits == 1`;
- benchmark CLI and suite support via `--ivf-scratch-soa-batch-decode` and
  `ivf_scratch_soa_batch_decode`.

## Validation Artifacts

### `cargo-check-pg18.log`

Command:

```text
cargo check --no-default-features --features pg18
```

Result:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.50s
```

Existing warnings are present in `src/am/mod.rs` and `src/am/ec_ivf/build.rs`;
they are not introduced by this packet.

### `cargo-test-suite-expand-recall.log`

Command:

```text
cargo test -p ecaz-cli expands_recall_with_defaults
```

Result:

```text
test commands::bench::suite::tests::expands_recall_with_defaults ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 355 filtered out; finished in 0.00s
```

### `cargo-test-posting-scratch-soa-no-run.log`

Command:

```text
cargo test posting_scratch_soa --no-default-features --features pg18 --no-run
```

Result:

```text
Finished `test` profile [unoptimized + debuginfo] target(s) in 2m 36s
```

The actual library test binary is not executed here because direct test-binary
execution in this local pgrx-linked session hits the existing
`undefined symbol: pg_re_throw` loader issue. The authoritative runtime
evidence for this slice is the PG18 benchmark suite in
`benchmarks/task51-local-ivf-scratch-soa/`.

### `git-diff-check.log`

Command:

```text
git diff --check
```

Result: passed with no output.

## Benchmark Evidence

Benchmark packet:

- `benchmarks/task51-local-ivf-scratch-soa/`

Authoritative suite status:

```text
[suite:task51-local-ivf-scratch-soa] completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Key result lines cited by `request.md`:

- recall parity: static and scratch SoA both `recall@10=0.9750`,
  `recall_p10=0.9000`, `ndcg@k=0.9986`;
- latency p50: `603.7 ms -> 590.5 ms`;
- EXPLAIN execution: `586.336 ms -> 570.902 ms`;
- identical work counts: posting pages `5192`, postings scored `138476`,
  heap TIDs scored `138476`, rerank rows `50`, heap blocks fetched `48`.

## Decision Record

This packet does not meet Task 51 Exp 3's promotion gate. The current scratch
SoA prototype should remain opt-in, should not go to AWS as a standalone
optimization, and should not trigger Posting Layout v2 work in this round.
