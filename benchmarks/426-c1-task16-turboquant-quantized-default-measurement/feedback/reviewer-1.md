## Feedback: Quantized-default measurement — ACCEPTED, with an honest decision point

Verified against:

- commit `c9a90bf` adding the packet artifacts
- cited artifacts
  (`tmp/task16-turboquant-quantized-default-m16only.summary`,
  `...-stageprofile.csv`, and matching recall summaries)
- isolated one-index-per-table surface matches packets `423`/`425`

### What's right

- **Measurement packet with no code changes.** Correct scoping —
  the policy diff is packet `425`, the proof is packet `426`. This
  separation makes both packets individually reviewable and
  re-runnable.
- **Uses the same isolated surface as packets `423` and `425`.**
  That keeps the `5.046ms → 2.958ms` delta apples-to-apples. Cross-
  packet comparisons on one-index-per-table lanes have been a
  recurring weak spot; this packet gets it right.
- **Recall *and* latency reported.** Not just "it got faster."
  Directly names `0.9251` (quantized) vs `0.9629` (heap-f32) at
  recall@10 on the same surface, which is exactly the tradeoff a
  default-policy measurement needs to show.
- **Internal stage profile corroborates.** `~41` exact-score calls
  per query versus packet `423`'s `~1605` is a load-bearing number:
  it proves the deferred-rerank shape actually landed, not just a
  timing improvement.
- **§Decision is honest.** "Levers 1 and 2 are not enough" is a
  conclusion many measurement packets avoid saying out loud; this
  one does. Calls out lever 3 as the justified next step before
  running it.

### Concerns

1. **`5.046 → 2.958` vs `pq_fastscan`'s `4.26` reads cleanly at
   glance but mixes comparison axes.** The `pq_fastscan` number was
   captured under a different cell in task 15; worth a sentence
   saying "on an isolated rebuilt surface, not a re-measurement of
   the `413`/`414` cells."
2. **Recall gap (`0.9251` vs `0.9629`) is reported but not
   quantified against the serious-lane recall target.** A reader who
   hasn't read the task-16 doc cannot tell from this packet whether
   `0.9251` misses the target by a little or a lot. A single line
   like "serious lane targets ≥ `0.96` recall@10" would close that.
3. **`graph_below_exact_queries = 14` / `worst_exact_gap = 1` on
   the quantized lane.** Not obviously a blocker, but these are the
   closest "something drifted" flags on the quantized recall output,
   and the packet doesn't interpret them. Worth a one-liner either
   way.

### Call

Accepted. This is the decision packet that correctly identified the
recall-preserving lane as the remaining bottleneck and licensed
lever 3 as justified next work. The honest "it's a tradeoff, not a
win" framing is the right shape.
