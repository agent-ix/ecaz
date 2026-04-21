# Review Request: resolve KNN `<#>` operators through extension types, not `pg_catalog`

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/commands/bench/recall.rs`
- `crates/ecaz-cli/src/commands/compare/pgvector.rs`

## What this packet is

This is another generic `ecaz-cli` blocker fix that needs to stand alone so it
can merge to `main` independently of the DiskANN measurement packet.

While driving the canonical real-corpus task-17 path, the benchmark KNN SQL was
hard-coding:

```sql
ORDER BY embedding OPERATOR(pg_catalog.<#>) ...
```

That is the wrong operator boundary. Both the ecaz corpus table and the
pgvector sidecar rely on extension-defined `<#>` operators chosen from the
operand types. Forcing `pg_catalog` bypasses that resolution path.

## What changed

### `crates/ecaz-cli/src/commands/bench/recall.rs`

`build_knn_sql(...)` now emits:

```rust
"SELECT id FROM {corpus_table} \
 ORDER BY embedding <#> \
 {enc}($1::real[], $2::integer, $3::bigint) \
 LIMIT $4"
```

The test that pins the recall SQL now also asserts the statement does not
contain `pg_catalog`.

### `crates/ecaz-cli/src/commands/compare/pgvector.rs`

`build_pgvector_knn_sql(...)` now emits:

```rust
"SELECT id FROM {sidecar} \
 ORDER BY embedding <#> \
 $1::real[]::vector({dim}) \
 LIMIT $2"
```

Its unit test now also asserts the generated SQL does not contain
`pg_catalog`.

## Why this slice

- This is the smallest correct fix for a real canonical-path blocker on the
  DiskANN recall/compare surfaces.
- It is generic, not DiskANN-specific. Any `ecaz bench recall` or
  `ecaz compare pgvector` run that needs `<#>` should let Postgres resolve the
  operator from the actual operand types instead of pinning `pg_catalog`.
- The fix stays tight: two SQL builders and two test assertions, with no
  surrounding refactor.

## Operator outcome

After this change, the benchmark/compare SQL matches the intended extension
surface:

- ecaz KNN resolves `<#>` against `ecvector`
- pgvector KNN resolves `<#>` against `vector`

That removes a generic query-generation blocker from the canonical real-corpus
DiskANN measurement path.

## Test evidence

```text
$ cargo test -p ecaz-cli --quiet

test result: ok. 219 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

Also passed for this checkpoint on `pg18`:

- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- The real-10k DiskANN Recall@10 artifact itself. This packet only fixes the
  last generic KNN SQL generation blocker on that path.
- Any wider query-builder cleanup. The scope here is only the incorrect
  `pg_catalog` qualification of `<#>`.
