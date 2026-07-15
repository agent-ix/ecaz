# Task 180 packet 002 artifact manifest

This manifest covers the benchmark-surface implementation checkpoint and its
compact PG18 smoke. The smoke proves that every diagnostic path executes; its
two-query/two-iteration measurements are **not** Phase 1 decision evidence.

## Provenance

- Owning task / packet: `task-180` / `reviews/task-180/002-100k-attribution-screen/`
- Implementation commit under test: `174da94efc82d1f0ea4d11751eb8a834f4d8c29f`
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
- Status: audited and dry-run expanded; Phase 1 execution pending.
- Registered matrix: 100k production, owner oracle, exact bounded sample, and
  persisted-head widths 32/64/128/256, all returning 32 seeds, with 200 held-out
  queries / 2,000 membership trials and 50 latency iterations after 10 warmups.

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

Corpus TSVs, truth cache, run directories, node logs, and per-arm regenerable
logs are intentionally excluded from the packet.
