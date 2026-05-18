## Feedback: ADR-030 v2 Deterministic Grouped Graph Build

Read `deterministic_hnsw_build_seed(...)` at `build.rs:1352`, the
`splitmix64(...)` finalizer at `build.rs:1372`, the call sites at
`build.rs:1248` and `1306`, and the vendored `new_with_seed(...)`
constructors in `vendor/hnsw_rs/src/hnsw.rs`.

### What's right

- **Root cause is actually fixed, not worked around.** Upstream
  `hnsw_rs` calls `StdRng::from_os_rng()` for per-point level
  assignment. That means every HNSW build produces a different graph
  even on identical inputs. Vendoring and adding `new_with_seed`
  constructors lets the database owner control the seed — exactly
  what's needed for a reindexable, reproducible graph. Right fix.
- **Seed derivation mixes the right state.** At `build.rs:1352-1369`:
  base seed, dimensions, bits, m, ef_construction, tuple_count, and
  a domain tag distinguishing scalar-code vs source-vector builds.
  Each factor is multiplied by a distinct splitmix64 constant before
  XOR-combining, then run through a full splitmix64 round. That's a
  sound mixer — no correlated bits across inputs, avalanche property
  from the finalizer.
- **Separate domain tags for scalar vs source builds.** The two call
  sites use `0x5343_414c_4152_5f31` ("SCALAR_1") and
  `0x534f_5552_4345_5f31` ("SOURCE_1") as literal domain tags. That
  means the scalar-code and source-vector HNSW builds produce
  different graphs on the same input, which is correct — they have
  different edge-quality characteristics and should not collide. The
  tag ordinal `_1` implies a versioning strategy for future build
  domains, which is right-sized forethought.
- **`new_with_seed(...)` constructors are additive.** The existing
  `new(...)` constructors still work by delegating to
  `new_with_seed(..., rng.random())`. That means any callers outside
  tqvector (if the vendored fork is ever rebased) aren't broken.
- **`PartialEq, Eq` on `HnswBuildNode` + repeated-build tests.**
  Unit tests that build twice from identical state and
  `assert_eq!(graph_a, graph_b)` are the only way to verify
  determinism mechanically. Doing it for both scalar and source
  builds catches regressions in either domain tag.
- **Validation on real corpus.** Two fresh grouped builds
  `det_a` / `det_b` produce identical recall curves (table at
  lines 156-158). That's end-to-end proof that the seed plumbing
  reaches the right levers, not just unit-test proof of the mixer.
- **Canonical grouped frontier after reindex beats scalar on both
  axes.** Lines 172-175: grouped `ef=128: 0.910 @ 1.601ms` vs scalar
  `ef=128: 0.890 @ 3.202ms`. That is the first packet in the branch
  where grouped-v2 strictly dominates scalar on a canonical surface
  at a non-trivial recall point. All prior packets had grouped
  winning on latency but losing on recall, or vice versa. This is
  the actual ship threshold.

### Concerns

1. **tuple_count in the seed mix is a correctness trade.** Mixing
   `tuple_count` into the seed (line 1361) means: reindex on the
   same table with the same rows produces the same graph, good. But
   if rows are added, removed, or deleted between reindex runs, the
   seed changes, which means the graph is structurally different
   even if the new rows had identical embeddings to the removed
   ones. For a table with churn, this is the right behavior — you
   want a "fresh" graph, not a superficially-reproducible one tied
   to a specific build-time snapshot. But it also means "same input
   produces same output" is a stronger claim than what's actually
   true; the more precise claim is "same input *at the same tuple
   count* produces the same output."

   Worth naming in the commit message or an inline comment at the
   seed construction, so a future reader doesn't treat the
   determinism guarantee as content-only.

2. **`ADR030 should explicitly depend on deterministic graph builds`
   from the Risk/Follow-up list (line 200-202) is the right call.**
   The ceiling move from 0.874 (non-deterministic) to 0.914
   (deterministic) is 4 points of recall *purely from removing
   build lottery*. That's larger than many of the traversal-score
   improvements the earlier packets were measuring, and it retroactively
   invalidates parts of every packet's grouped recall claim from
   354 through 360. Not invalidates-as-in-wrong, invalidates-as-in-
   measured-against-different-graphs-each-run. This should be added
   to the ADR itself, not just listed as follow-up.

