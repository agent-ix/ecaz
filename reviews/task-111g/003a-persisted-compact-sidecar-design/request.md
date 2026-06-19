# Task 111g Packet 003a — Persisted compact rerank sidecar: design checkpoint

**Role:** coder
**Branch:** `task-111g-coarse-rerank-representations`
**Base SHA:** `85443b51d` (packets 001+002 + reviewer feedback)
**Status:** design checkpoint — requesting reviewer sign-off on the on-disk shape
and lifecycle **before** the implementation slice (003b) lands.

## Why a design checkpoint first

The packet-001 / 002 reviewer feedback is unambiguous: the dispatch foundation is
correct, but f16/rabitq4 today **re-encode on the fly** from the full f32 heap
source, so there is no byte/IO saving. The 111g win (packet-005 model) requires
**persisting** the compact representation so rerank touches fewer bytes.

While tracing the build/scan paths I found the persisted-sidecar design has a
**real fork in the road** with durable on-disk and hot-path consequences, so per
`CLAUDE.md` ("Add ADRs for design decisions that need durable rationale" and the
003a/003b split the task authorizes) I am gating the storage shape on reviewer
sign-off before writing build/insert/vacuum/scan code.

## Key discovery (grounds the design)

The dense posting block already carries a **per-entry `rerank_tid`**
(`EC_IVF_POSTING_RERANK_TID_OFFSET`, `IvfDensePostingBlockRef::rerank_tid_bytes`,
`from_single_heaptid_postings` → `rerank_tids`). It is written **always-INVALID**
today (`build.rs:680` and `:705` pass `ItemPointer::INVALID`). This is clearly the
packet-005-intended hook: each posting points at its persisted compact payload.
So persisting compact reps does **not** require an on-disk format break — the slot
exists; it is just unused.

## The fork: how does rerank find a candidate's compact payload?

### Option A — carry `rerank_tid` through the candidate (REJECTED)

Set each posting's `rerank_tid` at build/insert, decode it in the dense scan, and
carry it into `EcIvfScoredCandidate` so rerank reads payload at `rerank_tid`.

Rejected because the dense scan hot path (`append_dense_posting_block_to_coalesced_scratch`
→ `IvfPostingScratchSoa` → `process_scratch_soa_postings` →
`record_scored_posting_candidates`) **flattens postings into a coalesced SoA** and
emits candidates by `heap_tids` only. Threading a per-heap-tid `rerank_tid`
through the coalesced SoA batch path would restructure coalescing and the batch
scoring loop — exactly what the task forbids ("do NOT change … coalescing").

### Option B — sidecar keyed by heap TID, looked up at rerank time (RECOMMENDED)

Persist a sidecar mapping **heap-TID → compact payload**, built **sorted by heap
TID**, and have the rerank stage read it **tid-sorted** (the rerank stage already
sorts survivors by heap TID and reads tid-sorted — `rerank_probe_candidates_table_side`
calls `candidates.sort_by(candidate_heap_tid_cmp)` then fetches per candidate).

This keeps the coarse stage, dense posting layout (`0x25`/`0x28`), coalescing, and
quant math **completely untouched** — the only scan change is *which bytes the
rerank loop reads* (sidecar payload vs full f32 heap source). It is the minimal
diff that delivers the byte reduction.

**Recommendation: Option B.**

## Proposed on-disk shape (Option B)

A new index-internal sidecar tuple kind, tag `0x2A`
(`IVF_RERANK_SIDECAR_BLOCK_TAG`), stored in its own page chain in the index
relation (not the heap), referenced by a sidecar head pointer.

**Metadata-page-full finding (reviewer decision needed):** the metadata page is
*exactly* full — `EC_IVF_METADATA_BYTES = 80`, and the last field
(`pq_group_size`) occupies bytes `78..80`. There is **no spare room** for a new
6-byte `ItemPointer` head. Two ways to store the sidecar head:

