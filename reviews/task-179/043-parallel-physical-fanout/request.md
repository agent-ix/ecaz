# Review request: parallel physical owner fanout

## Scope

Please review implementation commit `5a48c7ee9` as the code remediation for
packet 029 P2-1/P2-2/P3 and packet 036 P2-1.

The logical M2 transport already used `join_all`, but the active physical read
path entered `with_transport_state`/`runtime.block_on` once per remote owner.
Both hop expansion and frozen-row materialization therefore paid the sum of
remote owner round trips.

This checkpoint:

- constructs one physical request per non-local owner;
- enters the pooled transport runtime once per owner batch;
- establishes/refreshes each pooled connection outside the hot fanout;
- drives all owner expansion futures together with `join_all`;
- does the same for row-payload materialization;
- preserves position-driven response count/order validation;
- replaces missing-route `expect` panics with classified
  `EC_NODE_DESCRIPTOR` errors; and
- treats a missing immutable physical row-tier payload as
  `EC_GENERATION_MISSING` instead of silently dropping the top-k hit.

The local owner remains in-process. Remote result vectors retain roster/bucket
order, so reconstruction still uses the original request positions.

## Validation

See `artifacts/manifest.md` for exact-SHA PG18 evidence. The focused transport
unit suite includes a timing guard in which three 100 ms owner futures must
finish within 230 ms; a serial execution would require about 300 ms. The
existing three-owner physical handoff regression also passes.

## Benchmark status

This is not a performance closeout packet. Because the checkpoint changes scan
and materialization behavior, Task 179 still requires an isolated before/after
`ecaz bench suite` matrix at 10k/50k/100k with recall, latency, storage, and
physical topology evidence. That measurement will be stored separately and
cited before this finding or task is treated as closed.

## Requested decision

Please review the batching, error propagation, and response reassembly now.
Please do not treat static/unit evidence as the latency closeout; that decision
awaits the required A/B suite packet.
