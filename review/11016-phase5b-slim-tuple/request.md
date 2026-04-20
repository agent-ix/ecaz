# Review Request: Phase 5B — Slim `VamanaNodeTuple` per ADR-045

Branch: `adr034-diskann-access-method`
Author: coder-2
Companion to: 11014 (ADR-045), 11015 (Phase 5A vamana algorithm core)

## What this slice is

Second sub-slice of task 17 **Phase 5**: rewrite
`src/am/diskann/tuple.rs` from the original draft (which mirrored
tqhnsw's element-tuple shape) to the ADR-045 reference layout. This
unlocks Phase 5C's placeholder-then-patch persistence pattern by
making encoded length a pure function of the metadata-page
constants `(R, W, C)`.

## Scope

- `src/am/diskann/tuple.rs` — full rewrite (~454 lines including 13
  unit tests).

No other source files touched. No metadata-page changes (existing
`VamanaMetadataPage` already carries the fields needed to derive `W`
and `C` at decode time).

## What changed

### Before / after at a glance

| | Original draft | Slim layout (ADR-045) |
|---|---|---|
| Header bytes | 76 | 16 |
| Inline heaptid slots | 10 (`HEAPTID_INLINE_CAPACITY`) | 1 (`primary_heaptid`) |
| `graph_degree_r` per tuple | yes | no (read from metadata) |
| `binary_word_count` per tuple | yes | no (read from metadata) |
| `search_code_len` per tuple | yes | no (read from metadata) |
| Encoded length | varies with `heaptid_count` | fixed per (R, W, C) |
| Tuple bytes at 1536d / R=32 / grouped-PQ4 | ~660 | ~464 |
| Tuples per 8KB page | ~12 | ~17 |

### New layout

```text
[0]  tag: u8                          = TQ_VAMANA_NODE_TAG (0x06)
[1]  flags: u8                        (bit 0 = deleted, bit 1 = has_overflow_heaptids)
[2]  neighbor_count: u16              (filled prefix of neighbor_slots)
[4]  primary_heaptid: ItemPointer     (6)
[10] rerank_tid: ItemPointer          (6)   -- INVALID per ADR-044 default
[16] binary_words:   [u64; W]              -- W from metadata.dimensions.div_ceil(64)
     search_code:    [u8;  C]              -- C from metadata.search_subvector_count.div_ceil(2)
     neighbor_slots: [ItemPointer; R]      -- R from metadata.graph_degree_r; tail = INVALID
```

### Public API

```rust
pub const TQ_VAMANA_NODE_TAG: u8 = 0x06;
pub const FLAG_DELETED: u8 = 1 << 0;
pub const FLAG_HAS_OVERFLOW_HEAPTIDS: u8 = 1 << 1;
pub const HEADER_FIXED_BYTES: usize = 16;

pub struct VamanaNodeTuple {
    pub deleted: bool,
    pub has_overflow_heaptids: bool,
    pub primary_heaptid: ItemPointer,
    pub rerank_tid: ItemPointer,
    pub binary_words: Vec<u64>,    // length = W
    pub search_code: Vec<u8>,      // length = C
    pub neighbors: Vec<ItemPointer>, // length = R, tail = INVALID
    pub neighbor_count: u16,
}

impl VamanaNodeTuple {
    pub fn encoded_len(R: u16, W: usize, C: usize) -> usize;
    pub fn placeholder(R: u16, W: usize, C: usize) -> Self;
    pub fn validate(&self, R: u16, W: usize, C: usize) -> Result<(), String>;
    pub fn encode(&self, R: u16, W: usize, C: usize) -> Result<Vec<u8>, String>;
    pub fn decode(input: &[u8], R: u16, W: usize, C: usize) -> Result<Self, String>;
}
```

The `(R, W, C)` triple is threaded explicitly into encode/decode/
validate — the same pattern as tqhnsw's `read_element(tid, code_len)`.
No per-tuple length fields means the decoder *cannot* infer them; the
caller must supply from the metadata page. This is the ADR-045
Decision-1 contract made API-explicit.

`placeholder(R, W, C)` constructs a zero-filled, all-INVALID tuple of
the right shape — used by Phase 5C's persistence pass 1.

### Tests (13, all green)

All eight original layout-assertion tests (LA-010 through LA-017)
still apply, ported to the new constructor signature:

- LA-010 empty round-trip
- LA-011 filled round-trip (primary_heaptid, rerank_tid, binary
  sidecar, code, neighbors)
- LA-012 encoded_len matches encode().len()
- LA-013 foreign tag rejected
- LA-014 validate rejects neighbors Vec whose length ≠ R
- LA-015 validate rejects neighbor_count > R
- LA-016 empty neighbor slots decode as INVALID
- LA-017 deleted flag round-trips

Five new tests covering ADR-045-specific invariants:

- **LA-018** — `has_overflow_heaptids` flag is independent of
  `deleted`. Covers the new flags-byte encoding.
- **LA-019** — **fixed-length invariant** (ADR-045 Decision 3):
  placeholder and fully-filled tuples for the same (R, W, C) encode
  to the same byte length. This is the property that makes Phase 5C's
  placeholder-then-patch persistence sound.
- **LA-020** — header is exactly 16 bytes (locks the reference
  layout).
- **LA-021** — decode rejects payloads of the wrong length with the
  expected error.
- **LA-022** — validate rejects body sizes (`binary_words`,
  `search_code`) that don't match metadata.

```
running 13 tests
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 531 filtered out; finished in 0.00s
```

`cargo check --lib` clean (5 pre-existing dead-code warnings only).
Full diskann module: 25 tests pass (6 page + 13 tuple + 6 vamana).

## Review focus

1. **The `(R, W, C)` triple threaded everywhere.** Encode, decode,
   validate, and `placeholder` all take three numbers from the
   caller. This is verbose at the call site but unambiguously
   documents the metadata-page dependency. The alternative (build
   a `TupleSchema { r, w, c }` struct and pass that) is a small
   ergonomic win but reads less directly. Reviewer preference?
2. **Where do `W` and `C` come from at scan/insert time?**
   `VamanaMetadataPage` carries `dimensions` and
   `search_subvector_count`; the derivation rules are:
   - `W = if PAYLOAD_FLAG_BINARY_SIDECAR { dimensions.div_ceil(64) } else { 0 }`
   - `C = search_subvector_count.div_ceil(2)` (assumes PQ4 nibbles)
   These derivations live in Phase 5C's metadata-loading code, not
   in this packet. Confirm the derivation rules are right before
   5C lands.
3. **`primary_heaptid` is not optional.** A live node always has at
   least one heap row (the originating row); `INVALID` here means
   the node is in placeholder state during persistence pass 1, or
   has been logically deleted (with `deleted = true` to disambiguate
   in pass 2 / vacuum). Reviewer confirm this contract before the
   slim format ships.
4. **`has_overflow_heaptids` flag has no consumer yet.** The flag bit
   is reserved for the rare HOT-chain case where one Vamana node
   backs multiple heap rows. Phase 7 (insert) will add the overflow
   chain itself; for now the flag is always `false` in build-path
   output. Reviewer call: ship the bit reserved with no consumer
   (forward-compat) or hold it back until Phase 7. I went with
   reserved.
5. **Format-version compatibility.** `INDEX_FORMAT_V3_DISKANN = 3`
   is set on the metadata page; the slim tuple is the V3 wire format
   from day one — no V3-old to migrate from because no V3 indexes
   have shipped. Reviewer confirm this reading is correct (i.e., we
   are not breaking a wire format that has on-disk users).

## Questions to answer

- **Should `placeholder()` take a `padding_byte: u8` for fuzz
  testing?** Currently zero-fills. Argument for: flushes out
  decoders that accidentally depend on zero. Argument against:
  YAGNI — every consumer of `placeholder()` is the persistence
  pass-1 path, which writes the zero-filled placeholder verbatim.
  Held: zero-fill only.
- **Do we want a debug-only assertion that all tuples on a chain
  have the same encoded length?** §Open question 3 of ADR-045
  floats this. Could add now to `DataPageChain::insert_raw_tuple`
  with an opt-out for tqhnsw, or wait for drift. I'd defer until
  ADR-045 review settles the call.

## Not doing in this packet

- **Persistence wiring.** Phase 5C (packet 11009) ties the slim
  tuple, the algorithm core, and the metadata page together inside
  the AM build callback.
- **Overflow heaptid chain.** The flag bit is reserved; the chain
  itself is Phase 7 (insert) territory.
- **tqhnsw retrofit.** Out of scope per ADR-045 §Scope.

## Dependencies

- **ADR-045** ACCEPTED — gate for landing this packet. The slim
  layout *is* the ADR-045 reference layout, so 5B cannot land before
  ADR-045 is signed off.
- **ADR-044** — current default ("rerank from heap") is what makes
  unconditional `rerank_tid = INVALID` correct for V1. ADR-044's
  C1 reopen would flip the flag bit but not break the wire format.

## Companion packets

- **11014** — ADR-045 page-layout discipline.
- **11015** — Phase 5A vamana algorithm core (filed alongside).
- **11009** — Phase 5C build → persist plumbing (future).

## Definition of ready

- ADR-045 ACCEPTED (or ACCEPTED-WITH-AMENDMENTS).
- 13 tuple tests green (verified locally).
- Reviewer confirms the `(R, W, C)` threading and the `INVALID`
  primary_heaptid placeholder contract.
- Phase 5C does not start before this lands.

## Handoff notes

The slim layout is mechanical given the ADR-045 reference block —
the rewrite is essentially a 1:1 transcription. The non-mechanical
choices are:

- the `flags: u8` byte (bit-packed `deleted` + `has_overflow_heaptids`,
  six bits reserved)
- the `placeholder()` constructor (a zero-filled, INVALID-pointered
  template for Phase 5C pass 1)
- the explicit `(R, W, C)` API surface (vs. a `TupleSchema` struct)

Each is called out in §Review focus. If any are pushed back, the
rewrite is small enough that re-spinning is cheap — the body of
`encode` / `decode` does not change shape, only the headers around
it.
