# Review Request: Task 38 DistANN and Remote Fault Expansion

## Summary

This first Task 38 checkpoint adds `ec_distann` to the fault model without
aliasing it to `ec_diskann`. The model now has seven concrete fixtures:
`ec_hnsw`, `ec_ivf`, `ec_diskann`, `ec_spire`, and codec-specific
`ec_distann` fixtures for RaBitQ, TurboQuant, and grouped PQ.

`ecaz dev fault plan`, `prepare`, and `smoke` accept `--am distann` and an
optional `--distann-codec`. An unqualified DistANN selection expands to all
three codecs; aggregate selection expands to the four existing fixtures plus
all three DistANN fixtures. Each DistANN fixture has a distinct table/index
name and uses real local `ec_distann` DDL with the selected
`neighbor_code_format`.

The codec fixtures use dimensions supported by their real scorers:
TurboQuant uses the 1536-D no-QJL 4-bit lane, while RaBitQ and grouped PQ use
64-D deterministic vectors. DistANN now also participates in the existing
test-controlled palloc sweep at build, scan, insert, bulk-delete, and vacuum
callback boundaries.

The local PG18 fixture now has live evidence for all three codecs:

- `pg_cancel_backend` and `pg_terminate_backend` during repeated DistANN KNN
  work;
- statement and idle-in-transaction timeout;
- lock timeout across concurrent reindex, create-index, and vacuum-full;
- accumulator pressure, temp-file failure/accounting, AM-backed WAL rotation,
  insert/vacuum, and shared cleanup checks.

The initial exact accumulator assertion exposed a real approximate-result
boundary: TurboQuant returned 999 of the requested 1000 candidates. The final
gate records actual high-water and returned fraction and requires at least 95%
of target. The rerun passed at 100%, 99.9%, and 100% for RaBitQ, TurboQuant,
and grouped PQ respectively.

The provider foundation now has two socket-only modes:
`socket-reset` returns `ECONNRESET` and shuts down the matched connection;
`socket-slow` applies bounded latency. Both require an exact peer identity
(`tcp:HOST:PORT`, bracketed IPv6, or absolute named `unix:/path`) resolved
with `getpeername(2)`. Unnamed and abstract Unix peers never match, preventing
an empty `unix:` selector from aliasing unrelated accepted connections. The
provider covers scalar, vectored, datagram, and message socket I/O entry
points; file-provider matching remains separate.
Provider restore removes the peer filter along with the existing LD_PRELOAD
variables. Linux builds now enforce `-Wall -Wextra -Werror` for the provider.

The Linux LD_PRELOAD provider compiled successfully on Ubuntu 24.04 x86_64 in
the manually dispatched PG18 job for this PR head. That job does not load or
exercise the provider. This macOS/aarch64 host cannot build or execute it
and has neither cgroup v2 nor `systemd-run`. Consequently local provider EIO,
ENOSPC, measured slow-disk, SPIRE/DistANN exact-peer socket faults, and cgroup
OOM remain explicitly unavailable. The socket provider and cgroup operator
plans are validated, but no `fault=1` or live cgroup pass is claimed.

## Historical Context

The prior four-AM smoke implementation and reviewer cycles remain in
`reviews/task-36/001-31145-task36-38-hardening-validation/`. In particular,
the final review and its corrected split between live SPIRE SQL transport and
nonexistent object-store reads are background for the next socket-provider
slice.

## Validation

See `artifacts/manifest.md` and the packet-local logs:

- `cargo test -p ecaz-fault-injection`
- `cargo check -p ecaz-cli`
- focused `ecaz-cli` DistANN fault parsing test
- focused socket-provider CLI parsing test
- exact-peer socket-provider environment dry run
- focused DistANN cancel-plan dry run
- focused grouped-PQ timeout dry run
- full seven-fixture fault plan and `make fault-full` dry-run
- live all-codec DistANN cancel/terminate, timeout, lock-timeout, and resource
  lanes
- cgroup host-capability plan
- final PG18 recovery and cleanup SQL

`cargo fmt --all -- --check` was run and recorded. It fails on the refreshed
upstream base across many untouched files; the packet retains that output.
`rustfmt --check` passes for the modified Rust files.

## Reviewer Focus

- Does the seven-fixture model make DistANN codec coverage explicit enough to
  prevent silent single-codec passes?
- Are the codec-specific relation names and local `ec_distann` DDL an
  appropriate foundation for the live fault lanes?
- Are 1536-D TurboQuant and 64-D RaBitQ/grouped-PQ deterministic fixtures the
  right small supported dimensions for interruption smoke?
- Are the DistANN palloc hooks narrow and safely disabled by the existing
  production-default-off GUC?
- Is rejecting `--distann-codec` outside `--am distann` the right operator
  contract?
- Does exact named-Unix/TCP `getpeername(2)` matching adequately isolate SPIRE
  SQL and DistANN loopback-TCP transport faults from control traffic?
- Is the quantified 95% accumulator-pressure gate strong enough for
  approximate AMs while avoiding the historical weak `count >= 64` check?
- Are the Linux provider and cgroup deferrals stated narrowly enough to keep
  Task 38 open without confusing unavailable host capability with nonexistent
  SPIRE object-store functionality?

## Open Linux Evidence

Task 38 remains open. A Linux session must run a focused TCP `socket-reset`
case against a real SPIRE SQL or DistANN owner/payload peer and trace the
network syscalls used by that workload. The same host must still capture live
provider-backed DistANN EIO/ENOSPC/slow-disk evidence and the cgroup-v2 OOM
lane before task closeout.
