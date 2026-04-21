# Review Request: force ecaz measurement commands onto the ordered ANN path

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/psql.rs`
- `crates/ecaz-cli/src/commands/bench/recall.rs`
- `crates/ecaz-cli/src/commands/bench/latency.rs`
- `crates/ecaz-cli/src/commands/bench/overhead.rs`
- `crates/ecaz-cli/src/commands/compare/pgvector.rs`

## What this packet is

This is the next generic `ecaz-cli` blocker fix that needs to stand alone so
it can merge to `main` independently of the DiskANN measurement packet.

While driving the canonical real-corpus task-17 path, the benchmark query had
already been corrected to use the raw `real[]` ORDER BY operand expected by the
AM scans. That removed the type-shape bug, but the live pg18 planner still
preferred:

```text
Seq Scan on public.ec_hnsw_real_10k_corpus
  -> Sort
```

for the real 10k corpus unless `enable_seqscan`, `enable_bitmapscan`, and
`enable_sort` were disabled in-session.

That is the wrong operator boundary for `ecaz bench` / `ecaz compare`:
those commands exist to measure the selected ANN access method, not whatever
fallback plan the planner happens to prefer on a given corpus.

## What changed

### `crates/ecaz-cli/src/psql.rs`

Added a shared helper:

```rust
pub async fn prefer_ordered_ann_path(client: &Client) -> Result<()> {
    client
        .batch_execute(
            "SET enable_seqscan = off;
             SET enable_bitmapscan = off;
             SET enable_sort = off",
        )
        .await
        .wrap_err("forcing ordered ANN plan shape")?;
    Ok(())
}
```

### Measurement call sites

The helper is now applied before timed KNN work in:

- `bench recall`
- `bench latency` worker sessions
- `bench overhead`
- `compare pgvector`

No loader/storage/admin paths changed.

## Why this slice

- This is the smallest correct fix for a real DiskANN measurement blocker on
  the canonical path.
- It is generic, not DiskANN-specific. The same benchmark/compare surfaces are
  supposed to measure `ec_hnsw` and `ec_diskann`, and both would be vulnerable
  to seqscan/sort fallback when the planner declines the ANN index.
- The commands already preflight “does the requested AM index exist?”; this
  change completes that contract by making the timed query actually use the
  ordered ANN path instead of a fallback plan.

## Operator outcome

After this change, the measurement sessions no longer rely on natural planner
selection for ordered ANN scans. The selected access method is what gets
timed, which is the only behavior that makes `bench recall` / `latency` /
`overhead` / `compare pgvector` meaningful for task 17.

## Test evidence

```text
$ cargo test -p ecaz-cli --quiet

test result: ok. 218 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also passed for this checkpoint on `pg18`:

- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- The full real-10k DiskANN Recall@10 artifact. This packet only fixes the
  last generic measurement-path blocker uncovered while capturing it.
- Planner cost-model tuning. If the natural planner should prefer these ANN
  paths more often without guardrails, that belongs in extension cost work,
  not in this narrow measurement-surface fix.
