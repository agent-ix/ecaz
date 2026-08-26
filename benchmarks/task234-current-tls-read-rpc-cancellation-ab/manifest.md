# Task 234 current-TLS read RPC cancellation A/B

Date: 2026-08-25 (America/Los_Angeles)

## Provenance

- Control extension SHA: `ed5ac814c05350ca695533fcd54d0df11faa876b`
  (the exact Task 238/236 campaign integration head before Task 234).
- Candidate extension SHA: `7c42dc818e80fe68246dc5c45255640b81c551b1`.
  The production runtime delta is the Task 234 read-RPC deadline,
  cancellation, typed-error, and all-success normalization work. The candidate
  also contains the `ecaz-cli` Task 234 fault lane and this preregistered suite;
  neither is used by the extension serving path measured here.
- Host/toolchain: local Intel development host, PostgreSQL 18 release extension,
  `/home/peter/.ecaz/toolchains/pg18-ssl/bin`.
- Transport: fresh three-owner physical fixtures using verify-full mutual TLS
  (`--secure-remote-transport`). Each scale and arm built a fresh, isolated
  one-index-per-owner fixture; no shared-table A/B surface was used.
- Corpus: staged `ec_real_10k`, `ec_real_50k`, and `ec_real_100k` under
  `data/staged-current/`. Corpus/query data and stopped PGDATA directories are
  intentionally not committed.
- Fixture: graph degree 32, persisted 4,096-row head sample, head search width
  32, beam width 4, candidate heap 32, 100 hop rounds, 200 recall queries at
  top-10, and 50 warm latency samples after 10 warmups.
- Run order: all control steps, then all candidate steps. Latency deltas are
  single-run observations and can include order/cache noise; recall, prediction,
  head-membership, and storage artifacts provide the deterministic checks.

## Commands

The full expanded per-step commands are recorded in the suite manifests.

```text
/home/peter/.cargo-target/debug/ecaz bench suite audit \
  --config benchmarks/task234-current-tls-read-rpc-cancellation-ab/suite.json \
  --log-file benchmarks/task234-current-tls-read-rpc-cancellation-ab/artifacts/suite-audit.log

/home/peter/.cargo-target/debug/ecaz bench suite run \
  --config benchmarks/task234-current-tls-read-rpc-cancellation-ab/suite.json \
  --only-tag control \
  --artifact-dir benchmarks/task234-current-tls-read-rpc-cancellation-ab/artifacts/control \
  --manifest-output benchmarks/task234-current-tls-read-rpc-cancellation-ab/artifacts/suite-manifest-control.json \
  --results-output benchmarks/task234-current-tls-read-rpc-cancellation-ab/artifacts/suite-results-control.jsonl \
  --log-file benchmarks/task234-current-tls-read-rpc-cancellation-ab/artifacts/suite-control.log

/home/peter/.cargo-target/debug/ecaz bench suite run \
  --config benchmarks/task234-current-tls-read-rpc-cancellation-ab/suite.json \
  --only-tag candidate \
  --artifact-dir benchmarks/task234-current-tls-read-rpc-cancellation-ab/artifacts/candidate \
  --manifest-output benchmarks/task234-current-tls-read-rpc-cancellation-ab/artifacts/suite-manifest-candidate.json \
  --results-output benchmarks/task234-current-tls-read-rpc-cancellation-ab/artifacts/suite-results-candidate.jsonl \
  --log-file benchmarks/task234-current-tls-read-rpc-cancellation-ab/artifacts/suite-candidate.log
```

Audit passed all six configured steps. Both selective runs report three
succeeded, three intentionally skipped, zero failed, zero missing artifacts,
and zero stale artifacts.

## Results

Latency values are milliseconds. Storage is physical generation bytes / total
owner graph-side bytes. A negative latency delta favors the candidate.

