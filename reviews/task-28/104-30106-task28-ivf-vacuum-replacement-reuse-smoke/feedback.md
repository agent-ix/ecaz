---
reviewer: opus (main-conversation)
date: 2026-04-28
head: d54e1f40
scope: cumulative review of A1–A10 progress on `task28-ivf` against the merge gate in `plan/tasks/28-ivf-competitive-substrate.md`
---

# Task 28 IVF coder-1 progress review

This review reads the code at head `d54e1f40` and packets 30076–30106 against the
A1–A10 merge gate. It is anchored at packet 30106 because that is the latest
artifact, but the findings are cumulative.

## Merge-gate status

| item | status | evidence | notes |
|---|---|---|---|
| A1 cost-model audit | landed | 30076 | constants now have microbench-backed values; audit complete |
| A2 streaming vacuum | code landed, scale evidence missing | 30079 + `vacuum.rs:198`, `page::rewrite_ivf_postings_for_list_blocks` | task wording requires nlists ∈ {8,32,64} on ≥1M rows. Packet 30079 admits this measurement is still owed. Not closed. |
| A3 vacuum compaction | partial | 30080 + 30103/30104/30105/30106 | tuple-level reclaim + range-reuse insert exists; nlists=8 same-distribution refill fully reused, but nlists=32/64 still grew (30106). Index-size convergence under sustained churn is **not** demonstrated. |
| A4 typed score-mode dispatch | landed | 30102 | string compare is gone; typed `ExactScoreMode` match in place |
| A5 cache-key audit | landed | 30102 | `pg_test_ec_ivf_rescan_reuses_cached_prod_quantizer` asserts cache identity |
| A6 planner cross-test matrix | landed | 30077 | mixed-predicate shape is a real planner-quality concern but the packet calls it out |
| A7 score-bound prune | **not landed** | 30078 | trial was negative; A7 remains open. Task plan says A7 must land before merge. |
| A8 wire PQ-FastScan + RaBitQ | landed | 30081, 30082 | both variants build/scan/insert/vacuum; see naming concern below |
| A9 100k+ scale measurement | **not landed** | n/a | no packet records the matched-shape ec_ivf vs ec_hnsw 100k/1M sweep with build time, index size, recall@10/100, p50/p95/p99, memory hwm, cold/warm cache. The 100k PQ packets (30090–30095) are PQ-FastScan-internal sweeps, not the A9 substrate measurement. |
| A10 head-to-head | partial | 30084, 30096, 30097 | recommendation captured (PQ-FastScan g8 dominates at 100k; TurboQuant retains 10k/25k recall@100); recall@100, memory hwm, and cold/warm cache state still missing. Task plan also requires re-run after A7 lands. |

The branch is **not merge-ready**. A7 and A9 are the hard gates; A2 and A3 are
soft gates whose code has landed but whose claimed acceptance criteria
("bounded by O(page_size)", "index size on a churn workload should track live
tuple count") are not yet evidenced.

## Code findings (must fix before merge)

### F1. Dead `live_head_block` / `live_tail_block` fields in `ListBulkDeleteResult`

`src/am/ec_ivf/vacuum.rs:14-37` still maintains `live_head_block` and
`live_tail_block` and `record_live_posting` writes them on every live posting.
Commit `746a8eea` switched the head/tail repair to use the original
`directory.head_block` / `directory.tail_block` whenever live tids remain. The
two fields are now write-only. Either:

- delete the fields and the head/tail-tracking branches in `record_live_posting`
  (preferred — they are dead weight in the per-posting hot loop), or
- justify keeping them with a comment if a near-future change will read them.

This is a post-746a8eea cleanup that was missed.

### F2. Range-reuse insert is O(range_length) per row in worst case

`page::append_ivf_posting_to_list_range` (`src/am/ec_ivf/page.rs:1414`) tries
the tail block, then iterates `(head_block..tail_block).rev()`, opening,
exclusive-locking, and probing free space on every block in the list range
until it finds one that fits. Under high churn this is a per-insert O(range)
walk. The 30106 result that nlists=32/64 still grew on a same-distribution
fixture is consistent with this shape: when the list range is small but pages
are interleaved with directory or cross-list tuples, the walk fails to find
fitting space and falls through to `P_NEW`.

This is the right v1 shape for correctness, but call it out explicitly in a
comment and add a follow-on packet that profiles the walk under sustained
insert load. If the walk dominates insert time on the 1536D harness, a
free-space metadata sidecar (FSM-like, per list) is the real fix.

### F3. Empty-list range is forgotten on first vacuum

`run_bulkdelete` resets `repaired_head` / `repaired_tail` to
`BlockRef::INVALID` whenever `live_heap_tids == 0`. The pages that previously
held that list's postings become unreachable from any directory range, so the
range-reuse insert path cannot recover them — only `P_NEW` allocations after
relation growth can. This is consistent with the 30106 nlists=32/64 refill
growth: lists that emptied and then refilled cannot reuse their old pages.

This is the structural reason A3's "index size tracks live tuple count" claim
is not yet defensible. Either:

- preserve the list's old block range across the empty state (mark the
  directory entry with a recoverable range even when `live_heap_tids = 0`), or
