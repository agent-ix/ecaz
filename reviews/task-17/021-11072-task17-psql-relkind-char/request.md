# Review Request: Encode `pg_class.relkind` correctly in `ecaz-cli`

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/psql.rs`

## What this packet is

This is a generic `ecaz-cli` blocker fix that needs to stand alone so it can
merge to `main` independently of the DiskANN closeout lane.

The canonical task-17 path is:

- `ecaz corpus fetch`
- `ecaz corpus prepare`
- `ecaz corpus load`
- `ecaz bench recall`

While driving that exact path against the local pg18 scratch cluster,
`ecaz corpus load` failed before it could create the real-10k DiskANN corpus:

```text
checking relation "ec_hnsw_real_10k_corpus" exists
error serializing parameter 1
cannot convert between the Rust type `alloc::string::String` and the Postgres type `char`
```

The bug was in shared CLI plumbing, not in DiskANN itself: `psql::relation_exists`
was binding `pg_class.relkind` as a Rust `String` even though Postgres expects
its internal `"char"` catalog type. `corpus list` did not expose this because
the empty-database path never hits `relation_exists`; `corpus load` does.

## What changed

### `crates/ecaz-cli/src/psql.rs`

- Added a narrow helper:

```rust
fn encode_relkind(relkind: char) -> Result<i8> {
    i8::try_from(u32::from(relkind))
        .wrap_err_with(|| format!("relkind {:?} must be an ASCII catalog code", relkind))
}
```

- `relation_exists(...)` now binds `relkind` as an ASCII byte and casts it to
  Postgres `"char"` explicitly:

```rust
WHERE relname = $1
  AND relkind = $2::"char"
```

- Added unit tests pinning:
  - `'r'` / `'i'` encode to the expected catalog byte values
  - non-ASCII values are rejected before we talk to Postgres

## Why this slice

- This is the smallest correct fix for a real shared-CLI bug on the canonical
  DiskANN measurement path.
- It is generic and main-ready: any `ecaz-cli` command that calls
  `relation_exists(...)` was vulnerable, not just DiskANN.
- Keeping it isolated avoids mixing reusable CLI plumbing with the
  task-17-specific real-corpus measurement packet.

## Test evidence

```text
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 218 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also passed for this checkpoint on `pg18`:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- The actual pg18 real-10k DiskANN load / Recall@10 artifact. That remains a
  separate task-17 slice after this generic blocker fix lands.
- Any broader `ecaz-cli` SQL cleanup. This packet changes one shared helper
  and stops.