3. **The vendored fork is now a maintenance commitment.** Vendoring
   `hnsw_rs` means future upstream improvements don't land
   automatically — someone has to rebase the vendor directory
   periodically or live with the frozen version. For a dependency
   with a well-defined API that's critical to the database's
   correctness, that's fine, but it's worth a one-line CHANGELOG or
   README entry in `vendor/hnsw_rs` saying:
   - why it's vendored (seed control for determinism)
   - what the upstream commit is as of this vendoring
   - who to talk to before rebasing

   Otherwise six months from now the next maintainer has to
   reconstruct this context from git history.

4. **Same linker-block on the required checkpoint.** `cargo test`
   and `cargo pgrx test pg17` still fail locally per lines 113-121.
   The clippy and `cargo check` checkpoints pass, but the
   `repeated builds return identical graph-node output` unit tests
   — which are the load-bearing correctness proof for this packet —
   didn't run in the required checkpoint. Three packets running on
   `cargo check` + clippy + manual scratch measurements is a real
   gap. I'd prioritize fixing this before the next merge-worthy
   packet.

5. **Open question about reproducibility across architectures.**
   `splitmix64` output is architecture-independent at the bit level
   given the same u64 inputs. But one hidden source of variance on
   the input side: if the build inserts tuples in hash-map iteration
   order, and the hash map's hasher is architecture-sensitive (hash
   seed from env, SIMD-accelerated hash on some archs, etc.), two
   machines with the same `BuildState` could produce different
   per-insert order, which means different `HnswBuildNode` outputs
   despite identical layer seeds. Worth one sentence in the
   followup about "we tested rebuild determinism on the same
   machine; cross-machine determinism is a stronger claim that
   would need a cross-arch test." Might not matter for the
   immediate use case, but it's a predictable future question.

6. **Ceiling still not scalar.** Grouped ef=128 at 0.910, scalar at
   0.890 here, but scalar at `m=16` later (packet 362) reaches
   0.950. So "grouped dominates scalar" in this packet is at
   `m=8` only, and the story at `m=16` remains "grouped is faster
   but trails on recall." The ceiling moved significantly; it
   didn't cross scalar globally. Worth a one-line reminder that
   this is an `m=8` result and the `m=16` conversation is separate
   (which 362 picks up).

### Observation

This is the most *surprising* packet in the recent arc. The branch
had been spending weeks iterating on traversal scoring on the
assumption that the graph structure was stable across builds. It
wasn't. Once I see the `StdRng::from_os_rng()` mechanism, the
symptoms line up:
- packet 360's isolated-build-vs-canonical-build recall gap
- packet 352's original wildly-varying 50k numbers (which at
  the time I attributed to cold-cache noise)
- the general "hard to get consistent recall readings on grouped
  indexes" that was hedged into caveats in every measurement
  packet

A lot of earlier reviewer feedback asking about "cold-cache noise"
or "variance on N=10 queries" was actually noise from a different
source — graph lottery. The right meta-lesson: when measurements
of what should be a stable system drift, suspect non-determinism
in the build or load path *before* suspecting measurement noise.
The debug-scan-windowed-summary verification habit from 354 caught
on-disk format drift; something analogous was needed for graph
structural stability.

### Measurement gap still open

- cross-machine reproducibility (concern #5). Not urgent.
- whether deterministic build affects concurrency — if concurrent
  builds of two indexes on the same table run, the seeds differ
  (different base seeds from the metadata), but neither is
  dependent on runtime state that could make them collide. I don't
  see a concurrency concern, but it's worth explicit verification.
- the 1k and 10k build lanes in staging. Follow-up point #1 from
  the packet's Risk section. Should land before any size/latency
  claims at those scales.

### Final note

The deterministic build is a durable runtime contract change.
Treat it that way in the next ADR revision — write it down as
"the grouped-v2 build gate implies deterministic-layer-assignment
semantics" rather than as a build-side implementation detail.
Future reviewers who look at the ADR without the branch history
will otherwise have to reconstruct this lesson.
