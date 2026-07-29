# Review Request: systemd-Scoped cgroup OOM Drill

## Summary

This Task 38 checkpoint replaces the host-independent `cgroup-plan`
placeholder with an executable cgroup-v2 OOM operator covering all seven fault
fixtures: HNSW, IVF, DiskANN, SPIRE, and the RaBitQ, TurboQuant, and grouped-PQ
DistANN shapes.

`ecaz dev fault cgroup-smoke` runs each fixture in a separate user
`systemd-run --scope` with `MemoryMax` and `OOMPolicy=kill`. The constrained
worker initializes and starts a fresh PG18 cluster inside the scope, prepares
the selected fixture with a committed valid index, and starts an observed
active loop of real AM DROP/CREATE index builds. It then grows touched resident
memory in 8 MiB chunks until the cgroup OOM killer fires.

The outer operator remains outside the constrained scope and requires:

1. a worker marker emitted only after `pg_stat_activity` observes the repeated
   AM-build query active;
2. non-successful scope termination with systemd `Result=oom-kill`;
3. the constrained postmaster to be gone;
4. successful crash-recovery startup outside the scope;
5. a usable SQL session, the exact expected fixture row count, and the expected
   valid/ready ECAZ index;
6. a successful forced AM scan and the shared leak/lock/pin postconditions; and
7. a clean post-recovery stop.

Only small `scope.log`, `recovery.log`, and `postgres.log` evidence belongs
under the configured artifact directory. PostgreSQL data and socket files live
under a separate target-local runtime directory and are removed only after a
successful recovery probe, preventing cluster data from entering review
packets.

Equal or ancestor/descendant artifact/runtime roots are rejected. Runtime
roots inside `reviews/` or `benchmarks/` are also rejected before directory
creation, and the canonicalized roots are checked again after creation.

`make fault-cgroup-smoke` exposes the full seven-fixture lane.

## Validation

See `artifacts/manifest.md` and `artifacts/local-validation.log`.

- modified Rust files pass stable `rustfmt --check`;
- `git diff --check` passed;
- `cargo check -p ecaz-cli` passed;
- the rebuilt CLI parsed and dispatched `cgroup-smoke`, then failed closed on
  macOS with `cgroup smoke requires Linux` before creating artifact/runtime
  directories;
- the repository-wide formatter check remains blocked by unrelated existing
  formatting drift and did not modify those files.

This macOS arm64 host has no cgroup v2 or systemd user manager. This packet
therefore proves the compiled operator and host gate, not a live OOM event. A
supported Linux host must still execute the seven cases and retain the
packet-local logs before Task 38 closeout.

## 2026-07-26 Outside-Review Response

Follow-up implementation `631ec2940` addresses all three findings in
`feedback/2026-07-26-01-reviewer.md`:

- the worker commits the initial AM index before the interrupted build loop;
- recovery requires that exact index to exist and be valid/ready, then executes
  the forced fixture AM scan and shared cleanup postconditions;
- the parent passes the expected row count and recovery requires equality; and
- resolved artifact/runtime roots must be disjoint, with runtime forbidden
  beneath `reviews/` or `benchmarks/`.

See `artifacts/2026-07-26-review-response-validation.log`.

## Reviewer Focus

- Does the outer-process/inner-scope split prove recovery without allowing the
  verifier itself to be killed?
- Is an observed active repeated AM-build loop plus cgroup-local resident
  pressure a sound way to force an OOM during real AM work?
- Are `OOMPolicy=kill` and required `Result=oom-kill` sufficiently strong
  evidence that the entire postmaster/workload group followed the kernel OOM
  path?
- Do exact rows, the expected valid/ready index, a forced AM scan, and shared
  cleanup postconditions establish recovery for this checkpoint?
- Is the evidence/runtime separation sufficient to prevent PostgreSQL data
  directories from entering review packets?

## Remaining Task 38 Work

- execute the seven cgroup OOM cases on a supported Linux host;
- execute DistANN and SPIRE remote reset/slow modes on Linux;
- execute provider-backed DistANN local EIO/ENOSPC/slow-disk;
- retain all live packet-local marker, systemd, recovery, and provider evidence;
- obtain reviewer re-verification of the addressed packet 004 findings.
