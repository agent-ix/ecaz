# Task 224 packet 003 artifact manifest

- Initial code SHA: `0ad5d63930bb021114585f64da5ab3622e4ddf7b`
- Corrected code SHA: `7cafbd2027b05365afd47c6f8b34c0415e6b78fc`
- Task/packet: `task-224/003-isolated-candidate`
- Host/lane: Intel local, PG18, `distann-head-attribution-benchmark`
- Candidate: MAT-26 exact `real[]` binary sender, feature-only and default-off
- Measurement status: not yet run; reviewer seq01 returned NOT DONE and the
  corrected checkpoint/preregistration require rereview before the 100k screen

## Preregistered suite

- Config: `crates/ecaz-cli/suites/task224-mat26-fast-real-array-100k.json`
- SHA-256:
  `d9b086cc4664390dd8833e2ff8db8965e98a41a35965159cce14feda7834e941`
- Scale/fixture: `ec_real_100k`, one immutable generation, three owners
- Projection: vector-bearing
- Storage format/rerank/search: identical frozen generation, production
  payload projection, RaBitQ neighbors, persisted head, BW4/H100/L32,
  eager-0 and production lazy-10 variants in both decision-bearing steps;
  repeat/context steps rerun only matched production lazy-10
- Ordered steps: unprofiled control A, candidate, unprofiled control B, and a
  nonconforming profiled-control context arm, all on the same reused generation
- Headline instrumentation state: control A/B disable the Task 224 locality SQL
  wrapper; the candidate keeps that SQL wrapper disabled but its feature-only
  sender carries the timing/buffer shim needed for activation attribution. The
  profiled context arm measures native `typsend` behind the same shim and, by
  wrapping both projected values, conservatively upper-bounds the candidate's
  one-value shim cost.
- Isolated headline variable: `owner_fast_real_array_send=false/true`
- Noise floor: `N = abs(control_a-control_b) / mean(control_a,control_b)`;
  usefulness requires both >=5% and >=`2*N` warm-mean improvement
- Attribution gate: profiled-native send-region saving versus candidate must be
  positive and explain at least 50% of the headline end-to-end warm-mean saving
- Activation gate: candidate fast values >0; fallback values =0; ineligible
  requests =0 for vector-bearing latency
- Build gate: every step leaves `allow_debug_extension=false`; the run requires
  a release, non-`pg_test`, attribution-feature extension at the reviewed SHA
- Run directory: `/home/peter/.ecaz/clusters/task224-mat26-100k` (outside the
  repository, required for exact fixture reuse across suite steps; remove after
  cited results are captured)

## Validation artifacts

The following initial-checkpoint logs were generated at the initial head above:

- `cargo-check-pg18.log` — production build, candidate surface absent;
  SHA-256 `f57f4cf0ea1682b4706580010cacd14867318ef8f47d69874a04776d07bf12a3`
- `cargo-check-pg18-feature.log` — attribution feature build; SHA-256
  `79a9a3cb9119fe91b2751fcf4cc67303fdb064428c54efcbd09fabbf193bc34d`
- `cargo-test-ecaz-cli-task224.log` — four Task 224 CLI/suite tests;
  SHA-256 `33bfc6452575ad5c1267884bf8b29283a552dcdc9f7be47833d8fa387c34ad4a`
- `cargo-pgrx-test-pg18-fast-real-array.log` — SQL-level byte identity and
  wrong-type fail-closed checks; SHA-256
  `10a8249861e185a464d73a2db1faa496a2a88d93ac44df5150bb7977f71f27d8`
- `cargo-fmt-check.log` — formatter gate; SHA-256
  `a66e66e8bae5d635b7fbd2e4de0042d40fa8deff1ff30bd3ab5478120d08bec2`

The following corrected-checkpoint logs were generated at code head
`7cafbd2027b05365afd47c6f8b34c0415e6b78fc`:

- `review-fix-cargo-check-pg18.log` — normal production build, pass; SHA-256
  `f7bdf356cc883cc8aaf791489863cffde263e15ce8b326e5cbde03b9d626ca5a`
- `review-fix-cargo-check-pg18-feature.log` — attribution-feature build, pass;
  SHA-256 `7fbdbe9dcd82828892bdaf045395777deba132d0da254861d3c38b8ca0588f9d`
- `review-fix-cargo-test-ecaz-cli-task224.log` — all five Task 224 CLI,
  preregistered-suite, and provenance tests pass; SHA-256
  `8ae333ce18038ed19f53b5c332ac1f8b7fab73d536d957d9381b093acce01bd4a`
- `review-fix-cargo-test-fast-real-array-encoder.log` — two pure encoder tests
  pass; SHA-256
  `ca41b7277d5993d2518aeeb73324c07a61866bfeb391101e7737355318e19679`
- `review-fix-cargo-pgrx-test-pg18-fast-real-array.log` — PG18 byte identity,
  including the bitmap-without-NULL case, and wrong-type fail-closed checks;
  two pass; SHA-256
  `e0e1b05d84e3f99f7957386a937beba1a2887830bab715cdf3e8bb0c4d5a2fee`
- `review-fix-cargo-fmt-check.log` — formatter gate, pass; SHA-256
  `722edb903790e637ee7ce8645acb7411b38d46fbeeb8b7d870e7f446b7c4c192`

A preliminary SQL-test invocation used an ordinary SPI `Err` assertion for the
deliberate wrong-type PostgreSQL ERROR. PostgreSQL correctly aborted the
enclosing SPI transaction, so the harness reported failure after the byte
comparisons had succeeded. The probe was split into pgrx's `#[should_panic]`
form; the final two-test command recorded here passes. This was a test-harness
correction, not a candidate-code correction.

Reviewer seq01 (`feedback/2026-08-25-01-reviewer.md`) found four blockers at the
initial head: session-wide failure on non-vector projections, a debug `pg_test`
installation with the preflight bypassed, byte divergence for bitmap-bearing
arrays with no remaining NULL, and asymmetric instrumentation without a bound.
The corrected checkpoint degrades non-vector requests with explicit outcome
telemetry, falls back for every bitmap-bearing array, removes debug overrides,
adds control-repeat and profiled-context steps, and pins the provenance suffix.
The PG18 regression now includes the exact bitmap-without-NULL reproducer and
the encoder returns before forming an empty-array data slice.

Live `suite-manifest.json`, `results.jsonl`, recall/latency/storage/build/DML
logs, exact generation identity, and the A/B decision will be added after
outside review authorizes the run.
