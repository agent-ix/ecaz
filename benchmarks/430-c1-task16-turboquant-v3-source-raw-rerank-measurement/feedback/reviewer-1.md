## Feedback: V3 source-raw rerank measurement — ACCEPTED, with rollout flag

Verified against:

- commit `fd42eb5` adding packet artifacts
- packed-bytea column path through existing
  `tests.tqhnsw_debug_pack_f32_bytea`
- cited artifacts including same-table `source` vs `source_raw`
  rerank-profile CSVs from
  `tests.tqhnsw_debug_grouped_rerank_profile(...)`

### What's right

- **Same-table control run.** The most valuable piece of this
  packet is the decision to measure `source` and `source_raw` on
  the *same* rebuilt index, isolating the column-swap effect from
  the rebuild effect. Without that control, the `429 → 430` delta
  (`6.086 → 4.568ms`) would have read as "source_raw is a ~25%
  win" when the fair same-table comparison is `6.57%`. This packet
  gets both numbers, names them as different, and explains which
  one supports the productization argument.
- **Detailed rerank helper used, not just the coarse stage
  profile.** The `heap_fetch`/`heap_decode`/`heap_dot` split
  (`-69us` / `-136us` / `+2us` deltas) pins *where* the win comes
  from: heap fetch and decode, not dot-product math. That is
  actionable and correctly constrains the next packet's hypothesis.
- **Stale-TID failure reported as a real rollout constraint.**
  The initial post-backfill run throwing `could not fetch heap
  tuple at (4274,3)` is not a benchmark-tool bug — it is a real
  invariant that matters for any user deploying
  `rerank_source_column`. Naming this in the readout (not burying
  it) is the right call.
- **Recall preserved.** `0.9629` recall@10 / `0` score error /
  `0` exact-gap queries matches packet `429` exactly, so the
  source column swap is not hiding a correctness regression behind
  a latency number.

### Concerns

1. **Stale-TID recovery is undocumented beyond "REINDEX."** The
   packet names the failure and the fix, but does not answer the
   question a merge reviewer will ask: *is this a supported user
   workflow, or a footgun*? Options include:
   - require `REINDEX` after any heap rewrite touching the rerank
     source column (current behavior, documented)
   - detect the failure and raise a clearer user-facing error
   - add an explicit maintenance helper
   Packet `431` acknowledges the constraint as follow-on; please
   pick one of these before closing task 16, or explicitly defer
   it into its own tracked task. See questions in packet `432`
   feedback.
2. **`same-table source` rebuilt at `4.889ms` vs packet `429`'s
   `6.086ms` on a nominally similar rebuilt index.** That `~20%`
   shift in the baseline is not explained. Could be query cache
   warming, could be cross-run scratch state. Not a blocker for
   this packet's claim, but it is the first visible symptom of the
   measurement-noise problem that became dominant in packet `432`.
3. **Env-override surface used in this packet is measurement-only
   (good), but the readout treats it as the shipping interface for
   a moment.** The "§2 next justified implementation" framing
   (durable raw-f32 source column) is correct; the packet could
   have been more explicit that the env override is not the
   product.

### Call

Accepted. This is the first packet on the arc that identifies a
current-head lever that genuinely helps the serious lane, and it
does the measurement carefully enough to separate the column effect
from the rebuild effect. The stale-TID constraint is important and
is flagged — packet `431` picks up the productization.
