# Review request: owner query fixed-cost hardening

## Scope

Code checkpoint `4587c0d09` implements the code portion of packet 060's P2-6
and the associated query-path P3 findings. It also consolidates the duplicated
wire/shape/identifier helpers called out by the review. The required BW/H and
10k/50k/100k benchmark evidence is intentionally not claimed by this packet;
that measurement remains an open closeout gate.

## Changes

- Physical owner endpoints cache immutable retained-epoch metadata and prepared
  quantizer state, while reopening generation relations for each transaction.
- Pooled physical connections retain prepared statements. Cold connections,
  timeout refreshes, and independent identity setup are driven concurrently.
- The query vector is sent once per pooled owner session and later hops use a
  digest plus a non-null empty-array cache-hit sentinel. A secure six-argument
  SQL wrapper exposes that protocol without weakening endpoint privileges.
- The coordinator persists the bounded head graph once with its own canonical
  digest and loads it directly in each scan backend. Generation descriptors are
  shared through `Arc` rather than cloned with their codec artifacts.
- Beam traversal uses a deterministic binary heap rather than sorting the full
  accumulated frontier every round.
- `quote_ident`, fixed digests, NULL-bitmap validation, handoff-shape
  construction, and transport pool setup now have single authorities.
- The raw `EC_DISTANN_CONTROL_SCAN` AM backstop again has focused test coverage;
  stale module documentation and the options dead-code allowance are explained.

## Validation

At code checkpoint `4587c0d0980bf7c2d56c3dbb751ec36e4492ff08`:

- focused PG18 three-owner physical build/publish/CustomScan: 1 passed;
- physical query cache unit test: 1 passed;
- persisted head graph unit tests: 2 passed;
- production-library PG18 clippy with warnings denied: pass; and
- `git diff --check`: pass before commit.

The nullable cached-query prototype initially exposed the missing six-argument
SQL wrapper as an aborting pg_test assertion. The final non-null protocol and
secure wrapper are what the committed three-owner test exercises.

## Requested verification

Please review retained-cache transaction safety, prepared-statement/session
cache invalidation, persisted-head validation, heap ordering, and endpoint
privileges. This request remains open for outside feedback, and performance
closeout remains pending the separately recorded benchmark matrix.
