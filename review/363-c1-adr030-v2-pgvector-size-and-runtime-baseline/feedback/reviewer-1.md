## Feedback: ADR-030 v2 pgvector Size And Runtime Baseline

Read the install wrapper at `scripts/install_pgvector_pg17_scratch.sh`,
the size-comparison table at lines 99-106, and the direct-runtime
pgvector vs grouped-m16 comparison at lines 122-139.

### What's right

- **External baseline is overdue and exactly right to do now.**
  The branch has been producing tqvector-vs-tqvector comparisons
  for the last 30 packets. A pgvector baseline on the same
  corpus is what any reader outside this branch will want
  first. It's also what tells you whether "grouped-v2 beats
  scalar tqvector" is a real product story or just internal
  tuning. Doing it at the point where grouped-v2 finally has a
  stable deterministic-build operating point is the right
  timing.
- **Install wrapper is narrow and reproducible.** A shell
  script that builds a local pgvector checkout against the
  existing pg17 scratch tree using
  `PG_CONFIG=/home/peter/.pgrx/17.9/pgrx-install/bin/pg_config`.
  No download, no network dependency, the local checkout is a
  declared input. That's the right shape for a scratch-only
  build helper — the same pattern as
  `install_adr030_pg17_pg_test.sh` from 359. Worth carrying
  forward as the template for any future extension install
  scripts.
- **Size comparison is the headline finding.** `391 MB` vs
  `65 MB` is a ~6x size win against pgvector on this corpus.
  This is the first time the branch has a hard number for the
  ADR-030 size claim — prior "~9 tuples per page vs ~1 for
  pgvector" language in the repo was an architectural
  expectation; this is empirical confirmation. The 6x ratio is
  roughly the expected outcome given 1536-float vs 4-bit
  encoding with per-group metadata overhead — so it validates
  the on-disk design, not just the claim.
- **Both pgvector m=8 and m=16 indexes are identical at 391
  MB.** That's expected — pgvector stores the full f32 vector
  payload regardless of m, so m only changes the graph-edge
  overhead which is small relative to the vector payload. The
  comparison table gets this right by listing both sizes even
  though they're identical; that lets a reader see the invariant
  explicitly rather than wondering if a number was omitted.
- **Same-source-vector corpus.** `select id, source::vector(1536)
  as embedding from tqhnsw_real_50k_corpus` uses the same raw
  source vectors as the tqvector corpus. No divergence at the
  data layer. Good.
- **Honest readout on the runtime trade-off.** Lines 142-155:
  tqvector grouped-v2 is smaller and faster; pgvector is more
  accurate. The packet doesn't attempt to frame "0.940 vs 0.998
  Recall@10" as anything other than what it is — pgvector's
  recall is structurally higher on this corpus at this density
  because it has the full f32 vector signal to work with.
- **SQL-planner caveat surfaced.** Lines 157-172 flag that the
  plain tqvector SQL path wouldn't pick the isolated grouped
  index even with `enable_seqscan = off`. That's the same
  observation packet 360 made about the canonical shared-table
  case; naming it again here keeps the measurement-surface
  limitation visible, and it sets up packet 364's proper SQL
  harness.

### Concerns

1. **0.940 vs 0.998 Recall@10 at ef=128 is a 5.8pt gap — not
   small.** pgvector's recall at ef=40 (0.986) already beats
   grouped-v2's ceiling at ef=320 (0.946). For a user with
   recall requirements at or above 0.95, grouped-v2 can't
   compete on this corpus. That's worth naming directly. The
   packet's interpretation ("that is a real trade-off, not a
   clean win") is accurate, but I'd go further: at
   high-recall-requirement operating points, the trade-off
   isn't a trade-off — it's pgvector being the only option.
   The product framing is therefore "grouped-v2 is viable
   where recall target is ≤0.94." That's a non-trivial
   constraint to bake into the ADR.

2. **tqvector grouped m=16 direct-runtime numbers here differ
   from packet 362.** Packet 362 reported grouped m=16
   ef=128: 0.936 @ 2.445ms. This packet reports grouped m=16
   ef=128: 0.940 @ 2.227ms. Close but not identical, on the
   same corpus and same m. Two plausible explanations:
   - the isolated grouped-only corpus (363) is a different
     table from the shared canonical corpus (362), and the
     row iteration order under build may differ
   - measurement noise from different runs
   At 361's deterministic-build level, the same build state
   should produce the same graph; different *tables* can
   legitimately produce different graphs. The `0.940 vs 0.936`
   delta is small enough to plausibly be noise, but worth a
   sentence in the packet explaining whether the
   isolated-corpus grouped build is a different graph from the
   shared-canonical grouped build, or just a different
   measurement run on the same graph.