- **B1 (recommended): bump `EC_IVF_INDEX_FORMAT_VERSION` 2 → 3** and widen
  `EC_IVF_METADATA_BYTES` to `86`, adding
  `EC_IVF_METADATA_RERANK_SIDECAR_HEAD_OFFSET = 80`. `decode` already gates on
  `format_version` range and tolerates `bytes.len() >= METADATA_BYTES`, and the
  metadata buffer is a full page, so widening the struct is backward-compatible
  for *reading* (old v2 indexes decode with sidecar_head = INVALID = "no
  sidecar", i.e. f32 rerank). This is a clean, contained format bump — not a
  layout break of the posting/dense formats the task protects.
- **B2 (no format bump): reuse the always-INVALID per-posting `rerank_tid`** as
  the sidecar pointer. Rejected for the same reason as Option A — it forces the
  dense scan to carry rerank_tid through the coalesced SoA path.

I recommend **B1**: it is the only way to get a heap-TID-keyed sidecar head
without touching the dense scan, and the metadata format-version machinery is
designed for exactly this kind of additive field. Please confirm the version
bump is acceptable (it is the one on-disk-format change in this slice, and it is
additive + backward-readable).

Each sidecar block stores, for a tid-sorted run of entries:

    [tag:u8][entry_count:u16][payload_len:u16][rerank_format:u8]
    [heap_tids: count * ITEM_POINTER_BYTES]      (ascending, the sort key)
    [payloads:  count * payload_len]             (compact f16 or rabitq4 code)

- **f16 payload_len** = `dimensions * 2` (IEEE binary16 per dim) — half of the
  `dimensions * 4` f32 heap source. This is the headline byte win.
- **rabitq4 payload_len** = `IvfQuantizer(rabitq, bits=4).payload_len()` — the
  same compact code the on-the-fly path already produces, now persisted.
- f32 rerank: **no sidecar** (rerank_format=f32 keeps reading the heap source,
  bit-identical — AC1 / the heap_f32 reference path is unchanged).

Blocks are sized to fit a page; the chain is read in block order, and because each
block's heap_tids are globally ascending across the chain, the rerank stage can
**merge-join** the tid-sorted survivor list against the tid-sorted sidecar stream
(or binary-search within a loaded block) — no random IO, no full-source fetch.

## Lifecycle

- **Build** (`stage_build_plan`): the build already has `tuple.source_vector` for
  every row. After postings are written, emit the sidecar chain: for each row,
  encode the compact payload (f16 round-trip pack / `IvfQuantizer::encode_source`
  for rabitq4) keyed by `heap_tid`, sorted ascending by heap TID, packed into
  `0x2A` blocks. Only when `rerank_format ∈ {f16, rabitq4}`.
- **Insert** (`insert.rs`): append the new row's compact payload to a sidecar tail
  block (allocate a new block when full), keyed by its heap TID. (Insert order is
  not globally sorted; see "merge-join vs lookup" below.)
- **Vacuum/delete** (`vacuum.rs`): when a posting entry is tombstoned for a dead
  heap TID, mark the matching sidecar entry dead (a deleted bitmap per sidecar
  block, mirroring the posting deleted-bitmap), so rerank skips it. Full rebuild
  on REINDEX regenerates the chain from scratch (sorted).

### Merge-join vs per-candidate lookup (insert ordering caveat)

Build writes the sidecar globally tid-sorted, but inserts append at the tail
(locally unsorted). To keep rerank correct and fast without re-sorting on every
insert, the rerank stage will:

1. Load the sidecar chain into a scan-local `HashMap<heap_tid, &payload>` (or a
   sorted `Vec` + binary search) once per scan, bounded by the probed frontier —
   **only the survivor heap TIDs are looked up**, so we can instead do a single
   tid-sorted pass over the sidecar collecting only payloads whose heap TID is in
   the (small) survivor set.

I will pick the cheaper of {survivor-set filter during one sidecar pass} vs
{per-candidate lookup} in 003b and bench both via the byte counter; the survivor
set is bounded by `rerank_width`, so a single filtered pass is the likely winner.
**Flagging this as the one open implementation choice for the reviewer to weigh
in on if they have a preference.**

## Byte-accounting (the win evidence, AC for 003b)

Add an explain counter `stats_rerank_source_bytes_read` to `IvfExplainCounters`:

- heap_f32 / f32 path: accumulates `dimensions * 4` per reranked candidate (full
  f32 source fetched from the heap), as today.
- f16 sidecar: accumulates `dimensions * 2` per reranked candidate.
- rabitq4 sidecar: accumulates `rabitq4_payload_len` per reranked candidate.

The 003b pg_test win-evidence fixture asserts
`rerank_source_bytes_read(f16) < rerank_source_bytes_read(f32)` (≈0.5×) and
`rerank_source_bytes_read(rabitq4) < rerank_source_bytes_read(f32)` on the same
corpus/query — the byte reduction the reviewer asked to be proven by counter,
without needing `ecaz bench suite`.

## Constraints honored

- Coarse stage, dense posting layout `0x25`/`0x28`, coalescing, quant math:
  untouched (Option B touches only the rerank read path + a new `0x2A` chain).
- `heap_f32` / `rerank_format=f32` results stay bit-identical (no sidecar on that
  path).
- No stripped dead formats reintroduced (`0x26`/`0x27`/`0x29`/page-scatter).
- No new SIMD: scoring still goes through the existing `RerankScorer` +
  `candidate_batch` scorers; the sidecar only changes the *source* of the bytes
  fed to them.
- `rerank_placement='index'` stays rejected.

## Validation done in this checkpoint

- `cargo check --no-default-features --features pg18` on the base — passes
  (`artifacts/cargo-check-baseline.log`). No code change in 003a; this confirms
  the branch tip builds before 003b begins.

## Ask of the reviewer

1. Bless **Option B** (sidecar keyed by heap TID, tag `0x2A`, metadata head
   pointer) over Option A (rerank_tid-through-candidate), and confirm the
   **B1 metadata format-version bump (v2 → v3, +6 bytes, additive,
   backward-readable)** — the metadata page is full so a head pointer needs it.
2. Confirm the byte-reduction-by-counter evidence approach (no `ecaz bench
   suite`) satisfies the 003 win-evidence gate for the *code* slice (the full
   suite remains the Phase-3 / packet-002 gate on a provisioned lane).
3. Flag any preference on the merge-join-pass vs per-candidate-lookup rerank read.

003b will implement Option B end-to-end (build/insert/vacuum/scan + counter +
pg_test fixtures) on top of this sign-off.
