# Task 224 packet 003 live-screen decision

Disposition: **FAIL CLOSED — STOP MAT-26 as implemented. Do not advance to
packet 004 or a 10k/50k/100k release matrix.**

This is a preregistered screen failure, not evidence that the exact sender is
slower. Both live suites stopped before they could produce a complete candidate
comparison. Production remains unchanged; the candidate is feature-only and
default-off.

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
decision-bearing candidate result because the activation gate failed and the
candidate production lazy-10 arm never ran.

The candidate eager latency emitted 200 scans and these registered outcome
metrics:

| Metric | Value | Gate |
| --- | ---: | --- |
| `owner_projected_values` | 0 | **FAIL: required >0** |
| `owner_binary_send_bytes` | 0 | **FAIL: required >0** |
| `owner_fast_real_array_values` | 0 | **FAIL: required >0** |
| `owner_fast_real_array_fallback_values` | 0 | pass: required 0 |
| `owner_fast_real_array_ineligible_requests` | 0 | pass: required 0 |

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
| candidate fast values >0 | **FAIL**, value 0 |
| candidate fallback =0 | pass in the only executed candidate latency arm |
| candidate ineligible =0 | pass in the only executed candidate latency arm |
| eager control/candidate prediction identity | **PASS**, byte-identical |
| lazy-10 control/candidate prediction identity | **FAIL / candidate missing** |
| `C`, control envelope `N`, >=5%, and >=`2*N` | **NOT COMPUTABLE**: control B and candidate lazy-10 missing |
| p95/p99 <= control mean +5% | **NOT COMPUTABLE** |
| scan-count equality for attribution rows | **NOT COMPUTABLE** |
| finite positive `R` in `(0,1]` | **NOT COMPUTABLE** |
| `D_attr > 0` and >=50% of end-to-end saving | **NOT COMPUTABLE** |

The preregistration says a failed activation/precondition fails the gate rather
than permitting post-hoc normalization or a rerun, and that Task 224 STOPs
unless every gate passes. Multiple independent gates failed; the missing terms
do not create ambiguity or authorize a replacement measurement.

## Decision and carry-forward

1. **STOP MAT-26 as implemented.** Do not run packet 004 or productionize the
   exact sender.
2. Do not interpret the raw eager 26.3 ms context as a candidate win: control B
   was intentionally required to bound control A's build/position warmth and
   never ran, while activation was zero.
3. Production behavior is unchanged. Outside review must rule whether the
   feature-only candidate/instrumentation should remain as diagnostic code or
   be removed before merge.
4. Carry the native semantic control's 12/10 bounded-read failure explicitly;
   outside review must rule whether it needs a separate correctness/harness
   follow-up. It does not reopen or rescue MAT-26.
5. Task 225 has no Task 224 finalist from this screen; its conditional entry
   remains unsatisfied unless separately justified by its own measured premise.
