# Task 38 Completion Audit — Apple M5 Handoff

This audit tests the current branch against the full Task 38 handoff objective.
“Complete on M5” and “Task 38 closed” are separate conclusions.

| Requirement | Authoritative evidence | Assessment |
|---|---|---|
| Add DistANN as the fifth AM | `crates/ecaz-fault-injection/src/lib.rs`, `crates/ecaz-cli/src/commands/dev/fault.rs`, packet 001 | Complete. DistANN is not aliased to DiskANN and expands into codec-specific fixtures. |
| Cover RaBitQ, TurboQuant, and grouped-PQ DistANN workloads | Packet 001 request, manifest, live PG18 logs, and final reviewer approval | Complete on M5. All three have distinct relations/indexes and supported dimensions; local cancel, timeout, lock, resource, and cleanup lanes are evidenced. |
| Add real DistANN remote socket faulting | Provider interposers plus `distann_multicluster.rs`; packets 001/002 and seq-02 approval | Implementation complete and source-approved. Live Linux TCP reset/slow marker and syscall evidence is missing. |
| Add real SPIRE remote socket faulting | Provider interposers plus `spire_multicluster.rs`; packet 003 and seq-03 approval | Implementation complete and source-approved. The runner validates a healthy baseline, exact fault marker, stable slow result, and post-disarm recovery. Live Linux named-Unix reset/slow execution is missing. |
| Close historical cleanup nits | Packet 001 seq-02 approval; provider `-Werror`, full scalar/vectored/message interposers, named-Unix guard, errno preservation, typed DistANN fixture enum | Complete. No historical source-review finding remains. |
| Add cgroup OOM locally where supported | `fault.rs`, `Makefile`, `docs/hardening.md`; packet 004 seq-02 approval | Operator complete and source-approved. Apple M5 does not provide Linux cgroup v2 or a user systemd manager, so no local OOM event can be claimed. |
| Produce `reviews/task-38/001-...` | Packets 001 through 005 | Complete. Packet ordering and task-scoped storage follow repository policy. |
| Obtain outside review | Packet-local feedback for 001–004 | Complete for the implementation/source checkpoints. Review explicitly does not approve missing runtime behavior. |
| Preserve durable evidence discipline | Packet-local requests, manifests, logs, and feedback | Complete for current evidence. No corpus, raw SSM output, polling cruft, or runtime cluster data is committed. |
| Close Task 38 | Task exit criteria and live-evidence requirements | Not complete. Intel/Linux provider-backed file faults, both live socket modes, and seven cgroup OOM cases remain missing. |

## Final M5 Validation

- Final SPIRE response code: `aea65a78f`.
- Packet response: `85f66c18b`.
- Outside approval: `c9b8fddad`.
- `cargo check -p ecaz-cli --tests`: pass in 12m01s.
- Focused monolithic CLI unit test: stopped during link/codegen; no result
  claimed.
- Canonical status update: `f18e41e85`.

## Hold-Open Matrix

| Missing evidence | Required host capability | Closeout evidence |
|---|---|---|
| DistANN EIO/ENOSPC/slow-disk | designated Intel/Linux host with LD_PRELOAD and PG18 | Packet-local provider marker, workload result, recovery/postconditions, and measured slow baseline delta |
| DistANN socket reset/slow | same host, real owner/payload TCP peer | Exact-peer marker, network syscall trace, clean reset outcome or baseline-plus-latency result, and exact-source recovery |
| SPIRE socket reset/slow | same host, real participant named-Unix peer | Exact-peer marker, network syscall trace, accepted reset result, healthy stable slow result, and healthy stable recovery |
| Seven cgroup OOM fixtures | Linux cgroup v2 plus working `systemd-run --user --scope` | Per-fixture `Result=oom-kill`, dead scoped postmaster, exact rows, valid/ready index, forced AM scan, and shared cleanup postconditions |

## Conclusion

The Apple-M5-supported Task 38 work is complete and outside-reviewed. The full
task remains open, without a closeout or promotion claim, until the designated
Intel/Linux execution evidence lands.

