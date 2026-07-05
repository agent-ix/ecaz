# Review Request: Relation Cache Evict CLI

## Summary

Adds `ecaz dev evict-relation-cache` as a narrow local helper for Task 111h cold-cache evidence. The command resolves benchmark-table relation files through PostgreSQL catalogs and evicts their local OS page-cache residency with `posix_fadvise(POSIX_FADV_DONTNEED)`.

This is an enabling code slice only. It does not present cold-cache rerank benchmark results; those should be produced by a follow-on `ecaz bench suite` packet that invokes this helper as a raw step before latency measurements.

## Code Under Review

- `crates/ecaz-cli/src/commands/dev/mod.rs`
- `crates/ecaz-cli/src/commands/dev/relation_cache.rs`

Commit under review: `1751bf572205a31173859c79bd8fdec199f6f6ad`

## Behavior

- `--prefix <prefix>` resolves `<prefix>_corpus`, its indexes, toast table, and toast indexes.
- `--relation <regclass>` resolves explicit local relation files.
- Relation segment and fork files are included, including `base`, `base.N`, `base_fsm`, and `base_vm`.
- `--dry-run` prints the files and bytes without calling `posix_fadvise`.
- The command fails if no prefix/relation is supplied, no relations resolve, or resolved relations have no local files.

## Validation

- `artifacts/cargo-test-relation-cache.log`
  - `cargo test -p ecaz-cli relation_file_match_includes_segments_and_forks`
  - Result: `1 passed; 0 failed`
- `artifacts/dev-evict-relation-cache-help.log`
  - `cargo run -p ecaz-cli -- dev evict-relation-cache --help`
  - Result: command exits `0` and exposes `--prefix`, `--relation`, `--dry-run`, and common connection/logging flags.

Manifest: `artifacts/manifest.md`

## Review Focus

- Confirm the PostgreSQL catalog resolution covers the benchmark objects needed for local cold-cache latency suites.
- Confirm relation file matching includes PostgreSQL segments and forks without overmatching unrelated relation files.
- Confirm this is acceptable as the suite-level primitive for local OS page-cache eviction before Task 111h cold-cache measurements.
