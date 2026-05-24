# Task 59 Packet 002: Duplicate Expansion Fast Path

## Summary

Code commit under review:

- `1500c303df8ef08e9fb65f5d1c8434087fbc64cb`
  (`Skip DiskANN duplicate expansion reads`)

This checkpoint removes an avoidable post-scan read path in `ec_diskann`.
The SQL scan path materializes the full `rerank_budget` because executor
`LIMIT` is not visible to `amrescan`; before this change, duplicate heap-TID
expansion reread every returned graph node to check `has_overflow_heaptids`.

The scan shell now carries `has_overflow_heaptids` from each decoded
`VamanaNodeTuple` into `ScanCandidate` and `ScanResult`. Both the relation
reader path and the materialized-chain fallback now skip duplicate expansion
lookups for non-overflow nodes and emit the primary heap TID directly. Overflow
nodes still take the existing owner/overflow-chain path.

## Why This Slice

On the normal DBpedia/OpenAI benchmark corpus, duplicate-bound nodes should be
rare. At `rerank_budget` / `list_size` values up to 800, the previous SQL
materialization path could add up to 800 extra index tuple reads per query
after the actual greedy traversal and rerank work had already completed.

This is a targeted Graviton tuning slice before the final 10k/50k/100k/1M
suite. It does not change traversal, prefilter scoring, exact rerank, graph
quality, or overflow semantics.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,pg_test`
  passed; see `artifacts/cargo-check-pg18-pg-test.log`.
- `cargo test scan:: --no-default-features --features pg18,pg_test` built the
  test binary, then failed before running tests with local dynamic loader
  symbol error `undefined symbol: CacheRegisterRelcacheCallback`; see
  `artifacts/cargo-test-scan-pg18-pg-test.log`.

## Benchmark Status

No AWS benchmark result is claimed in this packet yet. The next step is a
focused `ecaz bench suite` Graviton run comparing the prior optimized baseline
against this commit at high `list_size` cells, then the full 10k/50k/100k/1M
suite after the remaining tuning/profile choice is settled.
