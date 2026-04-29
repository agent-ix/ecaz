# Task 28 IVF Recall Truth Cache

## Scope

Add an opt-in exact ground-truth cache to `ecaz bench recall` for the remaining
A9/A10 recall sweeps.

This is a harness-only change:

- new flag: `--truth-cache-dir <PATH>`
- cache key includes corpus ids, corpus source values, query ids, query source
  values, dimensions, query count, and `k`
- cache payload stores exact top-k truth ids and scores, not the full
  query-by-corpus score matrix
- NDCG keeps the prior semantics by recomputing predicted-id relevance from
  the original source vectors
- exact-truth top-k now uses partial selection plus a final top-k sort instead
  of sorting every corpus score for every query

## Why

The remaining 100k/990k/1M IVF recall runs repeat the same exact-truth setup
across quantizer, nprobe, and rerank surfaces. Reusing exact top-k truth removes
that repeated setup cost while keeping the benchmark surface explicit through a
CLI flag instead of ambient configuration.

The partial top-k selection reduces first-run exact-truth setup work for large
corpora because the harness no longer performs an O(corpus_rows log corpus_rows)
full sort per query when only the top `k` ids are needed.

## Validation

Command:

```text
cargo test -p ecaz-cli recall -- --nocapture
```

Result:

```text
23 passed; 0 failed; 0 ignored
```

Also run:

```text
git diff --check
```

Result: clean.

The same validation was re-run after the partial top-k selection change:

```text
cargo test -p ecaz-cli recall -- --nocapture
```

Result:

```text
23 passed; 0 failed; 0 ignored
```

Also run:

```text
git diff --check
```

Result: clean.

## Review Notes

This packet does not make a new recall or latency claim. It records the harness
change that should make the next A9/A10 measurement packets cheaper and more
repeatable.
