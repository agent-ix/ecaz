# Task 185 fixed-cap screen manifest

- Task bucket / packet: `reviews/task-185/003-fixed-cap-screen/`
- Status: preregistration plus isolated-attribution implementation; no screen
  benchmark artifacts captured
- Implementation head: `2ffe120a5a2e205d08c16539f0747ab31dbcf268`
- Planned first scale: 100k (`ec_real_100k`)
- Planned topology: three-owner physical sharded generation, one index per
  arm, run directories under `/home/peter/.ecaz/clusters/`
- Planned controls: cap 4096, exact head scoring, 32 seeds, graph degree 32,
  BW4/H100, RaBitQ traversal, exact final ranking
- Planned split: training rows 201--400; held-out evaluation rows 1--200
- Required runner: `ecaz bench suite` with a packet-local SuiteConfig

The implementation and candidate-level attribution contract must be reviewed
and committed before any run is added here. Corpus TSVs, truth caches, cluster
directories, and operational logs are not packet evidence and must not be
committed.

## Validation artifacts

- `gateway-isolated-attribution-100k-suite.json` — SuiteConfig with four-position
  isolated diagnostic; SHA-256
  `943ceb0f7157cd9b7e9752b645eaf23991ee93ef4fe2beeb60ef8f939446da68`
- `pg18-feature-check.log` — feature build with the benchmark endpoint enabled;
  SHA-256 `5936d6c6573e92774a2b726ce8d06377e875d0d25ba115cee1fc27a365473f26`
- `pg18-featureless-check.log` — production-shaped featureless build;
  SHA-256 `2f011af5f7c379f486c6e8d990edc82a9d2b8255691e115763bde7c9eb299b3e`
- `isolated-suite-preflight.log` — audit, dry-run, and current release
  preflight; SHA-256
  `ea3f2c6ca0d88419d2fbdfe123c87958068a0f63409e2f662f2281ef98f8010d`

The isolated endpoint currently covers one member at a time from the control's
returned seed list. It is not an alternate-head selector and no benchmark
result is claimed from it.

## Runner validation

- `gateway-isolated-attribution-100k-suite.json` — checked-in SuiteConfig
- `isolated-suite-preflight.log` — audit, dry-run, current release preflight,
  and cleanup record
- `suite-dry-run.log` — bounded runner expansion; SHA-256
  `ccb200665aa95c0e1cffc2f600f9a0095bfd4af640920aaa5faa7241c14a7222`
- `suite-run.log` — suite invocation record; SHA-256
  `584f998275bf4813286056b2a221eca68f87bd6331512fe95f38aace6d2437a1`
- `gateway-isolated-attribution-10k-smoke-suite.json` — suite config for the
  input-shape smoke; SHA-256
  `79469dd0e9dd61855d0d5fbe2f2afc408efd0b870c6cb8f720b6bc99b82dd298`
- `smoke-dry-run.log` — suite audit/dry-run for the bounded four-position
  smoke; SHA-256
  `977874d78b82988667054e2c77534f90af9d26643f478b5048e25892bb13a683`
- `smoke-input-shape.log` — concise failure record and fixture row counts;
  SHA-256
  `b982828fdbcf5fa4ad4d5bf4bc7e8fa63a475d411461ec8bf0a08ae709ca2830`

The 100k attempt was stopped during physical setup before any benchmark
milestone or result artifact. Its temporary cluster and operational logs were
removed; no incomplete measurement is used as evidence.

The 10k smoke also stopped during physical setup: its native query fixture has
200 rows, while the training-landmark path reserves rows 201--400. The exact
failed cluster and raw operational output were removed; no smoke result is
claimed.
