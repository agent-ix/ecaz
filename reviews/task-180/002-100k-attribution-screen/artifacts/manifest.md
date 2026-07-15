# Task 180 packet 002 artifact manifest

This manifest covers the benchmark-surface implementation checkpoint, its
compact PG18 smoke, and the first Phase 1 100k attribution screen. The smoke
proves that every diagnostic path executes; its two-query/two-iteration
measurements are **not** Phase 1 decision evidence. `screen-a` is decision-grade
for the production/owner/exact-sample and width axes.

## Provenance

- Owning task / packet: `task-180` / `reviews/task-180/002-100k-attribution-screen/`
- Implementation commit under test: `174da94efc82d1f0ea4d11751eb8a834f4d8c29f`
- Seed-count diagnostic follow-ups: `7c63fb124174ad44e2148c25f05df4419444ea8b`
  and `2dbd78450f4f94b7c48c60554b1f0db646ff8fe6` (feature-only scored-candidate retention plus an invariant
  that rejects a benchmark request when fewer seeds are actually returned)
- Installed extension SHA/profile: `174da94efc82d1f0ea4d11751eb8a834f4d8c29f` / `release`
- Nodes: three local PG18 physical owners; one index per source table; exact
  hash-shard topology with no replicated/shared-table measurement surface
- Corpus: `ec_real_10k` from `/home/peter/dev/ecaz/data/staged-current`
- Query SHA-256: `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`
- Storage format / traversal baseline: persisted RaBitQ neighbor codes; no OPQ
- Fixed search shape: graph degree 32, head cap 4096, BW4/H100, top-k 10

## Checked-in configs

### `implementation-smoke-suite.json`

- Command: `target/release/ecaz bench suite run --config reviews/task-180/002-100k-attribution-screen/artifacts/implementation-smoke-suite.json`
- Timestamp: 2026-07-14 (America/Los_Angeles)
- Purpose: live PG18 execution of `persisted_head`, `head_sample_exact`,
  `owner_scan`, and `exact_neighbor` against one immutable 10k generation.
- Workload: 2 held-out queries / 20 membership trials and 2 measured latency
  iterations after 1 warmup; validation only.

### `screen-a-suite.json`

- Command: `target/release/ecaz bench suite run --config reviews/task-180/002-100k-attribution-screen/artifacts/screen-a-suite.json`
- Timestamp: 2026-07-14 22:07-22:54 PDT.
- Status: succeeded in 2,574,997 ms; status reports one completed step, no
  failures, missing artifacts, or stale artifacts; all eight thresholds pass.
- Registered matrix: 100k production, owner oracle, exact bounded sample, and
  persisted-head widths 32/64/128/256, all returning 32 seeds, with 200 held-out
  queries / 2,000 membership trials and 50 latency iterations after 10 warmups.
- Runner commit: `29c5920308e9d8fb477c6297922c9d67b086eb9a`.
- Installed extension SHA/profile: `174da94ef...` / `release`, unanimous across
  all three nodes.

### `screen-b-seeds-suite.json`

- Command: `target/release/ecaz bench suite run --config reviews/task-180/002-100k-attribution-screen/artifacts/screen-b-seeds-suite.json`
- Timestamp: 2026-07-14 23:18-23:50 PDT.
- Status: succeeded in 1,885,299 ms; one completed step, no failures, missing
  artifacts, or stale artifacts; all five thresholds pass.
- Registered matrix: cap 4096, selected width 64, returned seed counts 32/64/128
  against one immutable 100k build. The feature-only implementation retains
  additional candidates already scored by the width-64 search for the
  128-seed arm and errors if the requested count is not actually returned.
- Installed extension SHA/profile: `53b62bbea7ce4be1bd8053daf504801f09b36352`
  / `release`, unanimous across all three nodes. The extension code change is
  `2dbd78450...`; the later branch-head commit contains review evidence only.

### `screen-c-caps-suite.json`

- Command: `target/release/ecaz bench suite run --config reviews/task-180/002-100k-attribution-screen/artifacts/screen-c-caps-suite.json`
- Timestamps: cap 8192, 2026-07-14 23:56 through 2026-07-15 00:29 PDT;
  cap 16384, 2026-07-15 00:31-01:11 PDT.
