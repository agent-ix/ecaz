---
type: ADR
id: ADR-080
title: "IVF Rerank Sidecar Directory Must Be Two-Level for Fat Payloads"
status: PROPOSED
impact: Refines ADR-079; affects the IVF rerank sidecar directory format (two-level for fat f16/rabitq4 payloads), FR-032, and NFR-001.
date: 2026-06-19
---
# ADR-080: IVF rerank sidecar directory must be two-level for fat payloads

Status: Proposed (2026-06-19)
Context tasks: 111g (index-side rerank placement); follow-up to
[ADR-079](ADR-079-ivf-rerank-sidecar-directory.md); benchmark packet
`benchmarks/ivf-111g-115-attribution/` Finding 8.

## Context

ADR-079 added a `(first_tid → block_tid)` directory over the `0x2A` rerank
sidecar chain so an index-side rerank reads only the survivor blocks instead of
the whole chain. It predicted index-side latency would fall "from ~540 ms to ≤
table-side." The re-bench (Finding 8, release `.so` `6078da14`, real DBpedia
1536-dim @100k, fresh build, no inserts) only **partially** confirmed that:

| index-side @100k | recall np64 / np200 | p50 np8 / np64 / np200 | vs ADR-079 pre-fix |
|---|---|---|---|
| rabitq4 | 0.917 / 0.942 | 7.7 / 9.6 / 16.0 ms | **fixed** — scales, ≈ table-side |
| f16     | 0.964 / 0.9975 | 146.8 / 150.2 / 159.2 ms | 3.6× faster, still slow |

- **rabitq4 fully realized ADR-079's thesis.** ✅
- **f16 did not:** it dropped 540 → ~150 ms but stayed flat across nprobe and
  far above table-side f32 (13 ms). The directed survivor read is bounded; the
  residual flat cost is something else.

## Root cause (grounded in code)

`load_rerank_sidecar_payloads_directed` (`src/am/ec_ivf/scan.rs:2613`) **fully
materializes the directory on every query** before binary-searching it:

```rust
let mut next = directory_head;
while next != ItemPointer::INVALID {
    let block = read_ivf_rerank_sidecar_block(index_relation, next)?;
    for (i, first_tid) in block.heap_tids.iter().enumerate() {
        let block_tid = ItemPointer::decode(...)?;
        dir.push((*first_tid, block_tid));   // every directory entry, every query
    }
    next = block.next_tid;
}
```

So the per-query floor is **O(number of sidecar blocks)** = O(N / entries-per-
sidecar-block), independent of survivor count or nprobe — exactly the flat ~150 ms
signature. The directory has one entry per *sidecar block*, and sidecar block
density is payload-size dependent:

| rep | payload_len | entries / 8 KB sidecar block | sidecar blocks @100k | directory entries | directory blocks walked/query |
|---|---|---|---|---|---|
| rabitq4 | ~64 B | ~120 | ~840 | ~840 | ~1–2 (negligible) |
| f16 | 3072 B | ~2 | ~50,000 | ~50,000 | ~74 + 50k decodes |

For f16 the directory is ~N/2 entries — the directed read cured the survivor
pass but the **directory walk became the new O(N) term**. rabitq4's compact
payload keeps the directory tiny, which is why it is already fast.

## Decision

Make the directory access **sub-linear** so the per-query cost is independent of N
for any payload size. Two-level (sparse) directory:

### On-disk (research project — clean format break, just rebuild)

- Keep the existing leaf directory chain (`first_tid → block_tid`, sorted by
  `first_tid`) unchanged.
- Add a **sparse top index**: one entry per *directory block*
  (`first_tid_of_that_directory_block → directory_block_tid`), sorted by
  `first_tid`. For f16 @100k that is ~74 entries → a single ~1 KB top block.
- Persist a new `rerank_sidecar_directory_top_head: ItemPointer` in IVF metadata.
  This is another metadata-width change → bump `EC_IVF_INDEX_FORMAT_VERSION`
  4 → 5 per `NFR-016` (same discipline as the v3→v4 bump), with a fresh
  `ivf_metadata_v5.hex` fixture and v4-rejected-by-version test.

### Scan

Replace the full-chain directory walk with: read the top index (1 block) →
`partition_point` to the directory block(s) covering `[min_survivor,
max_survivor]` → read only those directory blocks (≈1–2) → binary-search within →
read survivor sidecar blocks (already bounded). Total ≈ 3–4 block reads
regardless of N. The `directory_top_head == INVALID` (and `inserted_since_build
> 0`) cases fall back to the existing full-chain walk, so correctness never
depends on the top index being present.

### Build

After writing the leaf directory chain, make a second pass over the leaf
directory blocks emitting one top-index entry per block; link the top chain and
record its head in metadata. Build stays single-pass-per-level and O(N).

## Expected result

f16 index-side p50 drops from flat ~150 ms toward the survivor-bounded floor
(survivor block reads + ~3 directory reads), scaling with nprobe like rabitq4.
**Must be confirmed by re-benching `sidecar-index` f16 at 10/50/100k before any
promotion** — this codebase's predictions frequently fall flat, and ADR-079's own
f16 prediction already did.

## Consequences

- Index-side f16 becomes non-pathological. It still carries the fat 416 MiB index
  (16× table-side) and is **unlikely to beat table-side f32** (13 ms, recall
  0.999, 25 MiB lean index), which remains the measured best default. This ADR
  makes index-side f16 *correct and not-slow*, not *winning*; the high-value
  index-side option (rabitq4: fast + compact, ~0.94 recall) is already realized by
  ADR-079. Priority is accordingly **low** — sequence it behind higher-value IVF
  work.
- Generalizes: the same two-level shape keeps the directory cheap for any future
  fat rerank payload (e.g. an f32 sidecar).

## Alternatives considered

- **Per-backend cache of the materialized directory** (keyed by
  `relfilenode + directory_head + nblocks`): amortizes the O(N) walk across warm
  queries in a session (benches issue many queries, so it would collapse the f16
  number after the first query) and is far less code than a new format. But it is
  still O(N) cold, costs backend memory, and needs invalidation on insert — a
  workaround, not a fix. Reasonable as a *stopgap* if the format bump is not
  wanted yet.
- **Co-locate the compact rerank payload inside the dense posting block** (read
  during the coarse scan, zero extra fetch): best long-term, but a larger
  posting-format redesign — deferred (same disposition as in ADR-079).
- **Shrink directory entry width** (pack `first_tid`/`block_tid`): a constant
  factor, does not change the O(N) directory walk. Rejected.
