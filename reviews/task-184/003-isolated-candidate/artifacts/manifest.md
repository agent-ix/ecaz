# Task 184 packet 003 artifact manifest

- Task bucket / packet: `reviews/task-184/003-isolated-candidate/`
- Runner checkpoint: `eacd0026428a02cf306cc3be411a2007d9265b42`
- Candidate checkpoint: `245c2054f`
- Correctness-runner checkpoint: `7c2254e21`
- Status: complete; isolated candidate qualifies for full-scale confirmation

| Artifact | Command / meaning | Result |
| --- | --- | --- |
| `cargo-check-runner-extension.log` | `cargo check -p ecaz-cli` | pass |
| `cargo-test-runner-parser.log` | focused materialization-arm parser test | pass |
| `cargo-test-runner-suite.log` | focused same-generation expansion test | pass |
| `cargo-check-candidate.log` | feature PG18 check | pass |
| `cargo-check-candidate-production.log` | normal PG18 check | pass; benchmark GUC/path absent |
| `cargo-test-candidate-window.log` | focused deterministic ranked-window test | 1 passed |
| `cargo-check-correctness-runner.log` | CLI check after suite semantic-matrix extension | pass |
| `cargo-test-correctness-suite.log` | suite expansion / structured-result parser test | 1 passed |
| `cargo-test-correctness-sql.log` | nullable/toasted semantic SQL construction test | 1 passed |
| `implementation-install.log` | measurement-feature PG18 release install at `765f28a54` | pass |
| `cli-release-build.log` | release CLI build at correctness-fixture checkpoint `b51b0ad47` | pass |
| `install-provenance.log` | installed candidate library / runner CLI SHA-256 | `4ee29c...23031` / `fbeea8...48bba` |
| `suite-dry-run.log` | release-runner expansion of checked-in isolated suite | both steps and `0`/`10` arms present |
| `suite-audit-preflight.log` | suite input/shape audit | 2 steps pass |
| `isolated-candidate-suite.json` | checked-in suite config | 10k semantic gate + 100k isolated A/B |
| `isolated-run/suite-manifest.json` | final suite execution manifest | 2 completed, 0 failed |
| `isolated-run/results.jsonl` | normalized decision rows | 193 rows |
| `isolated-run/semantic-matrix-10k/distann-multinode-summary.log` | compact correctness summary | 7/7 scenarios pass |
| `isolated-run/isolated-ab-100k/distann-multinode-summary.log` | compact same-generation A/B summary | proceed |
| `suite-status.log` / `suite-audit.log` | post-run integrity checks | no missing/stale artifacts; audit pass |
| `suite-report.md` | runner-generated report | complete |

The runner accepts an optional sixth physical seed-variant field for the
materialization batch size. Five-field configs remain compatible and default
to eager materialization (`0`). Positive variants set the benchmark-feature-
only GUC in recall and latency child sessions while retaining the same
immutable generation and seed digest.

The candidate keeps pending remote `vec_id` identities in ranked output. On
executor demand it fetches all still-pending remote identities from the current
global 10-slot window, capped by the proven prefix, through the existing
schema/epoch-fenced endpoint. Search deepening starts a rebuilt window at the
already-consumed cursor, preventing duplicate fetches of the stable prefix.

Live correctness, same-generation isolated A/B, installed release provenance,
and compact suite outputs are under `isolated-run/`. The durable evidence set is
the checked-in config, suite manifest, normalized `results.jsonl`, and the two
step summaries; raw child, node, and per-arm logs were pruned as regenerable
exhaust.

## Live correctness result

The 10k semantic gate used a correctness-only nullable/toasted `payload_note`
column while keeping the real query vector unchanged. All seven structured
outcomes passed:

- unfiltered ordered output identity;
- quals rejecting the first 10 and first 20 ranked positions;
- genuine NULL payload datums;
- 12,800-byte toasted varlena projection plus qual;
- mixed winners (6 remote, 4 local);
- equal eager/lazy digests for every semantic query; and
- an actual remote-owner outage after a completed first fetch (6 remote rows
  requested), with the subsequent demanded batch aborting the query.

## Same-generation 100k result

Both variants used seed digest
`488caa73ad3f6c22864f9af309569ba4fe6edd72c8d535e71eec7bff78af6d50`,
head digest `50261d7627471fa3329535cd017ead6102cb220c62ca12dc9715178d05333b54`,
query SHA `a7cbec6f...41782`, the same published generation, and unanimous
release extension SHA `765f28a54` on all three owners.

| Metric | eager | lazy10 | Relative result |
| --- | ---: | ---: | ---: |
| distinct recall (200 / 2,000) | 0.9625 | 0.9625 | identical |
| mean latency | 39.30 ms | 23.40 ms | -40.5% |
| p50 | 38.80 ms | 23.10 ms | -40.5% |
| p95 | 50.50 ms | 26.50 ms | -47.5% |
| p99 | 55.60 ms | 27.50 ms | -50.5% |
| max | 56.20 ms | 28.10 ms | -50.0% |
| remote materialization | 25.910 ms | 10.292 ms | -60.3% |
| owner endpoint critical path | 23.105 ms | 9.210 ms | -60.1% |
| remote rows requested/query | 26.84 | 6.64 | -75.3% |
| logical payload bytes/query | 496,003 | 122,707 | -75.3% |
| remote rows consumed/query | 6.64 | 6.64 | identical |
| client rows/query | 10 | 10 | identical |
| physical generation bytes | 2,496,659,456 | shared | identical |

Construction was shared (`physical_ms=844493`, `publish_ms=970364`,
`single_ms=382927`). Topology, remote engagement, query separation, storage,
and installed release provenance all passed. The suite finished 2/2 steps with
0 failures, 0 missing artifacts, and 0 stale artifacts. The suite manifest's
runner label is `b51b0ad47-dirty` solely because packet-local output/log files
were created in the worktree before manifest capture; the runner binary hash is
recorded above and the independently attested installed extension is unanimous
release `765f28a54`.

Decision: **PROCEED** to packet 004's full 10k/50k/100k confirmation. The
candidate remains benchmark-feature-only and opt-in pending that decision.