- Status: both selected steps succeeded with no failures/missing/stale artifacts
  and all step-local topology, provenance, and recall-row thresholds passing.
  To stay within the host's disk capacity, the same checked-in suite was run
  one `--only` step at a time with separate manifests/results; each stopped
  regenerable run directory was removed before the next step.
- Registered matrix: exact-sample cap 8192 and cap 16384 as separate physical
  builds, holding 32 returned seeds, BW4/H100, graph degree 32, corpus, codec,
  queries, and topology fixed. This branch is required because exact cap-4096
  recall measured below the predeclared 0.9900 trigger.

## Durable artifacts and key lines

- `implementation-smoke/suite-manifest.json`: one succeeded step, no missing or
  stale artifacts; all six thresholds pass after provenance boolean
  normalization.
- `implementation-smoke/results.jsonl`: normalized topology, recall, latency,
  storage, head, engagement, and provenance rows. Key path-validation results:
  - three-node extension provenance unanimous at the implementation SHA;
  - topology gate passes with 10,000 total records/rows, zero non-owned rows,
    and zero orphans;
  - all four physical variants emit both membership and distinct recall plus
    distinct Wilson confidence bounds;
  - exact-neighbor emits `neighbor_score_mode=exact_neighbor` while persisted
    storage remains `stored_neighbor_code_format=rabitq`;
  - persisted head accounting emits sample count 4096, sample bytes 25,280,512,
    graph bytes 514,060, and estimated cache bytes 25,794,572;
  - remote engagement passes with two remote owners/materialization probes.
- `implementation-smoke/10k/distann-multinode-summary.log`: compact raw fixture
  source for the normalized rows above.
- `implementation-install.log`: clean release `cargo pgrx install` with
  `pg18 pg_test distann-head-attribution-benchmark`.
- `implementation-cli-build.log`: release CLI build used by the live fixture.
- `implementation-validation.log`: normal PG18 check, feature PG18 check, CLI
  check, and focused unit tests; all pass. The only warning is the pre-existing
  unused `LoadedDistributedPlacementConfig.path` field.
- `implementation-smoke-audit.log` and `screen-a-audit.log`: both configs pass
  `ecaz bench suite audit`.
- `implementation-smoke-resume.log`: re-normalizes the already-successful smoke
  after adding numeric provenance/engagement boolean fields; it does not rerun
  or alter the measured fixture.
- `screen-a/suite-manifest.json`, `screen-a/results.jsonl`, and
  `screen-a/100k/distann-multinode-summary.log`: immutable normalized and compact
  raw evidence for the completed first screen. Key 100k physical results:

  | Variant | Distinct recall@10 (95% CI) | Warm p50 | Warm p95 |
  | --- | ---: | ---: | ---: |
  | persisted width 32 / seeds 32 | 0.9275 (0.9153-0.9381) | 41.70 ms | 57.30 ms |
  | owner-scan oracle | 0.9970 (0.9935-0.9986) | 2467.60 ms | 2515.50 ms |
  | exact sample cap 4096 / seeds 32 | 0.9275 (0.9153-0.9381) | 42.20 ms | 55.20 ms |
  | persisted width 64 / seeds 32 | 0.9280 (0.9158-0.9385) | 40.00 ms | 52.80 ms |
  | persisted width 128 / seeds 32 | 0.9275 (0.9153-0.9381) | 40.80 ms | 53.90 ms |
  | persisted width 256 / seeds 32 | 0.9275 (0.9153-0.9381) | 41.20 ms | 55.00 ms |

  Width 64 is the pre-registered width-axis selection: it has the highest
  nominal distinct recall and lowest warm p50 among the overlapping intervals.
  Exact-sample equality establishes that approximate head-graph search is not
  the meaningful loss source at cap 4096. The 0.9275 exact-sample result fires
  the cap-growth branch. Owner recall of 0.9970 shows seeding is the primary
  loss but also remains below the final 0.9990 gate.
- `screen-a-report.md` and `screen-a-status.log`: generated suite report and
  completion audit. `screen-a-suite.log` is the top-level runner transcript;
  `screen-a-cli-build.log` records the release CLI build used for the run.
