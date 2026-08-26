# Task 224 packet 003 live-screen decision

Disposition: **STOP — MAT-26's latency effect is unmeasured, the candidate
axis is void, and Task 224 has no finalist. Do not advance to packet 004 or a
10k/50k/100k release matrix.**

The semantic suite failed its native-control bounded-read gate. The timing
suite stopped at an activation assertion that no runnable configuration could
satisfy, so its five zero outcome counters carry no information about whether
the exact sender ran. This is neither evidence that the sender is slower nor
evidence that it was inactive. Production remains unchanged; the candidate is
feature-only and default-off.

## Run identity

- Code/config/build SHA:
  `b834b7fb3715b8fea27d78bbf577c2b47b55d220`
- Runner CWD: clean detached worktree
  `/home/peter/dev/ecaz/.worktrees/task224-run-build` at that exact SHA
- CLI: `/home/peter/.cargo-target/release/ecaz`, SHA-256
  `b3b098936bc4a5ecef133e7e0897cc643fbf884b2b6f7aeb5092ab0b39016c6b`
- Installed PG18 extension: release attribution build, SHA-256
  `d72fae1db4d83fec0eb76d98db91274d49474cabb567aa684dda2066b0d3983f`
- Extension preflight, in both executed steps: unanimous three-node
  `extension_git_sha=b834b7fb3715b8fea27d78bbf577c2b47b55d220`,
  `extension_build_profile=release`, features
  `distann-head-attribution-benchmark,pg18`, no debug override
- Timing config SHA-256:
  `47234e2880271108685c49114c92ab12b2d792cea9542153a622e668a25abff2`
- Semantic config SHA-256:
  `d3a57b8b6d93bdf8d41bf5c9b31f9be6d5e6204b9cfb8d7a8ffd8cd714e09cb2`
- 10k staged manifest SHA-256:
  `cb3c68a3090ab4ff767f4e36448e5d90a95ae6416b50265a991d96184d00a561`;
  corpus/query identities
  `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75` /
  `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`
- 100k staged manifest SHA-256:
  `a0bc0522299fc8b331bc63e22b141b406f87f9894109d985a60f68fb4148c574`;
  corpus/query identities
  `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95` /
  `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`

Commands:

```text
cargo build --release -p ecaz-cli
env PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18,distann-head-attribution-benchmark
/home/peter/.cargo-target/release/ecaz bench suite run --config crates/ecaz-cli/suites/task224-mat26-semantics-10k.json --artifact-dir /home/peter/dev/ecaz/.worktrees/task224-owner-payload-locality/reviews/task-224/003-isolated-candidate/artifacts/semantic-run --log-file /home/peter/dev/ecaz/.worktrees/task224-owner-payload-locality/reviews/task-224/003-isolated-candidate/artifacts/semantic-run.log
/home/peter/.cargo-target/release/ecaz bench suite run --config crates/ecaz-cli/suites/task224-mat26-fast-real-array-100k.json --artifact-dir /home/peter/dev/ecaz/.worktrees/task224-owner-payload-locality/reviews/task-224/003-isolated-candidate/artifacts/run --log-file /home/peter/dev/ecaz/.worktrees/task224-owner-payload-locality/reviews/task-224/003-isolated-candidate/artifacts/run.log
```

No script, alternate runner, config edit, `--continue-on-error`, resume, or
post-failure selected-step run was used.

## Semantic suite

`semantic-run/suite-manifest.json` records runner SHA `b834b7fb...`, native
control `failed` with exit 1, and candidate `pending`. No `results.jsonl` was
emitted because the suite stopped at the failed step.

The native control returned correct eager/lazy row identity for
`exactly_one_window` but failed the registered bounded-read invariant:

```text
rows=10/10
identity=true
null_ok=true
external_toast_ok=true
remote_requested=8
local_consumed=4
payload_reads=12/10
duplicate_requested=0
```

Source: `semantic-run/control/distann-local-multinode.log`. The prior accepted
10k attribution example in Task 198 produced 6 remote + 4 local = 10, so the
12/10 result is a real current-checkpoint divergence, not the expected count
for this scenario.

Consequences:

- semantic control step exit: **FAIL**
- complete nine-scenario control set: **FAIL, incomplete**
- candidate semantic step exit: **NOT RUN**
- complete nine-scenario candidate set: **FAIL, missing**
- semantic composite gate: **FAIL**

This failure occurs on the native sender and therefore is not attributed to
MAT-26. It is carried for reviewer disposition as an independent current-path
bounded-read regression or harness-invariant issue.

## Timing suite

`run/suite-manifest.json` records runner SHA `b834b7fb...`, control A
`succeeded`, candidate `failed` with exit 1, and both control B and profiled
control `pending`. No `results.jsonl` was emitted because the suite stopped at
the failed candidate step.

Control A built one fresh 100k physical generation and passed topology,
serving, remote-owner, generation, recall, storage, and release-profile checks.
Its frozen generation identity is
`020049f843b8268155693f45158758d8e9883d946447c510de1f5b378cdc1fbdbebd`.
Raw control values, retained only as context:

| Variant | mean | p95 | p99 | recall |
| --- | ---: | ---: | ---: | ---: |
| eager control | 44.6 ms | 55.6 ms | 59.6 ms | 0.9285 |
| production lazy-10 | 27.7 ms | 30.9 ms | 32.8 ms | 0.9285 |

