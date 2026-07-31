# Task 210 P2a review request: the head is sharded across the roster

Commits under review: `6150f5bc6` → `161680298` (partition + per-shard graphs,
owner shard resolution, owner search endpoint, coordinator fan-out,
membership-only persistence, shape-derived read path).

## What changed

The FR-080 head was the only structure `DISTRIBUTEDANN` distributes that
ec_distann kept central: 4,096 full-precision f32 landmarks (25,280,512 bytes)
plus their graph, on the coordinator, constant in `N` — which is exactly why
NFR-021's removed constant-`C` exemption used to bless it.

Two facts made the fix cheap, and both are now proven rather than assumed:

1. **Head vectors never move.** A landmark's full-precision vector already lives
   on the owner its FR-078 placement hash selects, because the co-placed row
   tier uses the identical hash (ADR-085 D11). Each owner materialises its own
   shard from local reads; the coordinator keeps a bounded membership list.
2. **Shard graphs are built per shard.** A subgraph of the stitched head graph
   is not navigable over a shard — the same reason §3 builds the head from
   per-partition top layers.

## Result

| | control | sharded |
|---|---:|---:|
| coordinator resident bytes (100k) | 25,894,607 | **53,440** |
| recall@10 10k / 50k / 100k | 0.9990 / 0.9545 / 0.9275 | 0.9990 / 0.9545 / **0.9300** |
| warm mean 10k / 50k / 100k | 27.30 / 37.73 / 36.10 ms | 28.28 / 38.31 / **35.68** ms |

485× less coordinator state, recall identical-or-better at every scale, and
latency within a few percent — a small cost at 10k/50k, a small win at 100k
where head work spreads across three nodes instead of one.

Full numbers, provenance, and the eleven-attempt defect ledger are in
`artifacts/manifest.md`; structured rows in `artifacts/run/results.jsonl`.

## What this packet does NOT claim

- **§4.1 replication is not demonstrated.** The `head_replica_count=2` arm
  reports `head_replica_fallbacks=96`: routing was requested, no replica holds a
  shard copy, every request clamped back to its owner. That proves the clamp
  prevents mis-routing and nothing more. Populating replicas needs a
  publish-time step; `ec_distann_head_shard_export` is its transport and is
  unwired. The arm is registered `context` saying exactly this.
- **`outstanding_distribution_gap` is not `none`.** 53,440 bytes of
  empty-neighbour head-graph rows remain on the coordinator — bounded and
  constant in `N`, but non-zero, and the phase's bar was zero.

## Reviewer questions

1. Should the residual 53,440 bytes be eliminated (stop persisting the
   empty-graph rows) or retired as a justified bounded entry in
   `NFR_021_KNOWN_DISTRIBUTION_GAPS`? I lean toward eliminating it — a gap entry
   that survives on "the number is small" is how the constant-`C` exemption
   happened in the first place.
2. The latency cost at 10k/50k is a real regression at small scale. Acceptable
   as the price of the invariant, or worth a bounded coordinator-side head cache
   (which would need its own NFR-021 screen)?
3. Defect 2 in the ledger — the storage GUC set on the wrong statement — passed
   recall, latency, and every `pass=true` gate while doing nothing. Only the P0
   emitter caught it. Is there a cheaper standing check for "this mechanism ran"
   than an emitter per property?

Request open.