- add an index-level free-block list (small, persisted) that any list's insert
  path consults before allocating.

The first option is the smaller change and stays inside the existing list/
directory shape.

### F4. A8 reloption naming diverges from the task spec literally, and the
divergence is undocumented

The task plan A8 specifies the reloption is `quantizer` and the enum is
`IvfQuantizerProfile`. The code uses `storage_format` as the reloption name
(parsed in `src/am/ec_ivf/options.rs:46`, exposed via the `c"storage_format"`
column at `options.rs:325`). The internal enum is `IvfQuantizerProfile`, which
is fine, but the user-visible reloption name does not match the gate text.
Either:

- rename to `quantizer` to match the spec, or
- update the spec to record that the reloption is `storage_format` and the
  rename was intentional (`storage_format` is more accurate — it controls
  payload shape, not just scoring).

I suspect the second is what you want, but it is not recorded anywhere and a
strict reading of the merge gate fails. Pick one and commit it.

### F5. A2 acceptance language is stronger than the evidence

Packet 30079 admits the 1M-scale wall-time and peak-memory measurement is
still owed. The task plan says "Acceptance: vacuum peak memory bounded by
O(page_size), not O(list_size)." The streaming primitive is in place
(`page::rewrite_ivf_postings_for_list_blocks` walks one block at a time under
exclusive lock), so the *code* satisfies the bound, but the gate explicitly
asks for a measurement packet at nlists ∈ {8, 32, 64} on ≥1M rows. Either run
the measurement, or amend the task wording to say "code-only acceptance
because the streaming primitive is structural." I recommend the measurement
— it also stresses A3 and the F2/F3 reuse path together.

## Recommendations on remaining work

Sequencing the unfinished items:

1. **F1** is a 5-line cleanup; do it next regardless of priority — it is
   visible dead state inside the vacuum hot loop.
2. **F3** is the biggest single A3 unlock. Without it, refill-after-empty
   never reuses pages, and 30106's nlists=32/64 result will keep showing up.
3. **A2 measurement** at nlists ∈ {8,32,64} on ≥1M rows. This also gives F2
   a profile to argue from.
4. **A7 second attempt**. The 30078 lesson is "don't add per-query prepared
   state on the dev path." A reasonable next attempt: do the bound check
   inside the existing LUT scan loop using gamma + a posting-local
   pre-summary already computed by encode, so prepare cost is unchanged. If
   that is still negative on the n64/w25 surface, the right fallback is a
   posting-layout change (cluster postings by gamma so a per-cluster gamma
   max prunes a whole micro-batch) — but that is its own task and likely a
   follow-on, not this gate.
5. **A9** at 100k and 1M, after A2 measurement and at least one A7 attempt
   land. Run ec_hnsw on the same column for the matrix.
6. **A10 closure** on the 10k/25k/100k matrix with PQ-FastScan g8, including
   recall@100, memory high-water, cold/warm cache. The 30097 refresh has
   most of this for 10k/25k; combine with 30091/30094/30095 and add the
   missing memory + cache-state columns.

## What is already strong

- The streaming vacuum primitive in `page::rewrite_ivf_postings_for_list_blocks`
  is the right shape — one block at a time, exclusive lock, callback-driven
  rewrite/keep/delete decision. The `Delete` variant correctly uses
  `PageIndexTupleDeleteNoCompact` to keep line pointer numbers stable; the
  comment in 30080 about why a compacting delete broke directory TIDs is
  exactly the kind of decision-history that earns its keep.
- The A4/A5 typed-dispatch cleanup is small, complete, and well-tested
  (`pg_test_ec_ivf_rescan_reuses_cached_prod_quantizer` is the right shape
  for asserting cache identity).
- The A1 cost-model packet does the right thing: microbenchmarks the kernels
  and grounds the constants in measured cost rather than planner-selection
  tuning. The mixed-predicate honesty in 30076 / 30077 is good.
- The 30097 matched-width A10 refresh is honest about TurboQuant's recall@100
  win on 10k/25k and refuses to over-generalize the 100k PQ-FastScan win. That
  is the tone the A10 close-out should keep.

## Verification commands run for this review

- `git log --oneline -40`
- `git show 746a8eea d54e1f40 2c1196c2`
- `git diff 591c10ad..HEAD -- src/am/ec_ivf/`
- read `src/am/ec_ivf/vacuum.rs`, `options.rs`, `page.rs:1411-1500`,
  `quantizer.rs` profile dispatch
- read packets 30076, 30077, 30078, 30079, 30080, 30081, 30082, 30084,
  30096, 30097, 30102, 30103, 30104, 30105, 30106
