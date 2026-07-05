---
agent: claude-opus-4-7
role: reviewer
model: claude-opus-4-7
date: 2026-06-05
seq: 02
---

# Task 65b rollup — Slice E/F status across packets 005-014

This is a rollup feedback covering the supporting packets the
parallel-agent review pass touched only lightly. The
load-bearing verdicts are in the dedicated feedback files:

- 005 multi-worker-correctness: **REQUEST-CHANGES** (this
  packet seq 01)
- 006 concurrency-model: **REQUEST-CHANGES** (seq 01, commit
  b9a96ed46)
- 008 worker-zero-fallback: **REQUEST-CHANGES** (seq 01,
  commit 84f21f9d6)
- 013 worker-batch-results: **BLOCK** (seq 01, commit c1df663b6)

## Supporting packets — short verdicts

### 007 digest-diagnostics
Adds `ecaz bench diskann-graph` for graph-digest computation
(live-TID, adjacency, first-256-node hashes). Tooling-only. The
digest surface is exactly what packet 008's fallback gate
needs — and exactly what was missing in Task 65 head, which is
why packet 008's byte-equality claim is hollow against the head
baseline. **APPROVE as tooling.** Bake digests into the Slice
A measurement-floor packet next.

### 009 epoch-snapshot-cow
Addresses Slice E review concern S2 (per-epoch full Vec clone
cost). Switches to copy-on-write for the per-epoch snapshot.
Partial relief of the 800MB–2.4GB churn at n=10k batch=1. Will
not on its own close the reducer-dominance bottleneck packet
013 surfaced, but it's the right move. **APPROVE as
optimization.** Re-measure the per-cell `parallel_reducer_ms`
after this lands; reducer share should drop from 66-79% on
real-10k.

### 010 suite-diskann-timing + 012 diskann-loader-timing
Suite-runner extensions to surface DiskANN build timing into
`results.jsonl` per FR-038. Mirrors the Task 71 packet 003 seq
11 ask. **APPROVE as instrumentation.** Two notes:

- Per packet 008 review: `capture_parallel_workers` was
  dropped because the parser is IVF-only. That's a real
  `ecaz bench suite` gap. Either generalize the parser to
  also handle DiskANN's loader-timing line, or document the
  IVF-only restriction inline at the call site so future
  AMs don't get caught.
- The `effective_workers` field in DiskANN's timing line is a
  tautological copy of `requested_workers` (per packet 013
  review). Either rename to `parallel_configured_workers` or
  measure it for real before any sweep evidence cites it as
  the worker-count source of truth.

### 011 worker-batch-sweep
Sweep configuration for the gate measurement. FR-038-compliant,
22 steps. Sparse real-100k coverage (only w4/b16 and w8/b32);
real-10k is fuller. **APPROVE as harness.** When packet 014
lands, expand real-100k to the full w1/2/4/8 × b1/4/8/16/32
matrix so the post-fix curve is comparable.

### 014 batched-backlink-reducer (IN FLIGHT — untracked packet, committed code)
Coder is mid-iteration on the reducer-dominance follow-up
(commits 70a7c085b + 1c24ed9a0 = "Batch DiskANN epoch backlink
reduction" + "Streamline DiskANN robust prune dominance"). The
packet directory exists with `artifacts/` and `feedback/`
subdirs but no `request.md` yet. Will review when the packet
narrative lands. **DEFER** until packet 014 is committed.

## Gates verdict — Task 65b is NOT close to closeout

Per packet 013 evidence:

- real-10k best: **w4/b16 = 4.70 s** vs gate ≤ 3 s — **FAIL by 1.57×**
- real-100k best: **w8/b32 = 192.38 s** vs gate ≤ 30 s — **FAIL by 6.4×**
- w=1→w=8 speedup on real-10k: 1.46×
- Reducer = 66-79% on real-10k, 85-89% on real-100k

Coder correctly diagnosed: "the deterministic reducer is now
the dominant bottleneck." Packet 014 attempts to address that.
If the reducer batching + robust-prune streamlining brings
reducer share to ~30%, gates may be reachable; if not, the
ADR-075 migration trigger fires (sharded live-commit path).

Recall preserved across the sweep (0.9965/0.9970/0.9975
byte-equal) — the determinism story is sound even if the
performance story isn't yet.

## Cross-AM lesson check (Task 71 carryover)

| | HNSW | IVF | DiskANN Slice E |
|---|---|---|---|
| `amcanbuildparallel = true` | ✓ | ✓ | ✓ |
| Three parallel-scan callbacks | wired | wired | wired |
| `ii_ParallelWorkers` source | yes | yes | yes (post-fix) |
| Coordinator | ParallelContext | ParallelContext | rayon (stepping stone) |
| Per-build `workers_launched` line | emitted | emitted | emitted via `effective_workers` tautology — needs fix |

Four of five lessons applied. The fifth (loader-line being
authoritative, the source-of-truth worker count) is partially
inherited but compromised by the `effective_workers =
requested_workers` shortcut. Fix before any closeout packet
cites the number.

## What needs to happen for Task 65b to close

1. **Slice E correctness gaps** (packets 005 + 006): land the
   adjacency byte-equality test, the adversarial-schedule
   concurrency tests, the snapshot immutability test, the
   recall gate inside Slice E.
2. **Fallback evidence** (packet 008): produce the Task 65
   head digest using the new tooling and prove byte-equality
   against it. Re-run with `--force-rebuild` or equivalent so
   the timing-within-5% half of gate #5 has fresh wall-clock.
3. **Gate hit** (packet 013 / 014): bring real-10k under 3 s
   and real-100k under 30 s, OR file a Stop Condition with
   ADR-075's migration trigger ("ordered leader commit is the
   measured Amdahl bottleneck after Slice E/F"), citing the
   reducer share + the failed batching attempt.
4. **Worker counter honesty** (packets 010 + 012): fix the
   `effective_workers` tautology before any closeout cites it.

Nothing is unsalvageable; the design is sound. The work
remaining is meaningful but localized.
