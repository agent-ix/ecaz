## Feedback: ADR-030 v2 Selective Exact Traversal Budget

Read this packet alongside 357, since 357 supersedes this packet's
headline claim with verified counters. This review is therefore
partially about what the packet did well and partially about why
its budget=1 parity claim did not survive the next packet.

### What's right

- **Right batching of structural surface and experiment.** Adding
  scope (`all` / `layer0`) and limit (per-expansion exact budget) in
  one packet means the budget family can be swept cleanly in one
  env-matrix restart cycle. Good ergonomics.
- **`scripts/restart_adr030_scratch.sh` is the right response to the
  env-sprawl concern from 355 feedback.** Having one repo-local
  restart wrapper that prints the resolved ADR-030 env surface before
  startup would have caught packet 352's build-gate miss in seconds.
  This is the first packet in the sequence that treats the scratch
  workflow itself as a measurement surface rather than ad-hoc shell.
  Overdue but right.
- **Format verification up front.** Every measurement in the packet
  is anchored on lines 114-116:
  `emitted_result_count = 40`, `grouped_result_count = 40`,
  `compared_result_count = 40`. The 354-era verification habit is
  now embedded in the template. That matters more than any single
  measurement.
- **layer0-only exact traversal probed as a cheaper variant.** Even
  though it didn't carry — recall dropped from 0.878 (all-layer
  exact) to 0.868 at 50k and from 0.938 to 0.906 at 10k — probing it
  was right. The top-layer exact contribution is nontrivial on this
  corpus, and the negative result here is what directs the next
  packet's search. Useful evidence, not wasted work.
- **Scope restriction enum is enforced, not informational.** Per
  `scan.rs:880`, `grouped_exact_traversal_enabled_for_layer(mode,
  layer)` is the single decision point. Scalar paths remain inert.
  Right shape.

### Concerns

1. **The budget=1 parity claim is the headline, and it is wrong.**
   Lines 158-163 report budget=1 matching all-layer exact traversal
   at 50k (`0.8780`) and 10k (`0.9380`). Packet 357's direct rerun
   of the same operating point with the new hot-path counters shows
   budget=1 50k at `0.4900` — collapsed recall, not parity. The
   357 framing ("Treat that earlier measurement claim as superseded
   by this packet") is the right posture for the branch history, but
   it also raises a methodology question: how did this packet get a
   0.8780 result in the first place?

   The most likely explanation, given the scratch-env discipline
   issues called out in 354: the 50k measurement in this packet was
   probably taken while the exact traversal gate was still enabled
   at `scope=all`, `limit=NULL` (i.e., full exact), and the env
   variable for budget=1 was either not live or silently ignored due
   to a cache/restart ordering issue. Without a settings audit at
   measurement time, this is unprovable — exactly the gap 357's
   `tests.tqhnsw_debug_adr030_runtime_settings()` fills. 357's first
   concrete value-add beyond counters was making "which envs were
   actually live when this number was produced" a queryable fact.

   For process going forward: before trusting any measurement claim
   in a grouped packet, capture the runtime settings probe output
   alongside the numbers. Both go in the request.md under the
   corresponding table.

2. **Budget selection order is "grouped approximate score first,
   exact-rescore the best N".** That's stated in the Planned Slice,
   but the code at `score_budgeted_grouped_traversal_candidates` at
   `scan.rs:1740-1754` picks the top-N by approximate score, then
   exact-rescores. Once 357 showed that the grouped approximate score
   is not trustworthy for candidate selection (Spearman correlation
   with exact is ~0.17 per packet 357's approximate profile table),
   picking the "best" N by that bad signal is itself part of why
   budget=1 collapsed. The algorithm is coherent but the input
   ordering is the problem, not the budget size. That makes budget=4
   recoverable (more coverage) but budget=1 particularly fragile.

3. **`--exact-limit` is named after a purely numeric concept but is
   now gated to exact-scope=all by the code.** (Confirmed via
   `resolve_grouped_exact_traversal_limit` behavior gated by the
   disabled-mode check at line 726-730.) Fine for now, but the env
   surface would read better if the limit only had effect when
   `scope=all` and the user got a clear error for incompatible
   combinations, same as 358's frontier_head requiring scope=layer0.
   Small nit.

### Observation

The negative result at `layer0`-only and the subsequently-debunked
budget=1 result together pointed at a structural insight: cheap
exact-like seams around the existing grouped approximate scorer
don't work, because the approximate scorer is already feeding the
wrong candidates into whatever exact-rescoring budget picks up.
That's the insight that 358 (frontier-head) then confirmed and 359
(binary mode) acted on. So the packet is useful even though its
headline claim was wrong — it closes a specific family of "cheap
exact rescue" approaches.

### Measurement gap still open

- the "budget=1 parity" in this packet is closed with a negative
  result by 357; keep 357 as authoritative.
- the question "can we make exact-like traversal cheap?" is
  effectively closed too, per 358. The answer is "not without a
  better candidate-selection signal upstream of exact rescoring."

### Scratch-wrapper feedback for future packets

The restart wrapper is a good surface. Add one thing: after the
restart completes and before exiting, have the wrapper emit the
verified settings probe output (`SELECT * FROM
tests.tqhnsw_debug_adr030_runtime_settings()`) to stdout and to a
file. That way every scratch run has a mechanical record of what
envs were actually live, and measurements taken after it can cite
the audit file. Would prevent a repeat of the 352 / this-packet
measurement-vs-claim drift.
