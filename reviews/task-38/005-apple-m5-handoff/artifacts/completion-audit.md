# Task 38 Completion Audit — Apple M5 Handoff

This audit tests the current branch against the full Task 38 handoff objective
and every canonical validation/exit criterion. Source implementation,
M5-executed behavior, and Task 38 closeout are separate conclusions.

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

## Canonical Validation And Exit Criteria

| Canonical criterion | Authoritative evidence | Assessment |
|---|---|---|
| Every applicable lane completes without postmaster `PANIC` or leaked buffers/locks | Historical four-AM live packet `reviews/task-36/001-31145-task36-38-hardening-validation/`; packet 001 live DistANN cancel/timeout/lock/resource logs; packet 001 seq-01 review confirms zero sessions, locks, prepared transactions, and fixture pins | Partially complete. Historical four-AM lanes and the M5-executed DistANN lanes have clean postconditions. Provider-backed DistANN file faults, both remote socket modes, and all cgroup cases have no live result, so the criterion is not established across every applicable lane. |
| A deliberately introduced invalid-buffer-style bug is caught by the cancellation lane | No durable Task 38 mutation-control artifact found. The historical deliberate score perturbation belongs to SIMD differential validation and is not evidence for cancellation. | Unverified and M5-verifiable. A temporary controlled mutation plus live cancellation run must show that the lane fails for the intended reason, then the mutation must be removed. |
| Resource exhaustion catches an accumulator/palloc failure without recovery | Packet 001 `distann-all-codecs-resource-live.log` proves 1000/999/1000 accumulator high-water under constrained settings; historical packet proves four-AM resource and palloc behavior; current DistANN palloc sites are source-reviewed | Partially complete. M5 evidence proves DistANN accumulator pressure and recovery, but no durable five-AM/three-codec negative control demonstrates detection of an unrecovered palloc failure. |
| All five AMs and all three DistANN codecs survive applicable smoke lanes | Historical four-AM live evidence plus packet 001 live M5 DistANN codec logs | Partially complete. Local cancel/timeout/lock/resource coverage is established for all three DistANN codecs and historical local coverage exists for the original four AMs. The missing provider/socket/cgroup rows prevent full survival coverage. |
| Document every long-running-loop `CHECK_FOR_INTERRUPTS` site; file follow-ups for missing sites | `docs/hardening.md` “Current interrupt inventory”; source contains additional IVF parallel-build `ProcessInterrupts` usage | Not complete. The current inventory names DiskANN, SPIRE, DistANN, and HNSW but omits at least the IVF parallel-build site and has not been demonstrated exhaustive. This is a source-audit task that can be completed on M5. |
| `make fault-full` is locally authoritative | Packet 001 `make-fault-full-dry-run.log`; canonical Make target | Not complete and Linux-blocked for behavioral authority. The current M5 result is planning/dry-run evidence only. It cannot become locally authoritative until the live provider and cgroup prerequisites are available on the designated Intel/Linux host. |
| `docs/hardening.md` contains a fault-injection model | `docs/hardening.md` “PG Fault Injection” section, including seven-fixture matrix, status vocabulary, provider/socket/cgroup operators, shared postconditions, and interrupt inventory | Complete as documentation, subject to the explicit interrupt-inventory completeness gap above. |

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
| Cancellation mutation control | Apple M5 PG18 is sufficient | Packet-local temporary-mutation provenance and a live cancellation result showing the lane catches the deliberately introduced invalid-buffer-style defect |
| Resource/palloc negative control | Apple M5 PG18 is sufficient | Packet-local controlled failure provenance and a five-AM/seven-fixture result proving the smoke rejects an unrecovered accumulator allocation failure |
| Exhaustive interrupt inventory | Apple M5 source audit is sufficient | Every long-running loop mapped to an interrupt poll or a filed follow-up; include the currently omitted IVF parallel-build site |

## Conclusion

The current Apple-M5 implementation/source-review slice is complete and
outside-reviewed. Task 38 remains open, without a closeout or promotion claim,
for three M5-verifiable canonical validation gaps and the designated Intel/Linux
execution evidence.