3. **pgvector `ef=64` latency (2.384ms) is lower than `ef=40`
   (2.434ms).** Line 125-126. That's an inversion — higher ef
   shouldn't be faster in a well-behaved search. Likely
   measurement noise at 50 queries, but worth flagging because
   it signals the per-query variance on this lane is on the
   order of the ef=40-to-ef=64 latency difference. The
   grouped-v2 lane shows the same flattening at 0.9200/0.9380
   between ef=40 and ef=64 with a modest latency increase,
   which is more internally consistent. Might want more than
   50 queries for a publishable comparison.

4. **pgvector HNSW at ef=40 reaches 0.986 Recall@10.** That's
   remarkable and worth contextualizing. A Recall@10 of 0.986
   at ef=40 means on average 9.86 of the top-10 exact matches
   are recovered in a 40-candidate search pool. That's within
   0.012 of the effective ceiling (no ANN can exceed exact =
   1.000). So pgvector is essentially at ceiling even at its
   smallest measured ef. Two downstream implications:
   - pgvector has *no headroom* in this measurement — ef
     doesn't meaningfully help because the answer is already
     there
   - the right comparison is "tqvector grouped-v2 at
     target-recall-X vs pgvector at target-recall-X" rather
     than "same ef"
   A same-recall-target table would show grouped-v2's latency
   advantage more cleanly than a same-ef table, at least at
   the recall levels where both can reach the target.

5. **Isolated-table grouped-v2 ef=320 (0.946@5.693ms)
   outperforms the canonical m=16 grouped ef=320
   (0.938@6.132ms) from 362.** Both are from freshly rebuilt
   m=16 deterministic grouped graphs on the same source data.
   The isolated lane is faster *and* more accurate. At face
   value this repeats the surprise from 360 ("isolated tables
   are stronger than canonical shared-tables on grouped") —
   which 361 was meant to fix. The 361 fix was about the
   per-point random layer assignment; the remaining
   iteration-order variance from concern #2 could still
   produce slightly different graphs on different tables even
   with the same seed logic.
   Worth investigating: does
   `tqhnsw_real_50k_grouped_m16_idx` vs
   `scratch_tqhnsw_real_50k_grouped_m16only_idx` produce
   bit-identical graph structure on the same row data? If not,
   there's a residual non-determinism the 361 fix didn't
   close. If yes, the recall delta is build-surface-unrelated
   and something else explains 362 vs 363 divergence.

6. **pgvector HNSW build parameters aren't logged.** The
   packet says `ef_construction=128` for both lanes, but
   doesn't report pgvector's other build-time parameters
   (`m_l`, `multi-build`, etc., whichever defaults are in
   effect). For a baseline comparison, pgvector's build-side
   settings should be recorded in the packet so a future
   reader can reproduce — particularly if pgvector adds
   configurable build parameters in later versions.

### Observation

The size win (6x) is the paper-ready finding. The runtime
story is more nuanced — "faster at lower-recall operating
points but can't reach pgvector's recall ceiling" is the
accurate summary, and that's a product-level decision, not a
tuning problem.

There's also a meta-lesson for the branch: the branch has
been drifting toward "we need to match pgvector on all axes"
as the implicit ship criterion. With real numbers in front of
us, that's probably the wrong framing — pgvector stores 6x
the data so of course it has more information per query. The
right framing is "tqvector grouped-v2 at the same storage
budget as pgvector" — which would mean comparing grouped-v2
against a pgvector *half-precision* (vector(1536,fp16) via
pgvector 0.8's `halfvec`) or some other pgvector storage
reduction. At that storage ratio, the recall gap is probably
smaller, possibly inverted. Not required in this packet, but
worth keeping in mind as the branch converges on ADR-030's
ship criterion.

### Measurement gap still open

- **SQL-path latency comparison.** Addressed by packet 364.
- **Same-recall-target comparison table.** Pulling recall
  target columns instead of ef columns would change the
  qualitative read — see concern #4.
- **pgvector with storage-reducing features enabled** (halfvec
  or quantized-pgvector lane, if available in 0.8.2). That's
  the actual apples-to-apples same-storage comparison the
  branch eventually wants.
- **Recall@k for k > 10.** grouped-v2's candidate-selection
  weakness may be worse at top-10 than at top-100 — binary
  sign is a coarse signal that may recover many-of-100 better
  than exactly-top-10. Not urgent, but relevant to product use
  cases that ask for larger k.
