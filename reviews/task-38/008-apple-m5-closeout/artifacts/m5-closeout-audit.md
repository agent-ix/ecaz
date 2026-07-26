# Task 38 Completion Audit — Final Apple-M5 Boundary

This audit supersedes packet 005's point-in-time gap accounting. It does not
rewrite packet 005 or claim Task 38 completion. It distinguishes:

1. implementation/source completion;
2. behavior verified locally on Apple M5 with PG18; and
3. behavior still requiring the designated Intel/Linux host.

## Original Objective

| Requirement | Evidence | Assessment |
| --- | --- | --- |
| Add DistANN as the fifth AM | `ecaz-fault-injection`, `ecaz-cli`, packet 001, final outside approval | Complete. DistANN is a first-class AM and not a DiskANN alias. |
| Cover RaBitQ, TurboQuant, and grouped-PQ DistANN workloads | Packet 001 live M5 PG18 artifacts and final approval | Complete on M5. Each codec has a distinct fixture, supported dimension, table, and physical index. |
| Add real DistANN remote socket faulting | DistANN TCP provider/operator implementation; packets 001 and 002 approvals | Implementation/source complete. Live reset/slow syscall and marker evidence remains Intel/Linux-pending. |
| Add real SPIRE remote socket faulting | SPIRE named-Unix provider/operator implementation; packet 003 final approval | Implementation/source complete. Live reset/slow syscall and marker evidence remains Intel/Linux-pending. |
| Close historical cleanup nits | Packet 001 final approval | Complete. Provider coverage, errno preservation, typed fixtures, and cleanup-oracle findings are closed. |
| Add cgroup OOM locally where supported | Executable seven-fixture operator; packet 004 final approval | Implementation/source complete. Apple M5 lacks Linux cgroup v2 and user systemd scopes; live execution remains Intel/Linux-pending. |
| Produce `reviews/task-38/001-...` | Packets 001–008 | Complete. Evidence remains task-scoped and packet-local. |
| Obtain outside review | Final packet-local approvals for packets 001–007 | Complete for each stated implementation/source/M5 evidence scope. No approval substitutes for missing Intel/Linux behavior. |

## Canonical Validation And Exit Criteria

| Criterion | Evidence | Assessment |
| --- | --- | --- |
| Every applicable lane completes without postmaster `PANIC` or leaked buffers/locks | Historical four-AM Linux evidence; packet 001 DistANN M5 logs; packet 007 zero-pin and zero session/lock/prepared-xact evidence | Partially complete overall. Proven for executed historical/local lanes; unexecuted Intel/Linux provider/socket/cgroup lanes remain open. |
| A deliberately introduced invalid-buffer-style bug is caught by cancellation | Packet 007 live seven-fixture cancellation mutation markers and final approval | Complete on M5. The canonical “invalid buffer” text is an example; the approved control injects an exact wrong AM palloc failure and proves the production `57014` oracle rejects it. |
| Resource exhaustion catches an accumulator/palloc failure without recovery | Packet 007 live seven-fixture armed/disarmed real-AM recovery markers and final approval | Complete on M5. The same KNN scan fails while palloc remains armed and succeeds only after disarm/reset. |
| All five AMs and all three DistANN codecs survive applicable smoke lanes | Historical four-AM evidence and packet 001 M5 DistANN evidence | Partially complete overall. The Intel/Linux provider/socket/cgroup cases remain unexecuted. |
| Document every long-running-loop interrupt site and file follow-ups | Packet 006 exact explicit-boundary artifact, final approval, and Task 200 | Complete for Task 38's inventory/follow-up criterion. Task 200 owns classification/remediation of unpolled loops and remains open independently. |
| `make fault-full` is locally authoritative | Dry-run planning plus implemented operators | Not complete. Behavioral authority requires the designated Intel/Linux provider and cgroup prerequisites. |
| `docs/hardening.md` contains the fault-injection model | Seven-fixture matrix, operator model, provider/socket/cgroup procedures, recovery oracles, mutation controls, and interrupt inventory | Complete. |

## Resolution Of Packet 005 M5 Gaps

| Packet 005 gap | Closing evidence | Status |
| --- | --- | --- |
| Cancellation mutation control | Packet 007 code checkpoint `374166bd3`, live seven-fixture log, reviewer approval `67177c713` | Closed on M5. |
| Resource/palloc negative control | Same packet 007 checkpoint, armed/disarmed real-AM recovery evidence, and approval | Closed on M5. |
| Exhaustive interrupt inventory | Packet 006 inventory checkpoint and reviewer approval `6cc24bf3e` | Closed for Task 38; Task 200 owns follow-up work. |

## Remaining Designated Intel/Linux Matrix

Task 38 remains open until all of the following behavior is executed and
stored as packet-local evidence:

1. DistANN provider-backed EIO, ENOSPC, and measured slow-disk cases, including
   exact fault markers, clean workload result/recovery, and shared
   postconditions.
2. DistANN real owner/payload TCP reset and slow cases, including exact-peer
   marker/syscall evidence and recovery.
3. SPIRE real participant named-Unix reset and slow cases, including exact-peer
   marker/syscall evidence, healthy baseline/stable slow result, and recovery.
4. Seven systemd/cgroup-v2 OOM cases: HNSW, IVF, DiskANN, SPIRE, DistANN
   RaBitQ, DistANN TurboQuant, and DistANN grouped-PQ, each with
   `Result=oom-kill`, scoped-postmaster death, restarted valid/ready index,
   forced AM scan, and clean shared postconditions.

No AWS, remote-host, CI, nightly, or Intel command was executed for this M5
closeout.

## Conclusion

The Apple-M5 work boundary is complete and supported by outside-reviewed source
and live local PG18 evidence. Task 38 is intentionally still open. Its only
remaining execution boundary is the designated Intel/Linux matrix above.
