# Review request: direct physical graph reader

## Scope

Please review implementation commit `afcc2d6af` as the code remediation for
the remaining packet 020 P2-3b hot-path finding.

Physical hop expansion previously entered `Spi::connect`, formatted a
generation-specific SQL query, parsed/planned it, and copied three SPI columns
for every local hop and every remote expansion RPC. The generation already
owns a unique btree directory on graph-store `vec_id`, so this checkpoint uses
that native storage surface directly:

- opens and lock-holds the graph heap and its immutable directory index with
  the generation reader;
- begins one PostgreSQL index scan descriptor per requested node batch and
  reuses it across the batch with `index_rescan`;
- reads graph tuples into a relation-native table slot, detoasts the record,
  and decodes it without an SPI tuple or intermediate `Vec<u8>` copy;
- applies the same direct reader to coordinator-local hop expansion and the
  retained-generation remote expansion/materialization endpoint;
- cross-checks the requested index key, stored heap `vec_id`, embedded record
  `vec_id`, and stored/embedded row TID before accepting a node; and
- preserves the existing owner-missing versus local structural-error
  classifications and exact output ordering.

The benchmark-only full-owner seed control remains an intentional O(N) SPI
scan; normal production hop expansion and materialization no longer contain a
dynamic graph lookup query.

## Validation

See `artifacts/manifest.md`. At the exact implementation SHA, the live PG18
three-owner fixture builds, publishes, and reads one generation through local
and remote physical expansion plus frozen-row materialization. Strict clippy
passes for both the production feature set and the benchmark-control feature
set.

## Benchmark status

This checkpoint changes scan behavior, so static/live-test evidence is not a
performance closeout. Packet 048's persisted-head arm is the immutable
pre-change 10k/50k/100k baseline. A following canonical suite packet will run
the same persisted-head matrix with `afcc2d6af` installed and compare recall,
warmed latency, storage, topology, and same-data controls before packet 020
P2-3b is treated as closed.

## Requested decision

Please review the native index-scan lifecycle, datum/TID decoding, corruption
cross-checks, snapshot use, and error preservation. Please reserve the latency
decision for the required A/B packet.
