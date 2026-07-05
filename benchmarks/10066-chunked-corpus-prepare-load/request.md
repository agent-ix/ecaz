# Review Request: Chunked Corpus Prepare and Resumable Load

Scope:
- `crates/ecaz-cli/src/commands/corpus/prepare.rs`
- `crates/ecaz-cli/src/commands/corpus/load.rs`
- `crates/ecaz-cli/src/manifest.rs`

Task: `plan/tasks/coder2/10066-chunked-corpus-prepare-load.md`

## Problem

`ecaz corpus prepare` currently writes one corpus TSV and one query TSV,
which is awkward for the 990k/10k DBpedia anchor profile because the
corpus file is very large and the prepare path materializes all selected
vectors in memory before writing output. `ecaz corpus load` is also
single-shot: if a large load is interrupted, the operator has to restart
from the beginning.

Task 10066 asks for two durable improvements without changing index-build
semantics:

- an opt-in chunked artifact layout for prepare
- a resumable chunk-aware load path up to the point where index build can
  rerun independently

## Change Summary

### Prepare

`ecaz corpus prepare` now accepts `--chunk-rows`. When unset, the legacy
single-TSV path is unchanged. When set:

- output layout becomes:
  - `corpus/corpus-00000.tsv`, ...
  - `queries/queries-00000.tsv`, ...
  - sibling `<prefix>_manifest.json`
- pass 2 no longer materializes all selected vectors in memory
- instead, selected rows are streamed from parquet into per-chunk spill
  files keyed by output ordinal
- each chunk is sorted locally, hashed, written through `*.tmp`, and then
  atomically renamed to its final `corpus/...` or `queries/...` path

The manifest remains `manifest_version = 1` and gains additive chunked
metadata:

- `artifact_layout = "chunked"` or `"single_tsv"`
- `chunk_rows`
- per-section `chunks[]` entries with relative path, kind, row range,
  row count, byte length, and sha256

### Load

`ecaz corpus load` keeps the existing single-file path, but now also
accepts chunked manifests via `--manifest-file` and optional `--chunked`.

For chunked loads it now:

- parses and validates the chunked manifest shape
- resolves relative chunk paths from the manifest directory
- creates `ecaz_corpus_load_state` if absent
- verifies each chunk file’s row count, byte length, and sha256 before load
- loads one chunk per transaction through a temp staging table
- records the chunk state row in the same transaction
- skips previously loaded chunks whose state rows match the manifest
- rejects inconsistent table/state combinations instead of silently
  continuing

The table contract is unchanged after a successful load:

- `<prefix>_corpus`
- `<prefix>_queries`

### Tests

Added focused unit coverage for:

- chunk manifest parsing and validation
- chunk planning and row-range assignment
- chunk finalization ordering/checksum behavior
- chunk-state resume validation
- chunked manifest discovery in the loader

## Validation

Ran:

```bash
cargo test -p ecaz-cli corpus
```

Result: 51 tests passed, 0 failed.

## Review Focus

1. Is the additive manifest shape acceptable under the existing
   `manifest_version = 1` contract?
2. Is the spill-then-sort-per-chunk prepare path the right first-pass
   bounded-memory tradeoff for the anchor dataset?
3. Is the loader’s “reject inconsistent table/state” behavior strict
   enough, especially for partial manual cleanup cases?
4. Is encoding corpus embeddings during per-chunk insert the right choice,
   versus a later whole-table `UPDATE`?

## Not Done Here

- No PostgreSQL index-build resume work
- No real parquet / PG18 end-to-end run on this branch yet
- No measurement packet or raw load logs yet