| Scale | Arm | Recall@10 | Mean | p50 | p95 | p99 | Storage |
|---|---|---:|---:|---:|---:|---:|---:|
| 10k | control | 0.9990 | 7.03 | 6.86 | 7.87 | 10.10 | 242,860,032 / 76,046,336 |
| 10k | candidate | 0.9990 | 6.76 | 6.78 | 7.55 | 7.66 | 242,860,032 / 76,046,336 |
| 10k | delta | 0.0000 | -3.84% | -1.17% | -4.07% | -24.16% | 0 / 0 |
| 50k | control | 0.9540 | 8.16 | 8.07 | 9.32 | 10.20 | 1,243,512,832 / 410,189,824 |
| 50k | candidate | 0.9540 | 7.64 | 7.63 | 9.00 | 9.18 | 1,243,488,256 / 410,181,632 |
| 50k | delta | 0.0000 | -6.37% | -5.45% | -3.43% | -10.00% | -24,576 / -8,192 |
| 100k | control | 0.9295 | 8.40 | 8.13 | 10.20 | 11.10 | 2,498,215,936 / 831,750,144 |
| 100k | candidate | 0.9290 | 8.02 | 7.98 | 9.52 | 9.82 | 2,498,215,936 / 831,758,336 |
| 100k | delta | -0.0005 | -4.52% | -1.85% | -6.67% | -11.53% | 0 / +8,192 |

The 10k and 50k prediction files are byte-identical between arms. At 100k,
one query replaces result id `73101` with `63646` at the top-10 boundary,
accounting for the one-hit-in-2,000 recall difference. Head-membership files
and their embedded digests are identical at every scale. The Task 234 code
does not change graph construction or score computation, so this isolated
boundary replacement is treated as run-order/tie nondeterminism rather than a
material recall regression. Storage differs by at most one 8 KiB page on the
graph side at 50k/100k and is operationally neutral.

Disposition: the current-TLS candidate reverses the historical pre-TLS
negative latency signal recorded in the Task 234 ledger. On this current-base
run it is recall/storage neutral within measurement resolution and faster in
all recorded warm latency percentiles. Accept the implementation for outside
review; do not treat these single sequential latency runs as a general-purpose
performance claim.

## Reviewer-required 50k reuse drift bound (2026-08-26)

Packet 004 feedback accepted every gate except the latency disposition and
requested one `reuse_fixture: true` 50k repeat for each arm. The original
fixtures had already been removed, so each arm was rebuilt once and then
repeated immediately against that exact stopped fixture. No 10k/100k fixture
or fault matrix was rerun.

Secure fixture reuse required two narrow runner corrections. The fixture now
reloads its existing TLS artifacts and role instead of replacing them, and the
documented `skip_fault_drills` path skips the mutating routed DELETE/VACUUM
lifecycle drill before shutdown. The reuse validator remains strict: both
repeat steps attested 50,000 source rows, the exact extension SHA and release
profile, the query slice, and `persisted_head` before measuring.

| Arm | Observation | Mean | p50 | p95 | p99 | Same-fixture mean shift |
|---|---|---:|---:|---:|---:|---:|
| control | fresh seed | 8.83 | 8.57 | 10.90 | 11.30 | — |
| control | reuse repeat | 8.47 | 8.36 | 10.40 | 10.60 | -4.08% |
| candidate | fresh seed | 9.43 | 9.30 | 10.90 | 11.10 | — |
| candidate | reuse repeat | 8.74 | 8.53 | 10.50 | 10.80 | -7.32% |
| repeat delta | candidate vs control | +3.19% | +2.03% | +0.96% | +1.89% | — |

The old 50k observations were 8.16 ms control and 7.64 ms candidate. Relative
to that old control, today's control seed/repeat are +8.21%/+3.80%; within the
new fixture, control itself shifts -4.08% on immediate reuse. The resulting
run-to-run band covers the historical screened +7.1% signal. The candidate's
repeat-only +3.19% mean delta is also inside the directly observed drift band,
so the evidence supports no resolved latency regression rather than a speedup
claim. Final ACCEPT/REJECT disposition remains with the outside reviewer.