The candidate reused that exact fixture and passed the unanimous release
preflight. Its eager recall predictions are byte-identical to both control-A
prediction files; all three SHA-256 values are
`3dbd83a5591960affef89f3225d5a650f7c3fdaa2f7af325be98fe24cd3701ae`.
Its raw eager timing was mean 26.3 ms, p95 29.8 ms, p99 31.6 ms, but it is not a
decision-bearing candidate result because activation is unobservable and the
candidate production lazy-10 and control-B arms never ran. Control A eager was
44.6 ms on the same frozen generation. The arms report the same owner/payload
workload, but differ in two uncontrolled ways: fixture position/warmth and the
fast-sender flag whose effect is unobservable. Their 41% gap—roughly eight
times the registered 5% usefulness threshold—is therefore unattributable in
either direction: it is neither a candidate win nor evidence that the sender
did nothing. Control B was the term intended to separate those effects, and it
never ran.

The activation assertion was structurally unsatisfiable:

1. the CLI requires `owner_fast_real_array_send` together with
   `skip_owner_locality_profile`;
2. `generation_read.rs` makes the fast sender and locality profiling mutually
   exclusive;
3. without the locality profile, `tid_profile` is empty and
   `owner_requested_tids` remains zero;
4. `remote_transport.rs` records projected values, binary-send bytes, fast
   values, fallbacks, and ineligible requests only inside
   `if owner_requested_tids > 0`; and
5. the CLI then requires the first three exported values to be nonzero.

The candidate did reach owner payload work—400 remote owners, 6,328 remote
candidates and installed payloads, 12,656 payload columns, and 77,960,960
payload bytes—but the sender may or may not have activated. The available
telemetry cannot distinguish those cases.

The candidate eager latency emitted 200 scans and these registered outcome
metrics:

| Metric | Value | Gate |
| --- | ---: | --- |
| `owner_projected_values` | 0 | **UNOBSERVABLE: coordinator suppressed** |
| `owner_binary_send_bytes` | 0 | **UNOBSERVABLE: coordinator suppressed** |
| `owner_fast_real_array_values` | 0 | **UNOBSERVABLE: coordinator suppressed** |
| `owner_fast_real_array_fallback_values` | 0 | **VACUOUS: coordinator suppressed** |
| `owner_fast_real_array_ineligible_requests` | 0 | **VACUOUS: coordinator suppressed** |

Source:
`run/candidate/physical-eager-control-vector_bearing-latency.log`. The harness
stopped on the first failed item with `Task 224 MAT-26 candidate produced zero
owner_projected_values`; source:
`run/candidate/distann-local-multinode.log`.

## Gate accounting

| Preregistered term | Result |
| --- | --- |
| exact release SHA/profile/features | **PASS** |
| timing same-generation reuse | **PASS** through candidate fixture decision |
| semantic control exit + exact nine-set | **FAIL** |
| semantic candidate exit + exact nine-set | **FAIL / NOT RUN** |
| candidate fast values >0 | **UNOBSERVABLE**: activation assertion is structurally unsatisfiable |
| candidate fallback =0 | **VACUOUS / UNOBSERVABLE**: exported default zero |
| candidate ineligible =0 | **VACUOUS / UNOBSERVABLE**: exported default zero |
| eager control/candidate prediction identity | **PASS**, byte-identical |
| lazy-10 control/candidate prediction identity | **FAIL / candidate missing** |
| `C`, control envelope `N`, >=5%, and >=`2*N` | **NOT COMPUTABLE**: control B and candidate lazy-10 missing |
| p95/p99 <= control mean +5% | **NOT COMPUTABLE** |
| scan-count equality for attribution rows | **NOT COMPUTABLE** |
| finite positive `R` in `(0,1]` | **NOT COMPUTABLE** |
| `D_attr > 0` and >=50% of end-to-end saving | **NOT COMPUTABLE** |

The preregistration requires every gate to pass and does not permit post-hoc
normalization or a replacement run. The semantic composite failed, and every
candidate usefulness, tail, and attribution term is unavailable. The
activation assertion itself cannot classify the candidate. Those facts support
STOP, while leaving MAT-26's latency effect explicitly unmeasured.

## Decision and carry-forward

1. **STOP MAT-26 with its latency effect unmeasured.** The candidate axis was
   void; Task 224 has no finalist and will not run packet 004.
2. Bar the raw eager 26.3 ms context from any candidate claim. The 44.6→26.3
   ms arms differ in both fixture position/warmth and an unobservable sender
   flag, so the 41% gap is unattributable in either direction. Its magnitude is
   about eight times the decision threshold, and control B never separated the
   effects. Tasks 229--233 must compare arms at matched fixture position (or a
   preregistered counterbalanced envelope), never a fresh-build control against
   a reused candidate.
3. Retain the feature-only, default-off candidate and instrumentation as
   diagnostic code. Production behavior remains unchanged.
4. Carry the independent native semantic control's 12/10 bounded-read
   divergence to Task 239 for exact-current-main reproduction, diagnosis, and
   fix or evidence-based invariant disposition. Do not blindly widen the bound.
5. Task 225 remains conditional on its own finalist-stability and hideable-RTT
   premise; Task 224 neither satisfies nor rejects that premise. Task 229 is the
   next mandatory prototype, with Task 239 required before its semantic
   closeout.
