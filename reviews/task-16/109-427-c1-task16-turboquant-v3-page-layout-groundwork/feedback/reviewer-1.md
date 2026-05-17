## Feedback: TurboQuant V3 page-layout groundwork — ACCEPTED

Verified against:

- commit `0ab91db` adding `INDEX_FORMAT_V3_TURBO_HOT_COLD`,
  `TQ_TURBO_HOT_TAG`, and `TqTurboHotTuple` / `TqTurboHotTupleRef`
- `src/am/page.rs` graph-storage classification now accepting V3 as
  a turboquant format (V1 | V3 → TurboQuant; V2 → PqFastScan)
- new Miri-style tuple roundtrip coverage alongside single-page and
  chain-extension tests

### What's right

- **Dormant by construction.** No build/scan/insert/vacuum path
  reads or writes V3 in this packet. That is the right shape for a
  wire-format introduction — the on-disk layout can be reviewed
  without racing against runtime behavior.
- **Metadata helper is explicit, not implicit.**
  `MetadataPage::current_v3_turbo_hot_cold(...)` names the format
  bump at its use site instead of relying on
  `graph_storage_format()` to silently flip semantics. Easier to
  audit than a naked constant change.
- **Tag identity picked cleanly.** `TQ_TURBO_HOT_TAG = 0x06`
  doesn't collide with any of the five existing tags, and the
  `TqTurboHotTuple` shape (inline heap TIDs + neighbor TID + cold
  rerank TID + optional binary-sign sidecar) is a recognizable
  analogue of the pq_fastscan hot tuple.
- **Test surface covers encode/decode, borrowed access, single-page
  roundtrip, multi-page chain, and Miri-style layout.** That is
  genuinely the right low-level matrix for a new tuple tag.

### Concerns

1. **V1 and V3 are both classified as `TurboQuant` by the same
   match arm.** `graph_storage_format()` now returns `TurboQuant`
   for both, which is correct at the format-family level but means
   downstream storage-descriptor code has to distinguish hot/cold
   V3 from inline V1 by *something else* (presumably payload flags
   plus format_version). Packet `428` has to do that work. Worth a
   comment on `graph_storage_format()` noting that version-level
   distinction lives downstream, not here.
2. **No "V3 implies a specific payload flag set" invariant pinned
   yet.** Turbo-hot tuples carry an optional binary sidecar, but
   there is no explicit test that a V3 metadata page with
   `PAYLOAD_FLAG_BINARY_SIDECAR` off still roundtrips correctly, and
   vice versa. Low stakes because packet `428` wires the live path,
   but easier to close here than after runtime lands.
3. **Cold rerank TID inside `TqTurboHotTuple` but no explicit
   "unset" sentinel test.** If `Tq_Turbo_Hot_Tag` tuples could ever
   be written before their cold rerank payload (e.g. crash-torn
   build), a reader needs to know whether "cold TID = InvalidTid"
   is valid. Packet `428` answers this by making build always write
   both, but worth making the invariant a debug assertion at the
   tuple-ref accessor.

### Call

Accepted. This is the right size and shape of a dormant substrate
packet. Review cost paid here makes packet `428` much easier to
audit because the runtime wiring isn't also moving the wire format
around.