The targeted commands were:

```text
/home/peter/.cargo-target/debug/ecaz bench suite run \
  --config benchmarks/task234-current-tls-read-rpc-cancellation-ab/suite.json \
  --only control-50k-drift-seed --only control-50k-drift-repeat \
  --manifest-output benchmarks/task234-current-tls-read-rpc-cancellation-ab/artifacts/suite-manifest-control-drift-v2.json \
  --results-output benchmarks/task234-current-tls-read-rpc-cancellation-ab/artifacts/suite-results-control-drift-v2.jsonl \
  --log-file benchmarks/task234-current-tls-read-rpc-cancellation-ab/artifacts/suite-control-drift-v2.log

/home/peter/.cargo-target/debug/ecaz bench suite run \
  --config benchmarks/task234-current-tls-read-rpc-cancellation-ab/suite.json \
  --only candidate-50k-drift-seed --only candidate-50k-drift-repeat \
  --manifest-output benchmarks/task234-current-tls-read-rpc-cancellation-ab/artifacts/suite-manifest-candidate-drift-v2.json \
  --results-output benchmarks/task234-current-tls-read-rpc-cancellation-ab/artifacts/suite-results-candidate-drift-v2.jsonl \
  --log-file benchmarks/task234-current-tls-read-rpc-cancellation-ab/artifacts/suite-candidate-drift-v2.log
```

## Durable artifacts

- `suite.json` — preregistered six-step config; SHA-256
  `7ac4ab819d421245c3d3a6cedd4e91fc811ca4927f6433e90d09cbc30c2014e1`.
- `artifacts/suite-manifest-control.json` — completed control manifest;
  SHA-256 `6a908a5cead9d9d5facc0a35651e4ce0f17eec5add861814e56f557d6d12120d`.
- `artifacts/suite-manifest-candidate.json` — completed candidate manifest;
  SHA-256 `23086f4b1c984b706b5deedc1c1e89f2c4471c877c5e19b8dfca4ba062241ab6`.
- `artifacts/suite-results-control.jsonl` — normalized control rows;
  SHA-256 `31f4e73925ff6ded099c0bbeb42a576ba50d1a9b902e1774f5ca0145ab02e99a`.
- `artifacts/suite-results-candidate.jsonl` — normalized candidate rows;
  SHA-256 `e70cc4d48bad9d30d0e2d9fc12d25d0010c9b679f77c0521d28f216135135309`.
- `artifacts/{control,candidate}/<arm>-<scale>/` — compact per-step summary,
  recall, latency, predictions, head membership, and PostgreSQL diagnostic
  logs. The suite results JSONL and manifests are the machine-readable source
  of truth; per-step summaries contain the cited result lines.
- `suite.json` after the reviewer follow-up — ten-step config; SHA-256
  `c8b5f71eb677820deff07dffe445c97a8f959710aa2b8836dd36da5c48018812`.
- `artifacts/suite-manifest-control-drift-v2.json` and
  `artifacts/suite-results-control-drift-v2.jsonl` — completed control
  seed/reuse run; SHA-256 `157576a702eb9a7b46b151dcfa8b1788b4d2715216affad7df22312e00ba618f`
  and `deae038ea100e78065febc1a67cb2dcd38f6b0e6799cf3bd09e8169f5bd353dc`.
- `artifacts/suite-manifest-candidate-drift-v2.json` and
  `artifacts/suite-results-candidate-drift-v2.jsonl` — completed candidate
  seed/reuse run; SHA-256 `500851cbff6cc7217456c4e8dd59828cce2dbd3afb28773564d1d60d91228156`
  and `d28d13bcab5197a568e9beace28dbe8594e8fcf6769eda7767746a3915d88e3d`.
- `artifacts/run/{control,candidate}-50k-drift-{seed,repeat}/` — the four
  compact summaries and latency logs cited above. Fixtures remained isolated
  one-index-per-owner; PGDATA and corpus data are not committed.
