# Task 239 packet 003 live-run decision

## Disposition

**HARNESS REGRESSION CORRECTED; EXACT-MAIN LAZY-10 SEMANTIC PATH RESTORED
TO 10/10.**

The sole authorized packet-003 invocation completed successfully. The result
classifies packet 001's 12/10 semantic observation as a shared-session harness
GUC leak, not production bounded-read overfetch. With every arm explicitly set
and attested, the exact-main production lazy-10 path requested six remote rows,
consumed four local rows, and performed exactly ten payload reads for ten
returned rows. No production extension source changed, so the repository's
10k/50k/100k runtime-change closeout matrix is not triggered.

Task 239 remains review-open until an outside reviewer accepts this result and
the synchronized task/index/roadmap closeout.

## Frozen run identity

- Invocation count: exactly one non-dry suite invocation; no continuation,
  resume, selected-step execution, retry, or replacement run.
- Detached runner/extension checkpoint:
  `4ab2aa9a90f14b045298ac9fe408f9a4b586bf3c`.
- Immediately pre-run runner binary SHA-256:
  `ce5bfeb1ea486c2fbed3027a703bba49122ac5123f46672cd0aaf1b4b0eb5163`.
- Config SHA-256:
  `53e13d779e2452a4282f8a076c17eb082396df615efd3e45393d2054257a4532`.
- Extension preflight: release, unanimous across three nodes, features exactly
  `distann-head-attribution-benchmark,pg18`, `debug_override=false`.
- Suite manifest: `dry_run=false`, one selected step, `status=succeeded`, exit
  code 0.

The pre-run identity and free-port checks are recorded in
`pre-run-attestation.log`; the exact invocation and suite outcome are in
`live-suite.log` and `live-run/suite-manifest.json`.

## Fixed-gate adjudication

Both the main multinode log and compact summary contain exactly one row for
each of the nine required scenarios. No Task 167 quality-gate skip and no
semantic `pass=false` row appears.

| Scenario | Result | Remote | Local | Reads/bound | Digest |
|---|---:|---:|---:|---:|---|
| `fewer_than_window` | 5 rows | 6 | 2 | 8/10 | `08efa609...d2017dfc` |
| `exactly_one_window` | 10 rows | 6 | 4 | 10/10 | `df979e2d...6cfc77d` |
| `more_than_window` | 15 rows | 10 | 5 | 15/20 | `82f675ad...38e4013` |
| `reject_first_window` | 10 rows | 12 | 8 | 20/1024 | `3568e635...eb44755` |
| `reject_multiple_windows` | 10 rows | 33 | 15 | 48/1024 | `f4aeaffe...f73c` |
| `null_payload` | 10 rows | 12 | 8 | 20/1024 | `fef30f6c...f8fb8` |
| `toasted_projection_qual` | 10 rows | 39 | 19 | 58/1024 | `001d1fa6...e878d4` |
| `mixed_local_remote` | 10 rows | 6 consumed | 4 | n/a | pass |
| `post_first_batch_remote_failure` | first 10 rows | 6 requested | n/a | n/a | `bd4f381d...9c99e02f` error |

All seven core rows report `control_batch_size=0`,
`candidate_batch_size=10`, eager/candidate identity, and zero duplicates.
Mixed-owner consumption sums to ten with zero duplicates. The outage arm emits
its first ten rows and then fails closed with the fixed error identity.

Both recall children completed over 200 queries / 2,000 trials at 0.9990.
Their prediction files are byte-identical to each other and to packet 001:

```text
801f6a0b83237047fea6ebd92cb1b85f07aa8dd80ee6dbd5c7877153e724fb6e
```

The routed DELETE+VACUUM drill also reports `pass=true`. One-iteration latency
and storage rows remain diagnostic context and have no decision weight.

## C5 cleanup

The stopped external fixture occupied 1.2 GB. After all evidence was captured,
`/home/peter/.ecaz/clusters/task239-main-port-semantics-10k` was removed and
confirmed absent. The cluster state is not review evidence and is recoverable
only by regeneration; no packet artifacts were removed.
