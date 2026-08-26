# Task 224 packet 003 artifact manifest

- Head SHA: `0ad5d63930bb021114585f64da5ab3622e4ddf7b`
- Task/packet: `task-224/003-isolated-candidate`
- Host/lane: Intel local, PG18, `distann-head-attribution-benchmark`
- Candidate: MAT-26 exact `real[]` binary sender, feature-only and default-off
- Measurement status: not yet run; this revision requests code and
  preregistration review before the 100k screen

## Preregistered suite

- Config: `crates/ecaz-cli/suites/task224-mat26-fast-real-array-100k.json`
- SHA-256:
  `ddad71b7f8d92b9ec3e061e2622ff09820d4edfb3ea400c196f7dbcbe8746d57`
- Scale/fixture: `ec_real_100k`, one immutable generation, three owners
- Projection: vector-bearing
- Storage format/rerank/search: identical frozen generation, production
  payload projection, RaBitQ neighbors, persisted head, BW4/H100/L32,
  eager-0 and production lazy-10 variants in both steps
- Instrumentation state: both arms use full stage/work counters but explicitly
  disable the Task 224 locality SQL wrapper; both therefore execute the same
  unprofiled production payload SQL shape
- Isolated variable: `owner_fast_real_array_send=false/true`
- Run directory: `/home/peter/.ecaz/clusters/task224-mat26-100k` (outside the
  repository, required for exact fixture reuse across suite steps; remove after
  cited results are captured)

## Validation artifacts

The following logs are generated at the exact head above:

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

A preliminary SQL-test invocation used an ordinary SPI `Err` assertion for the
deliberate wrong-type PostgreSQL ERROR. PostgreSQL correctly aborted the
enclosing SPI transaction, so the harness reported failure after the byte
comparisons had succeeded. The probe was split into pgrx's `#[should_panic]`
form; the final two-test command recorded here passes. This was a test-harness
correction, not a candidate-code correction.

Live `suite-manifest.json`, `results.jsonl`, recall/latency/storage/build/DML
logs, exact generation identity, and the A/B decision will be added after
outside review authorizes the run.
