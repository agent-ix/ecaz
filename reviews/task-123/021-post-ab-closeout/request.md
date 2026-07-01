# Task 123 Status Sync: Post-A/B No-Promote Closeout

This packet records the closeout of the reopened Task 123 multi-instance
core-algorithm scope. It adds no new run. It implements the reviewer acceptance
of the packet 020 closeout request:

`reviews/task-123/020-post-ab-closeout-request/feedback/2026-06-30-01-reviewer.md`

## What Closes

The 2026-06-28 reopened multi-instance latency + communications scope for Task
123 is now closed as **no-promote / re-scope**. The evidence chain:

- Packet `017-multinode-communications-prune-ab` — accepted as the
  communications datapoint. `id,source` ships ~73.9 MB of heap payload over three
  remotes for 200 queries vs ~48 KB for `id` (~1540×), while latency is flat
  within each surface. Transport payload bytes are **not** the dominant local
  core-path latency driver.
- Packet `018-dedupe-prune-threshold` — code LGTM. The dedupe-aware fix (commit
  `d2ffbdaa9`) makes `pre_materialization_min_ip_to_keep()` engage under
  `VecIdDedupeEnabled`; exact-score logic is recall-safe.
- Packet `019-dedupe-prune-multinode-ab` — the engaged-guard b2/b4 A/B.
  recall@10 = 1.0000 with prune on **and** off across n1024/b2/nprobe64 and
  n128/b4/nprobe96; prune-on/off latency is flat.
- Packet `020-post-ab-closeout-request` — reviewer accepted the no-promote
  closeout, conditional on the wording correction carried below.

## Final Verdict (corrected wording, per packet 020 review)

The dedupe-aware pre-materialization prune threshold (`d2ffbdaa9`) is retained
as a **correctness / plumbing** fix. In the representative b2/b4 multi-instance
matrix it is **recall-safe** (recall@10 = 1.0000 with prune on) and
**latency-neutral** on the coordinator path.

Its actual leaf-side engagement (rows pruned) was **not** captured: packet 019
does not surface `truncated_candidate_row_count`, and every coordinator-side
structural counter (`candidate_sum`, `payload_rows_sum`, `payload_bytes_sum`,
`remote_heap_candidate_sum`, `merge_input_sum`, `merge_output_sum`) is
byte-identical prune-on vs prune-off. So packet 019 cannot distinguish an
engaged-but-ineffective prune from a still-inert one. The prune is therefore
**not a demonstrated latency lever and is not promoted**. Whether it dropped any
rows at these configs is unmeasured.

## Shipped State: Prune Default Flipped Off

Consistent with the no-promote conclusion, `ec_spire.pre_materialization_prune`
now defaults to **off** (`src/am/ec_spire/options/mod.rs`). The feature merges as
dark, opt-in plumbing; a session must explicitly set
`ec_spire.pre_materialization_prune=on` to exercise it. This keeps main's default
read behavior identical to pre-merge and defers any promotion to a measured
result under Task 131. (Unit tests are unaffected: the `#[cfg(test)]`
`pre_materialization_prune_enabled()` returns `true` regardless of the GUC
default.)

## Explicit Non-Claims

This closeout does **not** claim:

- true cross-network performance;
- realistic payload transport cost;
- a pre-materialization prune latency win;
- measured prune engagement (rows dropped);
- a default SPIRE promotion.

## Follow-up → Task 131

Optimization follow-up moves to newer SPIRE tasks, primarily
`plan/tasks/131-spire-streaming-global-topk-pruning.md`:

- engagement-instrumented prune — surface `truncated_candidate_row_count` (or a
  remote leaf-scan scored-vs-materialized delta) in the bench output and re-run
  on/off so drops are visible;
- an off-disk clean-latency rerun for any absolute-latency verdict (the n128
  ~5.1–5.2 s regime is disk-constrained);
- recall-safety of the prune at configs/scales where it actually engages heavily
  (Task 112/113 loose-bound / k-cap precedent), untested where engagement is
  non-trivial.

## Requested Decision

Confirm the status-sync: Task 123 closed as no-promote / re-scope, prune default
off, follow-up routed to Task 131. This implements the already-granted packet 020
acceptance; no new measurement is required.
