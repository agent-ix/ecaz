# Task 224 packet 003 artifact manifest

- Initial code SHA: `0ad5d63930bb021114585f64da5ab3622e4ddf7b`
- Seq01 correction SHA: `7cafbd2027b05365afd47c6f8b34c0415e6b78fc`
- Seq02 correction / required run SHA:
  `b834b7fb3715b8fea27d78bbf577c2b47b55d220`
- Task/packet: `task-224/003-isolated-candidate`
- Host/lane: Intel local, PG18, `distann-head-attribution-benchmark`
- Candidate: MAT-26 exact `real[]` binary sender, feature-only and default-off
- Measurement status: live screen run after reviewer seq04 authorization;
  semantic control failed closed and the timing activation gate was
  structurally unobservable; reviewer seq05 accepted STOP with MAT-26
  unmeasured/no finalist; reviewer seq07 review-closed packet 003 ACCEPT after
  the seq06 correction round

## Preregistered suite

- Timing config:
  `crates/ecaz-cli/suites/task224-mat26-fast-real-array-100k.json`
- Timing config SHA-256:
  `47234e2880271108685c49114c92ab12b2d792cea9542153a622e668a25abff2`
- Semantic config: `crates/ecaz-cli/suites/task224-mat26-semantics-10k.json`
- Semantic config SHA-256:
  `d3a57b8b6d93bdf8d41bf5c9b31f9be6d5e6204b9cfb8d7a8ffd8cd714e09cb2`
- Scale/fixture: `ec_real_100k`, one immutable generation, three owners
- Projection: vector-bearing
- Storage format/rerank/search: identical frozen generation, production
  payload projection, RaBitQ neighbors, persisted head, BW4/H100/L32,
  eager-0 and production lazy-10 variants in both decision-bearing steps;
  repeat/context steps rerun only matched production lazy-10
- Timing steps: unprofiled control A, candidate, unprofiled control B, and a
  nonconforming profiled-control context arm, all on the same reused generation
  and with no correctness matrix or crash/restart drill
- Semantic steps: native control and fast-sender candidate, each a fresh 10k
  non-reuse fixture with its own run directory/ports; each runs the nine-case
  eager/lazy-10 correctness/failure matrix and has no timing decision weight
- Headline instrumentation state: control A/B disable the Task 224 locality SQL
  wrapper; the candidate keeps that SQL wrapper disabled but its feature-only
  sender carries the timing/buffer shim needed for activation attribution. The
  profiled context arm measures native `typsend` behind the same shim and, by
  wrapping both projected values, conservatively upper-bounds the candidate's
  one-value shim cost.
- Isolated headline variable: `owner_fast_real_array_send=false/true`
- Control-envelope floor:
  `N = abs(control_a-control_b) / mean(control_a,control_b)`; because control B
  is a later lazy-10-only `stage_counter_only` step, `N` includes protocol and
  position drift, while control A is measured after building and warming the
  fixture. These differences can only inflate the required `2*N` improvement
  bar.
- Attribution gate: select the unique lazy-10/physical/vector-bearing
  `physical_benchmark_stage` rows for `materialize_owner_binary_send_work`,
  `materialize_owner_endpoint_critical`, and
  `materialize_owner_endpoint_work`; require every `scans` field to equal its
  latency-row `count` and all counts to equal 200. With
  `R=min(P_critical/P_work,F_critical/F_work)` in `(0,1]`, require
  `D_attr=(P_send-F_send-0.005083)*R > 0` and
  `D_attr >= 0.5*(C-candidate_mean)`. The fixed 0.005083 ms subtraction is
  packet 002's measured scalar-send cost, 0.073% of the vector send bucket.
- Activation gate: candidate fast values >0; fallback values =0; ineligible
  requests =0 for vector-bearing latency
- Build gate: all six steps leave `allow_debug_extension=false`; the extension
  must be a release, non-`pg_test` attribution build and the CLI must be a
  release build from exact SHA
  `b834b7fb3715b8fea27d78bbf577c2b47b55d220`. Preflight must be unanimous and
  every normalized row must repeat that exact SHA/profile. Build both artifacts
  from a clean detached checkout at that SHA, not from branch HEAD.
- Semantic gate: each semantic step exits zero and emits exactly one
  `physical_materialization_correctness` row for each of
  `fewer_than_window`, `exactly_one_window`, `more_than_window`,
  `reject_first_window`, `reject_multiple_windows`, `null_payload`,
  `toasted_projection_qual`, `mixed_local_remote`, and
  `post_first_batch_remote_failure`; any duplicate or missing scenario fails.
