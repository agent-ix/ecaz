# Review Request: `ecaz corpus load` ensures the `ecaz` extension exists

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/commands/corpus/load.rs`

## What this packet is

This is another generic `ecaz-cli` blocker fix that needs to stand alone so it
can merge to `main` independently of the DiskANN measurement packet.

While driving the canonical real-corpus task-17 path on a fresh pg18 scratch
database:

```text
ecaz corpus fetch
ecaz corpus prepare
ecaz corpus load
ecaz bench recall
```

`ecaz corpus load` failed after manifest verification with:

```text
creating table ec_hnsw_real_10k_corpus
db error
ERROR: type "ecvector" does not exist
```

That is the wrong operator boundary. If `corpus load` is the supported way to
bootstrap a fresh measurement database, it should not require a manual
`CREATE EXTENSION ecaz` step first.

## What changed

### `crates/ecaz-cli/src/commands/corpus/load.rs`

Right after connecting, `run(...)` now does:

```rust
client
    .batch_execute("CREATE EXTENSION IF NOT EXISTS ecaz")
    .await
    .wrap_err("ensuring ecaz extension")?;
```

Nothing else changed:

- manifest verification still runs before any DB writes
- corpus/query table reload logic is unchanged
- index planning and reloption handling are unchanged

## Why this slice

- This is the smallest correct fix for a real canonical-path failure on a
  fresh target database.
- It is generic, not DiskANN-specific. Any `ecaz corpus load` on a database
  without the extension would hit the same `ecvector` type failure.
- The pattern already exists in `compare pgvector`, which ensures
  `CREATE EXTENSION IF NOT EXISTS vector` before touching the sidecar table.
  `corpus load` now applies the same operator standard to the `ecaz`
  extension itself.

## Operator outcome

After this change, the exact same direct command that previously failed on
`type "ecvector" does not exist` proceeded to:

- create/load `ec_hnsw_real_10k_corpus`
- create/load `ec_hnsw_real_10k_queries`
- encode `ecvector` embeddings
- build the `ec_diskann` index on the real-10k corpus

That unblocks the next task-17 step: the actual `bench recall` capture.

## Test evidence

```text
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 219 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also passed for this checkpoint on `pg18`:

- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- The real-10k DiskANN Recall@10 artifact itself. This packet only fixes the
  last generic loader blocker on the canonical path.
- Broader database bootstrap/admin surfaces. `corpus load` only needs the
  narrow `CREATE EXTENSION IF NOT EXISTS ecaz` step.
