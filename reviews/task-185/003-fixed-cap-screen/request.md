---
task: 185
packet: 003-fixed-cap-screen
role: coder
status: open
date: 2026-08-07
seq: 01
head: 2ffe120a5
---

# Task 185 fixed-cap screen checkpoint

This packet records the preregistered 100k control and isolated returned-seed
attribution slice. It does not claim that a gateway candidate has been
selected or that the fixed-cap A/B screen is complete.

## Entry condition

Packet 002 established a disjoint 100k training control and exact truth join,
but also showed that the current origin trace attributes only the effective
bounded traversal: under BW4/H100, returned seed positions 1--4 expanded and
positions 5--32 did not. That trace cannot score arbitrary 4,096-member head
landmarks. The implementation checkpoint for this packet must therefore add a
candidate-level isolated attribution surface, or a deterministic simulation
proved byte-equivalent to it, before constructing a gateway selector.

Implementation checkpoint `917995972` adds the first narrow surface: a
benchmark-only endpoint that reruns the physical scan with exactly one member
of the control's returned seed list. This isolates per-candidate expansion and
truth-hit behavior from the 32-seed beam. It intentionally does not claim to
score arbitrary head members, alter production selection, or constitute the
gateway selector. Suite-driver wiring was extended at `2ffe120a5` with a
bounded isolated-position limit. The alternate-head candidate pool remains
before the A/B run.

## Frozen screen

After that surface is reviewed, compare exactly these three 100k arms first:

1. Task 182/185 `training_landmarks_exact` control;
2. one gateway set-cover selector ranking candidates by marginal bounded-
   traversal truth coverage; and
3. one diversity-aware returned-seed selector that penalizes shared traversal
   basins.

All arms keep cap 4,096, exact scoring of the persisted head, 32 returned
seeds, graph degree 32, BW4/H100, RaBitQ traversal, exact final ranking, the
three-owner physical topology, and the same corpus and query identities.
Training construction may use only rows 201--400 and their exact training
truth. Rows 1--200 remain held out until the evaluation A/B is fixed.

## Decision rule

Select by held-out distinct recall first, then overlapping-CI warm p50, cached
bytes, and construction time. Training truth coverage is diagnostic and cannot
break an evaluation tie. Advance at most one fixed-cap candidate. If neither
candidate improves held-out recall without a material measured regression,
close the task with STOP and carry the limitation to Task 186/conditional
follow-up.

## Required evidence

The eventual run must be driven only by `ecaz bench suite`, with a checked-in
SuiteConfig and packet-local artifacts. The screen must report, for every arm,
recall and CI, warm latency distribution, storage/cache bytes, construction
time, head and seed digests, disjoint training provenance, topology and remote
engagement, and unanimous release provenance. This first packet-003 screen is
100k diagnostic evidence; only a useful winner may proceed to the required
10k/50k/100k confirmation packet 004.

No production default, persisted format, graph, traversal, materialization,
or release configuration changes are authorized by this preregistration.

## Captured control and isolated attribution

The valid 100k suite completed successfully through `ecaz bench suite`:

- release-profile preflight passed on all three owners with unanimous
  extension SHA `57ee20b5da9df0d5efe1a922a12808ab62ad52e9`;
- physical topology and remote-owner engagement passed (`owners=3`,
  `remote_verified=2`, `source_rows=100000`);
- control recall@32 was `0.9205`, CI `[0.9078, 0.9316]`, with 200 queries and
  2,000 distinct top-10 trials;
- warm latency was `43.00 ms` after 10 warmups; physical generation was
  `2,496,659,456` bytes, with construction `955,502 ms` and publication
  `1,094,062 ms`;
- isolated attribution completed 800 reruns (positions 1--4 × 200 training
  queries), with exact training truth union coverage `1,833/2,000` and ordered
  marginal coverage `[1,577, 138, 84, 34]`.

The detailed result and provenance artifacts are listed in
`artifacts/manifest.md`. The isolated endpoint still covers only members of
the control's returned seed list; it is not arbitrary-head candidate scoring
and no selector or production change is claimed.

## Validation of the implementation checkpoint

- PG18 feature build with `distann-head-attribution-benchmark`: pass.
- PG18 featureless build: pass.
- Suite audit and dry-run: pass; the emitted command includes
  `--gateway-isolated-trace` and the bounded 200×4 training-slice isolated
  matrix. The frozen production/A-B contract remains 32 returned seeds.
- Current release preflight: pass on three nodes with unanimous SHA
  `57ee20b5da9df0d5efe1a922a12808ab62ad52e9`.
- 100k control plus isolated attribution suite: pass, exit code 0; packet-local
  recall, latency, storage, membership, prediction, trace, and normalized
  result artifacts captured.
- Native 10k smoke: stopped before measurement because its 200-row query file
  cannot provide the disjoint rows 201--400 training slice; no 10k result is
  claimed.

The packet-local build logs, suite config, preflight log, and manifest are the
durable validation evidence.