- Run directory: `/home/peter/.ecaz/clusters/task224-mat26-100k` (outside the
  repository, required for exact fixture reuse across suite steps; remove after
  cited results are captured)
- Semantic run directories:
  `/home/peter/.ecaz/clusters/task224-mat26-semantic-control-10k` and
  `/home/peter/.ecaz/clusters/task224-mat26-semantic-candidate-10k`; remove both
  after their cited matrix rows are captured
- Volatility/parallel safety: the fast function remains
  `volatile, parallel_restricted` while `array_send` is immutable/parallel-safe.
  The value is consumed inside a per-row lateral payload expression where
  neither arm can be folded or parallelized; any residual planner effect is a
  candidate handicap. The profiled context's extra scalar wrapper is instead
  anti-conservative for attribution, so its measured 0.005083 ms/scan cost is
  explicitly subtracted by the gate above.

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
  `8ae333ce18038ed19f53b5c332ac1f8b7fab73d536d957d9381b093acce01bd4`
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

Reviewer seq02
(`feedback/2026-08-25-02-reviewer.md`) accepted the sender, bitmap parity,
native fallback, outcome telemetry, release preflight, provenance, feature
isolation, endpoint arity, and 5% usefulness threshold. It withheld run
authorization because the timing candidate combined fixture reuse with a
fixture-mutating correctness matrix, control A's matrix would restart owners
before the candidate, the attribution rule was not executable from named
artifact rows, and the hash above had an extra trailing character.

The following seq02-correction logs were generated at exact code/config head
`b834b7fb3715b8fea27d78bbf577c2b47b55d220`:

- `review-fix2-cargo-check-pg18.log` — normal PG18 build, pass; SHA-256
  `8f340b3fb841716084ea50d25a83f33012a2624e409d5afcd4c4d7d3ac732ef4`
- `review-fix2-cargo-check-pg18-feature.log` — attribution-feature PG18 build,
  pass; SHA-256
  `b5aaa44738330514d22eebeca04b3af16d27ab67002d3fb0e27f07847cba47b6`
- `review-fix2-cargo-test-ecaz-cli-task224.log` — all six Task 224 CLI,
  timing-suite, semantic-suite, and provenance tests pass; SHA-256
  `e55d9ac54cc90f1630bc5f6b111e09bc145853009672debeb23915308d6cebed`
- `review-fix2-cargo-test-reuse-exclusions.log` — the focused suite validator
  test covers all three fixture-mutating drill exclusions; one pass; SHA-256
  `946889ea5f04d57b92e095b48243b67e1fc92028cdc5a696f6e723b62c4aa86a`
- `review-fix2-cargo-fmt-check.log` — formatter gate, pass; SHA-256
  `56faf67e0b699d69d624bcc1db0d37c4ee02b06f91ad8e7186ff7fbeb7206930`

The exact-checkpoint dry runs are retained under `artifacts/dry-run/`:

- `timing.log` — four timing commands expand without
  `--materialization-correctness`; SHA-256
  `2f8ba0db248230193917fdd1c074b5efd21f1c709b6eff92390544eaf2c82127`
- `timing/suite-manifest.json` — runner SHA is exact `b834b7fb...`, control A
  is the only non-reuse timing step, and no step enables the debug bypass;
  SHA-256
  `4217a631dfb013d0d50580481896176bbc7ff2f54205a02c32a730c18073338a`
- `semantics.log` — two isolated semantic commands expand with
  `--materialization-correctness` and without `--reuse-fixture`; SHA-256
  `0d3e76d0c1cbdaf6f85f727697904c52225d6f703dd6b21851c75ed287cf504f`
- `semantics/suite-manifest.json` — runner SHA is exact `b834b7fb...` and no
  step enables the debug bypass; SHA-256
  `7753af463e7ebf20d54aa89e08226df398864febd4460e977cb332b59bbbb551`

These dry runs validate argument construction only. They are not measurement
evidence and carry no latency, recall, storage, or semantic decision weight.

Reviewer seq03 (`feedback/2026-08-25-03-reviewer.md`) independently verified
B1--B4 closed, all 22 hashes exact, all named timing-gate fields executable,
and the reused timing fixture unmutated. It withheld both suites only because
the required attribution build emits nine correctness rows, not the previously
preregistered seven. The semantic gate above is the pre-measurement correction:
step exit plus the complete nine-scenario set, exactly once per step.

Reviewer seq04 (`feedback/2026-08-25-04-reviewer.md`) accepted the correction
and authorized both suites with no gate amendment. Both were then invoked from
the clean detached `b834b7fb...` checkout, so the CWD-derived runner SHA and the
compiled extension/CLI SHA agree.

