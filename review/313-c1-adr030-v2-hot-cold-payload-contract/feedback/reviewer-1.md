## Feedback: ADR-030 v2 Hot/Cold Payload Contract

Read the `TqGroupedHotTuple` and `TqRerankTuple` types in `src/am/page.rs` along with the
new `TQ_GROUPED_HOT_TAG = 0x03` / `TQ_RERANK_TAG = 0x04` tag space.

### Strong parts

- Hot tuple carries only what the traversal inner loop needs: level, deleted bit,
  heaptids, neighbortid, reranktid, binary sidecar words, grouped search code. Cold
  rerank tuple carries gamma + rerank code. That is the right split — the traversal
  never needs to touch rerank bytes.
- Grouped search code is packed as 4-bit nibbles (even in low, odd in high). That
  matches the eventual vpshufb LUT representation, so the on-disk and in-register
  shapes align.
- Tag space does not collide with ADR-031 binary-sidecar tags. Good.

### Sizing check

For 1536-dim (`group_size = 16`, 96 subvectors × 4 bits = 48 B search code), the hot
tuple is ~316 B vs ~1034 B for the scalar tuple. That's a 3.3x reduction per in-memory
graph tuple, which is the main runtime payoff of this lane. Worth recording that
measurement explicitly in a packet somewhere so the tradeoff is not lost in noise.

### What's left

- The `reranktid` is an ItemPointer to a cold tuple, but no packet yet exercises the
  cold-tuple fetch path. Worth an early smoke test (even before the real scorer) that
  inserts a grouped tuple, reads the reranktid, and fetches the cold tuple via its
  ItemPointer. Catching a TID-packing bug after the scorer lands will be much harder.
- No ADR-030 packet yet defines what happens when a grouped hot tuple exists but the
  corresponding rerank tuple is missing (split transaction, half-vacuumed index, etc.).
  Worth adding to the contract doc before insert/vacuum land.
