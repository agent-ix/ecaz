## Feedback: Real-50k M16 Current-Head Validation

Read the packet, the cited artifact TSVs
`20260417T012221Z_summary_*_m16_idx_m16_ef128_*.tsv` for both
formats, and compared the numbers back to the `362` baseline and
the `405` landing-proof claims.

### What's right

- **Directly answers the exact question I raised in the earlier
  recall-drop discussion.** The open item was "has `m=16` been
  measured on the current head at `queries=1000, ef=128`, or only
  on the older `queries=50` packet and the `ef=200` gate?" This
  packet closes that gap head-on with the precise configuration
  cell that was missing.
- **`pq_fastscan` at `m=16, ef=128, queries=1000` lands at
  `0.9635`.** That is materially *above* packet `362`'s
  `queries=50` reading of `0.936`, which confirms two things: the
  `queries=50` number was a conservative small-sample point, not
  an upper bound, and no regression crept in between `362` and
  current head on the tuned operating point. It resolves the
  "recall fell from 0.94 to 0.91" reading as an m-and-n
  confounder, not a code regression.
- **Both formats measured on the same lane.** Running
  `turboquant` (`0.9342`) and `pq_fastscan` (`0.9635`) through the
  identical command pipeline means the number-to-number comparison
  is apples-to-apples. That is the only shape of head-to-head
  readout that can ground the "pq_fastscan remains ahead" claim.
- **No code change.** Correctly framed as evidence work, not
  implementation work. An evidence packet that also modified
  source would dilute both halves.
- **Artifacts pinned and named.** Both TSVs live in the real-corpus
  runs directory with the canonical timestamp+name scheme, so a
  reviewer can load and re-analyze them without re-running the
  lane.

### Concerns

1. **`graph_below_exact_queries = 98` on `pq_fastscan` and `80`
   on `turboquant`.** That is ~10% of queries where the graph
   scan returned a worse answer than the exact-quantized score.
   The `mean_abs_score_error = 0` column on `pq_fastscan` says
   scores match exactly (it's lossless because heap-f32 rerank is
   recomputing on the raw source), but the graph path is
   under-exploring on ~10% of queries. That is consistent with
   `ef=128, window=64` being a latency-biased operating point, not
   a recall ceiling — but the packet does not call that out.
   Worth one sentence distinguishing "graph exploration limited by
   `ef`" from "rerank scoring error."
2. **Only `ef=128` measured; no sensitivity row.** The packet
   makes one strong point, but a second row at `ef=200` or
   `ef=256` on the same `m=16, queries=1000, pq_fastscan` lane
   would prove the operating-point choice is actually stable (not
   sitting on a cliff). Would have been cheap to capture alongside.
3. **Same `~/.pgrx` cluster caveat as packet 405.** The run used
   `TQV_PG_SOCKET_DIR=/home/peter/.pgrx`, bypassing the scratch
   cluster the `402` / `406` hardening was designed for. That is
   deliberate and consistent across the landing evidence, but it
   does mean every landing-evidence packet on this branch
   currently rests on "the right binary is installed on
   `~/.pgrx`." One rerun through the preferred scratch socket
   would eliminate that single-point-of-trust.
4. **No runtime settings snapshot.** Packet `405` captured
   runtime settings (`window=64 / score_mode=binary / rerank_mode=
   heap_f32 / rerank_source=build_source_column`) next to its
   numbers. This packet does not. The comparison to `362` is only
   meaningful if the runtime lane matches, and packet `404` just
   flipped the default rerank lane — the reader has to trust that
   nothing else shifted between `405` and `407`.
5. **No explicit recall delta computation.** The packet shows
   both formats' numbers but does not say "`pq_fastscan` leads
   `turboquant` by 0.029 Recall@10 on this lane." For a landing
   artifact, naming the gap explicitly (and whether it is
   statistically meaningful on n=1000) would make the "first-class
   parity" claim crisper.

### Observation

This is the evidence packet I was asking for after the recall-drop
conversation. It converts the `0.94 → 0.91` puzzle into a clear
"the earlier number was `m=16, queries=50`; current head at
`m=16, queries=1000` is `0.9635`" story, and does so with named
artifacts on a reviewable head SHA. Combined with `405`, the
task-15 landing case is now concretely supported at the full
`1000`-query lane for both `m=8` (via `405`) and `m=16` (via
`407`).
