## Feedback: Scratch Explicit-Format Runtime Matrix

Read the packet's eight result tables, the four readout sections,
and the planner-honesty note at the bottom.

### What's right

- **First full `corpus × format × m × ef` matrix on the scratch
  cluster.** Every prior landing packet (`405`, `407`) measured a
  single operating cell. This packet finally stands up the full
  grid on one runtime lane, so "pq_fastscan wins at the serious
  operating point" becomes a claim you can see the shape of,
  not a point estimate.
- **One runtime lane for every row.** `binary` traversal +
  `heap_f32` rerank + `build_source_column` + scratch-cluster
  only. Named up-front and then every row shares it. That is the
  only way an 8-cell matrix can be compared cross-cell honestly.
- **Scratch cluster only.** Addresses packet `405`'s open concern
  directly — the run does not lean on `~/.pgrx`.
- **The planner honesty at the bottom is the best part.** Packet
  could have reported a shared-table SQL matrix and quietly hidden
  the planner cross-choosing between sibling indexes. Instead, it
  names the problem out loud and declines to overclaim. That is
  exactly the right posture for landing-proof work.
- **Reads out correctly: recall gain is not free.** "pq_fastscan
  wins recall but loses latency on direct runtime" is a stronger
  and more honest landing story than "pq_fastscan is better."
  The `50k, m=16, ef=128` cell (`0.9635 @ 7.0ms` vs `0.9342 @
  5.3ms`) is the one every future review will cite.

### Concerns

1. **`exact_quantized Recall@10 = 0.8301` on `50k` and `0.9660`
   on `10k`.** That is a big gap between "what the exact quantized
   path can reach" and "what the graph reached" at larger corpus.
   The `pq_fastscan m=16 ef=200` cell at `0.9671` actually
   *exceeds* `0.8301`, which means the heap-f32 rerank is
   recovering recall that the quantized path would have lost.
   That is the actual load-bearing insight in the matrix — the
   packet reads it as "pq_fastscan wins recall" but the deeper
   story is "heap_f32 rerank recovers quantization loss at scale."
   Worth naming explicitly because it affects the next design
   conversation (do we ever want `heap_f32` on `turboquant`?).
2. **No `queries` column in the matrix.** Packet `407` measured
   at `queries=1000`; this packet doesn't name the count. If it
   was also `queries=1000`, say so — otherwise a reader
   comparing cross-packet has to guess, and the whole recall
   discussion has already been bitten by `queries=50` vs `=1000`
   confounds once.
3. **No standard error on the recall numbers.** At `n=1000` a
   Recall@10 of `0.9635` has a std-err around `0.006`, so the
   `0.9635` vs `0.9342` gap at `50k, m=16, ef=128` is real, but
   fine-grained comparisons like `0.9465` vs `0.9455` at
   `10k, m=8` between `ef=160` and `ef=200` are within noise.
   One sentence on measurement uncertainty would keep a future
   reader from reading too much into adjacent cells.
4. **`pq_fastscan` latency wall at `~4.6ms` on `10k, m=8` at
   `ef=128` vs `turboquant` `1.9ms`.** That is a 2.4x latency
   multiplier at the small-corpus default. If the landing
   argument is "pq_fastscan is the higher-recall default," the
   small-corpus latency cost is the thing that will dominate the
   first operator's first impression. Worth calling out explicitly
   that the recall case is stronger at `50k+` than at `10k`.

### Observation

Best landing-evidence packet on the arc so far. The full matrix
plus the planner-honesty note about why the SQL matrix is not
here yet is exactly what the `405` feedback asked for. Pair this
with packet `414`'s clean isolated SQL matrix and the landing
evidence bar is now complete on runtime-surface terms.
