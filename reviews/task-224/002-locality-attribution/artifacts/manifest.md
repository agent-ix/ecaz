# Task 224 packet 002 artifact manifest

Date: 2026-08-25 (America/Los_Angeles)

- Task bucket / packet: `reviews/task-224/002-locality-attribution/`
- Head / runner / extension SHA:
  `3328b1be1e6dc4e4b014c4dbcf3a75ddc625243c`
- Lane: local Intel Core i9-10900K x86_64, three-owner physical PG18, release
  extension with `pg18,distann-head-attribution-benchmark`
- Fixture: `ec_real_100k`, 100,000 rows, 200 held-out queries, top-k 10,
  persisted head 4096, BW4/L32/H100, production lazy-10 materialization
- Isolation / reuse: the id-only step created one fresh three-owner generation;
  narrow-scalar, vector-bearing, and toasted steps reused that exact fixture.
  All four result rows report epoch fingerprint
  `0200c6a2417349f759d4782652fd9f7a4d7b1ff2a37e0c1fa32f7552db10a8262e26`
  and the same query-slice SHA-256
  `966fcdfb55bd8c36b05ca308e871407237e9bc22ad4355d42dd51db0b54e42c3`.
- Runtime directory: `/home/peter/.ecaz/clusters/task224-owner-locality-100k`,
  outside the repository. It was removed after the cited artifacts were
  captured and every PostgreSQL node had stopped.
- Result source of truth: `artifacts/run/results.jsonl`
- Decision calculation: `artifacts/gate-calculation.md`

## Suite command

```text
/home/peter/.cargo-target/release/ecaz bench suite run \
  --config crates/ecaz-cli/suites/task224-owner-payload-locality-100k.json \
  --manifest-output reviews/task-224/002-locality-attribution/artifacts/run/suite-manifest.json \
  --results-output reviews/task-224/002-locality-attribution/artifacts/run/results.jsonl \
  --log-file reviews/task-224/002-locality-attribution/artifacts/suite-run.log
```

The suite was generated at `2026-08-25T17:38:53-07:00`. All four selected
steps succeeded:

| Step | Duration (ms) | Reuse disposition |
| --- | ---: | --- |
| `owner-locality-id-only-100k` | 1,403,403 | fresh fixture/build |
| `owner-locality-narrow-scalar-100k` | 149,393 | exact generation reuse |
| `owner-locality-vector-bearing-100k` | 153,905 | exact generation reuse |
| `owner-locality-toasted-100k` | 157,720 | exact generation reuse |

The id-only step's build rows report `physical_ms=827070` and
`publish_ms=954064`; reuse steps correctly report zero build/publish time.
Every step reports unanimous release SHA/profile provenance and identical
physical storage (`3,189,694,464` bytes, amplification `1.353813`). This packet
is a four-shape attribution gate, not a candidate A/B, so its one-scale
NFR-021 aggregate is expectedly `unavailable` and is not used as a scaling
claim.

## Key cited results

| Shape | Warm mean / p95 / p99 (ms) | Payload SPI (ms/scan) | Binary send (ms/scan) | SPI-minus-send (ms/scan) |
| --- | ---: | ---: | ---: | ---: |
| id-only | 11.80 / 14.30 / 15.80 | 0.491981 | 0.005083 | 0.486898 |
| narrow scalar | 12.10 / 14.40 / 15.80 | 0.563287 | 0.010019 | 0.553268 |
| vector-bearing | 28.20 / 31.40 / 33.90 | 7.799571 | 6.967996 | 0.831575 |
| toasted | 46.10 / 69.90 / 74.40 | 2.049820 | 0.431869 | 1.617951 |

The registered tie-break selects MAT-26's 24.709206% vector-bearing ceiling
over MAT-25's 3.509655% per-scan toasted ceiling. See
`gate-calculation.md` for timer scope, locality counters, alternative toasted
query normalization, and arithmetic.

## Build and validation artifacts

- `artifacts/pgrx-install-release.log`: release PG18 attribution extension
  install, exit 0; installed `ecaz.so` SHA-256
  `5d1db9b20e262ffd3c7292d6c17a02ce06a969861a2c93c78415349d78e44fc9`.
- `artifacts/cargo-build-ecaz-cli-release.log`: release CLI build, exit 0; CLI
  SHA-256 `fc095d9fec095ac9795622dfbcdf93261dde086c2fe3a8818c027aa662ba30d6`.
  It contains one pre-existing `LoadedDistributedPlacementConfig.path`
  dead-code warning and no error.
- `artifacts/suite-dry-run.log` and `artifacts/suite-dry-run-manifest.json`:
  final four-step command expansion before the live run.
- `artifacts/test-owner-tid-profile.log`: focused feature test, 1 passed.
- `artifacts/test-profiled-payload-sql.log`: focused feature SQL test, 1 passed.
- `artifacts/test-production-payload-sql.log`: featureless production SQL pin,
  1 passed.
- `artifacts/test-ecaz-cli-task224.log`: payload-shape SQL and suite expansion /
  structured-row tests, 2 passed.

## Durable SHA-256 identities

- Suite config: `db524924ff4df2669eb00a1b3011609b08d687d79961a24b2074258934f06f94`
- Live suite manifest: `78c1fb57f33070b466a451a786fd3049100c29cd70a7b442a6a6947e86a77925`
- Normalized results: `6ae594061b42403e8f2153153cdacb60ffc5beefe39378b9f2e9a410b0e17504`
- Suite log: `4ebc9544783665e6fd0354d7e73623028fcd0ffc9e500d622a7abae3a674dce4`
- Dry-run manifest: `b55f257d76918f8f3e565ded4d78f6ede493851b0ee19c342fb14e02e0082d3f`
- Dry-run log: `b616b2558e370ea0d6295b2b88cd805109b22c77664bf8c64bec500c5bf0cd77`
- PG18 install log: `1199f52f33556f9ae355acc2295665ebf364964125cb34ae7aeca28dfc3279d5`
- CLI release-build log: `beab99020a218b81f888d9749b4a90cea19a72017d4e6eac14e65a0cd59278b0`
- TID-profile test log: `a7f21de358eacf3df756dc2ebb1369b356e9d73dd2fc8ef5f9c63621fe603d65`
- Profiled-SQL test log: `9ec33dcf749ebb4ee8d8473da67a53cca604a59abe50651799a9b6ef526bd75b`
- Production-SQL test log: `d87c28ad6aaa02067c770bbb68b3141e204bf0f10596c34dd44582ef82d7c37b`
- CLI Task 224 test log: `b559fb82fc7e1cf8f2da7754bbe3401a836cb913262ab5af880395824327eb56`

No corpus/query/truth data, PGDATA, tunnel state, polling exhaust, or failed-run
artifact tree is committed. The successful reusable fixture was intentionally
deleted after evidence capture.
