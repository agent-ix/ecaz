# Review Request: Relation-Cache Reloptions

Scope:
- `src/am/mod.rs`

What changed:
- `relation_options` no longer issues an SPI query against `pg_class` to read reloptions.
- It now reads `m`, `ef_construction`, and the optional `build_source_column` directly from the relation descriptor's cached `rd_options`.
- The rest of the build and insert option flow is unchanged; this slice only removes the catalog query from the hot path.

Review focus:
- Whether direct `rd_options` access is the right reloptions boundary for this access method
- Whether the string reloption decoding is correct for the current `TqHnswReloptions` layout
- Whether existing build and insert coverage is sufficient for this narrow hot-path fix

Questions to answer:
- Is using the stored string offset from `TqHnswReloptions` the correct way to decode `build_source_column` here?
- Is there any compatibility or lifetime risk in reading `rd_options` this way across build and insert entry points?
- Should this slice add a dedicated regression helper around `build_source_column`, or do the existing build and insert paths already cover the important behavior?
