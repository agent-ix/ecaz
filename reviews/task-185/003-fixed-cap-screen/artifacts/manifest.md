# Task 185 fixed-cap screen manifest

- Task bucket / packet: `reviews/task-185/003-fixed-cap-screen/`
- Status: 100k control plus isolated-attribution capture complete; selector
  screen pending
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
selector result is claimed from it.

## Captured 100k control

- Suite manifest: `run/suite-manifest.json`; one step succeeded, exit code 0,
  duration 2,173,032 ms; runner SHA `a0200166cb5ef236ac5f2bcf20a1bb46d9ce5140`
- Normalized results: `run/results.jsonl`; SHA-256
  `0ae677b4556664813c1d824b72043db80ec3970bb9b8ed18db026069b55de604`
- Summary: `run/training-landmarks-control-100k/distann-multinode-summary.log`;
  SHA-256
  `99540f845ba3ef32c7cc62b07be2c83c96fa3e3ad69b7eb1eee844a723d260cc`
- Isolated trace: `run/training-landmarks-control-100k/physical-control-gateway-isolated-trace.json`;
  SHA-256
  `cfbe7dc03f286c83f6b14f5db2a6710ab5dc49fd18c640b2c138b7cf22d5b3d5`
- Isolated analysis: `run/training-landmarks-control-100k/physical-control-gateway-isolated-analysis.json`;
  SHA-256
  `cc41ff27d2fcc1fb7f32c910bd0ecdb0801d3a6d7b4342099ab19cd68e58fcbe`
- Recall: 0.9205, 95% CI [0.9078, 0.9316], 200 queries / 2,000 trials;
  `physical-control-recall.log`; SHA-256
  `eb15f1f5b8f7215555d41f7bd6f99f9d41162156c95cd5b67893de832ae55e90`
- Warm latency: 43.00 ms, 10 warmups, one measured sample;
  `physical-control-latency.log`; SHA-256
  `acbe092833d73d233d9c6d6186307008fe61888b97a5947167921a1127501649`
- Physical generation bytes: 2,496,659,456; construction 955,502 ms;
  publication 1,094,062 ms; three owners, two remote owners, release profile,
  unanimous extension SHA `57ee20b5da9df0d5efe1a922a12808ab62ad52e9`.

The isolated trace covers 800 reruns (four returned seed positions × 200
training queries). Exact training truth coverage is 1,833/2,000 (91.65%);
ordered marginal coverage for positions 1--4 is [1,577, 138, 84, 34], with
4,343 redundant seed truth hits. These are valid returned-seed basin
diagnostics only; they do not score arbitrary members of the 4,096-row head
and do not select a gateway policy.

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

The completed external cluster and operational logs were removed after the
packet-local evidence was captured. The native 10k smoke remains a fixture
input-shape failure: its query file has 200 rows, while the training-landmark
path reserves rows 201--400. No 10k benchmark result is claimed.
