# Review request: cache, batch, and state hardening

## Scope

This packet implements a second narrow slice from packet 060's review. Code
checkpoint `0b2d4fbab` addresses P2-5, P2-7, and two P3 findings concerning
the Ready transition and SPI error preservation. It does not claim that the
remaining packet-060 inventory is complete.

## Changes

- The legacy materialize endpoint now resolves owned rows through
  `cached_index_entry`, reusing the cached directory instead of rebuilding the
  entire on-disk directory for every RPC.
- Physical handoff still creates row-tier tuples individually to obtain their
  PostgreSQL TIDs, but collects the graph payloads and inserts the whole graph
  batch with one `unnest` statement. The returned insert count must equal the
  batch size.
- The `Registered` to `Ready` transition uses `RETURNING 1` and requires
  exactly one changed row, so a stale or invalid lifecycle state cannot be
  silently accepted.
- Reviewed SPI failures now retain PostgreSQL's underlying error detail in the
  coordinator error string.

## Validation

At code checkpoint `0b2d4fbab`:

- PG18 stage-batch atomic replay/directory test: 1 passed, 0 failed;
- PG18 materialize heap-identity callback test: 1 passed, 0 failed;
- PG18 production-library clippy with warnings denied: pass; and
- `git diff --check`: pass before commit.

The callback tests exercise both the new batched graph insert and the cached
legacy directory lookup. These changes do not alter scoring, traversal,
ordering, or storage format, so they do not independently trigger a new
10k/50k/100k benchmark matrix.

## Requested verification

Please verify the packet-060 P2-5/P2-7 fixes, the exact Ready-transition row
count, and preserved SPI diagnostics. This request remains open for outside
feedback.