- `checksums.sha256`: SHA-256 checksums for the Screen A config, suite manifest,
  normalized results, compact raw summary, report, and status output. Later
  screen checksums will be appended after those suites complete.
- `screen-a/100k/physical-*-{recall,latency}.log`: cited per-arm raw measurement
  tables. Node PostgreSQL logs and the duplicate full fixture log were pruned.
- `screen-b-seeds/suite-manifest.json`, `screen-b-seeds/results.jsonl`, and
  `screen-b-seeds/100k/distann-multinode-summary.log`: immutable normalized and
  compact raw evidence for the returned-seed axis:

  | Returned seeds at width 64 | Distinct recall@10 (95% CI) | Warm p50 | Warm p95 |
  | ---: | ---: | ---: | ---: |
  | 32 | 0.9280 (0.9158-0.9385) | 40.30 ms | 52.60 ms |
  | 64 | 0.9280 (0.9158-0.9385) | 40.20 ms | 52.20 ms |
  | 128 | 0.9280 (0.9158-0.9385) | 41.40 ms | 53.60 ms |

  The 128-seed arm completed under the new returned-count invariant, proving
  that the flat result is not silent truncation to width 64. All three recall
  intervals are identical; seed count is not the cap-4096 limiter. The
  within-axis latency tie-break would choose 64 seeds, but all cells remain far
  below the 0.9990 quality gate and above the 37.6 ms p50 anchor.
- `screen-b-seeds-report.md`, `screen-b-seeds-status.log`, and
  `screen-b-seeds/100k/physical-*-{recall,latency}.log`: generated report,
  completion status, and cited per-arm raw tables. Node PostgreSQL logs and the
  duplicate full fixture log were pruned.
- `seed-count-install.log`: clean release install from branch head
  `53b62bbea...` with `pg18 pg_test distann-head-attribution-benchmark`.
- `screen-c-caps/{suite-manifest-cap8192.json,results-cap8192.jsonl}` and
  `screen-c-caps/{suite-manifest.json,results.jsonl}`: immutable normalized
  evidence for the two selected cap steps. Each manifest reports one succeeded
  and one deliberately skipped step because of the disk-safe `--only` split.
  Both use the same config SHA, query SHA, release extension SHA, and physical
  topology.

  | Exact-sample cap | Distinct recall@10 (95% CI) | Warm p50 / p95 | Head sample / graph / cache | Physical / publish |
  | ---: | ---: | ---: | ---: | ---: |
  | 4096 | 0.9275 (0.9153-0.9381) | 42.20 / 55.20 ms | 25,280,512 / 614,055 / 25,894,567 B | 863,543 / 990,021 ms |
  | 8192 | 0.9250 (0.9126-0.9357) | 43.50 / 55.70 ms | 50,561,024 / 1,230,461 / 51,791,485 B | 907,186 / 1,033,853 ms |
  | 16384 | 0.9440 (0.9330-0.9533) | 45.20 / 59.30 ms | 101,122,048 / 2,468,745 / 103,590,793 B | 1,019,494 / 1,149,168 ms |

  Cap 8192 is a negative recall/latency result. Cap 16384 has the highest
  nominal recall, but its interval overlaps the cap-4096 width-64 seed cells
  and it has worse latency plus four times the head bytes. Under the registered
  recall/overlapping-CI/p50/head-byte order, Phase 2 therefore selects persisted
  cap 4096 / width 64 / 64 returned seeds (same-run recall 0.9280, p50 40.20
  ms), not cap 16384.
- `screen-c-cap{8192,16384}-{report.md,status.log,suite.log}` and each cap
  directory's compact summary plus cited recall/latency tables: reports,
  completion evidence, runner transcripts, and raw measurement tables. Node
  PostgreSQL logs, duplicate full fixture logs, and run directories were pruned.

The exact-neighbor arm was not run: the selected bounded seeding result 0.9280
is 0.0690 below the same-run owner oracle 0.9970, so Task 180's within-0.0050
trigger is false.

Corpus TSVs, truth cache, run directories, node logs, and per-arm regenerable
logs are intentionally excluded from the packet.
