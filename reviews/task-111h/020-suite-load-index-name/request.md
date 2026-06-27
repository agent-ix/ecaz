# Task 111h / Packet 020 Review Request: Suite Load Index Name

## Summary

This packet requests review for a small `ecaz bench suite` runner extension
needed by the Task 111h 1M shared-table benchmark plan.

The code checkpoint is:

- `7acfc4985786eb2ac93dd1b98159d2f8d44fe910` `task111h: allow suite load index names`

The change adds an optional `index_name` field to load steps and forwards it to
`ecaz corpus load --index-name`. This lets a suite create one shared corpus
table while giving each one-index-at-a-time surface a deterministic index name.

## Why This Exists

The local 1M rerank matrix is not practical as isolated one-table-per-cell
surfaces on the current disk. A shared-table matrix with one active IVF index at
a time avoids duplicating the 1M heap table for every format/width cell, but the
suite runner previously had no way to pass an explicit index name through load
steps.

This is a runner plumbing change only. It does not alter index behavior,
storage layout, query behavior, or benchmark interpretation.

## Validation

Packet-local test log:

- `artifacts/cargo-test-ecaz-cli-suite-load-index-name.log`

Result:

```text
test commands::bench::suite::tests::expands_chunked_load_without_corpus_query_paths ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 407 filtered out; finished in 0.00s
```

## Review Ask

Please review whether the suite runner should expose `load.index_name` this way
for shared-table benchmark surfaces, and whether the focused argument-expansion
coverage is sufficient for this narrow plumbing change.