Reviewer seq05 (`feedback/2026-08-25-05-reviewer.md`) accepted STOP/no rerun but
identified the structurally unobservable activation gate and required
decision-record, tracking, and diagnostic-comment corrections. Those changes
do not alter runtime behavior. `review-fix3-cargo-fmt-check.log` records
`cargo fmt --all -- --check` passing with exit 0; SHA-256
`82aea97006f3560b7ec6e933b059b9b03f1c688e575197f138cc6ec135cb6433`.
No behavior test was rerun for comment/documentation-only corrections.

## Live screen artifacts and result

- Decision record: `screen-decision.md`; full provenance, all gate terms, raw
  context values, unavailable terms, and STOP disposition
- CLI build log: `release-build-cli-b834b7fb.log`; release build pass; SHA-256
  `03e668c263f03128de4febf15a5ecba4ea967291f147dda1d50ac3941a1e475d`
- Extension install log: `release-install-extension-b834b7fb.log`; PG18
  release attribution install pass; SHA-256
  `94212eb7c36b0626cfd72fd428bd050c2081ed2475e01cad0becbeb7590e733e`
- Semantic suite log: `semantic-run.log`; exit 1 at native-control bounded-read
  failure; SHA-256
  `d406fbcaccc06f42801a96b7448926caea2021f2b4a98197286d7dbd59b7c2af`
- Semantic suite tree: `semantic-run/`; fresh one-index-per-table 10k control
  fixture, runner SHA `b834b7fb...`; control failed, candidate pending; no
  `results.jsonl` was emitted
- Timing suite log: `run.log`; exit 1 at candidate activation failure; SHA-256
  `5931d0fe289eefed7fe8d0725a975668d8d60e1555592a96780736e94a7d6070`
- Timing suite tree: `run/`; fresh one-index-per-table 100k control followed by
  exact fixture reuse; control A succeeded, candidate failed, control B and
  profiled control pending; no `results.jsonl` was emitted

The 10k native semantic control failed `exactly_one_window` with correct and
identical 10/10 results but `remote_requested=8`, `local_consumed=4`, and
`payload_reads=12/10`. The 100k candidate passed exact-SHA reuse and emitted a
byte-identical eager prediction file. Its executed latency arm exported five
zero outcome counters, but the accepted unprofiled fast-sender configuration
leaves locality-derived `owner_requested_tids=0`, and coordinator accounting
suppresses all five counters in that case. The CLI activation assertion was
therefore unsatisfiable: the zeros carry no sender information, including the
two formerly labelled passes. The candidate reached 400 remote owners, 6,328
remote candidates/payloads, 12,656 payload columns, and 77,960,960 payload
bytes; whether the exact sender activated is unknown.

The raw eager 44.6→26.3 ms control/candidate movement is a 41% unattributable
gap—about eight times the 5% decision threshold—not a candidate result. The
arms differ in both fixture position/warmth and the fast-sender flag whose
effect is unobservable; the gap is neither a sender win nor evidence of sender
inactivity. Candidate lazy-10, control B, and profiled control never ran, so
`C`, `N`, the usefulness and tail comparisons, `R`, and `D_attr` are not
computable. No post-hoc rerun was attempted or authorized.

Corpus provenance:

- staged 10k manifest SHA-256
  `cb3c68a3090ab4ff767f4e36448e5d90a95ae6416b50265a991d96184d00a561`;
  corpus/query SHA-256 `c67c5810...35e75` / `a2c191bb...04ae8`
- staged 100k manifest SHA-256
  `a0bc0522299fc8b331bc63e22b141b406f87f9894109d985a60f68fb4148c574`;
  corpus/query SHA-256 `07275cfd...3a95` / `a7cbec6f...1782`

All live-run files are enumerated with byte hashes in
`live-artifact-sha256.txt`; ledger SHA-256
`24293bb640d391c3880531bcb7e7a7a733914cb22db0498a88220e6c0036b762`.
Temporary cluster directories were stopped by the harness and are removed
after this record was captured.

Disposition: **STOP with MAT-26's latency effect unmeasured, a void candidate
axis, and no Task 224 finalist.** Do not advance to packet 004 or a full-scale
matrix. Production remains unchanged and the feature-only/default-off
candidate remains as diagnostic code. Task 239 carries the independent native
12/10 bounded-read divergence; Task 225 remains conditional on its own premise,
and Task 229 is the next mandatory prototype once that semantic blocker is
closed.
