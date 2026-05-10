# ec_diskann M5 NEON Follow-Up — Build A/B + Reviewer Suggestions 2 / 3

Reviewer: this packet closes out the three forward-looking
suggestions from `30207-01-reviewer.md`. Suggestion 1 turned into
the largest Apple-Silicon-specific effect measured in this round.
Suggestions 2 and 3 are explicitly deferred with the same
cold-cache prerequisite recorded.

## Headline — Suggestion 1 is a real Apple-Silicon win

The reviewer flagged that the existing `dceda05` NEON kernel is
also called from `ambuild::source_inner_product_distance`, and
that build does many more exact dot products than rerank, so the
build-time effect should be larger than the query-time effect.
Measured, back-to-back, on the same `m5_diskann_real10k` real-data
fixture and reloptions:

| build | code | elapsed |
|---|---|---:|
| `m5_diskann_real10k_scalar_build` | `e5f380a1` (scalar) | `32.61 s` |
| `m5_diskann_real10k_neon_build` | NEON | `6.74 s` |
| `m5_diskann_real10k_neon_build_2` | NEON | `6.89 s` |

NEON mean `6.81 s` vs scalar `32.61 s` => **`4.79x` speedup,
`-79.1%` build elapsed**. Both NEON passes are within `0.15 s` of
each other; the gap is well outside any plausible system-noise
band. Recall@10 / NDCG@10 are bit-identical between the two arms
(`0.9965 / 0.9970 / 0.9975` and `0.9999` across `L = 64 / 200 / 800`),
so the build-time speedup is not a quality-vs-speed trade.

This is roughly an order of magnitude larger than the rerank-time
effect on the kernel-stress lane in `30205` (`-1.8 ms` p50, `-11%`)
and is delivered by the same `dceda05` kernel that is already on
the branch. No new code change is required — the win is already
delivered by the existing committed NEON specialization once the
branch is installed.

## Suggestion 2 — same-page-run grouping: investigated, deferred

The reviewer's framing was: "group rerank fetches by heap block
and consume same-page runs while the page is already hot/pinned.
That is more promising than the synchronous prefetch shape that
was reverted."

Investigated by reading the existing fetch path. The current
rerank loop calls `table_tuple_fetch_row_version` per row via
`scan_state::fetch_heap_row_version`. Implementing
"hold-pin-across-same-block" would require bypassing
`table_tuple_fetch_row_version` and rolling a manual fetch path:

- `ReadBufferExtended` + `LockBuffer(BUFFER_LOCK_SHARE)` once per page,
- `PageGetItemId` + `PageGetItem` per row,
- `HeapTupleSatisfiesVisibility(snapshot, ...)` per row,
- HOT-chain follow via `heap_hot_search_buffer` where applicable,
- `LockBuffer(BUFFER_LOCK_UNLOCK)` + `ReleaseBuffer` once per page,
- direct varlena access (or `slot_getsomeattrs` on a freshly
  populated slot) for the IP kernel.

Structurally that is a significantly bigger change than the
heap-TID sort that landed in `30205`, AND the warm-cache savings
are bounded above by what `table_tuple_fetch_row_version`
currently spends on buffer pin / unpin against an
already-cached page (one shared-buffer table lookup plus an
atomic pin-counter increment / decrement per row). At
`rerank_budget=800` and roughly half the rerank rows sharing a
block on the heap-TID-sorted batch, that ceiling is on the order
of `~30 us` saved per query, inside the per-pass `0.5 ms`
stddev already seen in `30205`.

The non-trivial benefit shows up only on **cold cache**, where
the saved work is not "pin / unpin" but "do not re-issue a TOAST
or heap-page read." That is the same regime that already gates
the deferred cold-cache prefetch revisit in suggestion 3.

Recommendation: do not implement same-page-run grouping until a
cold-cache harness exists. At that point, both this and the
async-overlapping prefetch from `30206` should be evaluated
together against the cold-cache numbers, since they are competing
candidates for the same Apple-Silicon I/O hypothesis.

## Suggestion 3 — cold-cache prefetch revisit: deferred (already recorded)

Acknowledged; the suggestion itself is a deferral guidance.
Packets `30206` and `30207` already record the cold-cache
prerequisite for the prefetch revisit. This packet adds
suggestion 2 to the same blocked-on-cold-cache list, so when a
cold-cache harness lands the next round can evaluate three
candidates (prefetch, async-overlap prefetch, same-page-run
grouping) against one set of cold-cache numbers instead of
chasing them one at a time.

## Closing the loop

After this packet the reviewer's three suggestions are all
addressed:

| suggestion | disposition |
|---|---|
| 1. measure build-time NEON impact | **promoted** as a `4.79x` build-time win on this fixture |
| 2. same-page-run grouping | investigated, deferred with cold-cache prerequisite |
| 3. cold-cache prefetch revisit | deferred per reviewer's own framing |

The branch tip stays at `45a666c7` plus this packet. No new code
checkpoint; suggestion 1 needed only measurement, and suggestions
2 / 3 are explicit deferrals.

## Artifacts

All artifacts under `artifacts/`. See `artifacts/manifest.md` for
SHAs, commands, and per-pass tables. The recall correctness rerun
reuses the truth cache from
`review/30204-task29-diskann-m5-neon-rerank/artifacts/truth_real10k_k10.json`.
