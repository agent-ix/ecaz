# Review request: Task 179 aggregate closeout

## Status

**Closeout requested; not yet complete.** This packet is the sole aggregate
closeout request required by the task plan. Please do not accept Task 179 on
the basis of self-attestation: acceptance criteria 1 and 13 explicitly require
outside-reviewer decisions on Task 163 D8 and Task 172's physical matrix.

Review base: `b7924eee9a8408dbeac0a14f9b3da2b915ac017f`.

## Required outside decisions

Please explicitly decide all three:

1. **AC-1:** Does `reviews/task-163/005-d8-scale-memory/`, together with its
   already reviewed packets 003–004, close D8 / FR-077-CON-4 with bounded
   stitch residency, measured completion high-water, 10k/50k/100k RSS/HWM,
   and recall-neutrality evidence?
2. **AC-13:** Does `reviews/task-172/003-postfix-physical-matrix-acceptance/`
   close the latency axis left open by Task 172 packet 002 and provide accepted
   10k/50k/100k physical-versus-single recall, warmed latency, storage, and
   topology evidence for Task 179? This is deliberately narrower than closing
   Task 172's broader telemetry/capacity work.
3. **Aggregate:** Do packets 039–058 close the carried findings from the last
   outside-reviewed Task 179 packets without introducing a new P1/P2 blocker?

If any answer is no, please leave a packet-local finding here rather than a
chat-only response.

## Acceptance-criteria matrix

| AC | Evidence | Requested disposition |
| --- | --- | --- |
| 1 | Task 163 packets 003–005; packet 005 exact suite 5/5, 11/11, bounded stitch bytes 35,784→36,240 across 10k→100k and equal 10k recall A/B | Outside decision required |
| 2 | Task 179 packets 002 and 006; reviewed golden/independent decode/endian/version/layout and descriptor-v2 rebuild-only coverage | Previously reviewed; confirm no regression |
| 3 | Packets 002, 020, 031; distributed control remains metadata-only/pre-publish fail-closed, legacy lane explicit | Previously reviewed |
| 4 | Packets 003, 005, 016; transactional row/graph/directory/journal staging, rollback/replay/abort tests | Previously reviewed |
| 5 | Packets 002, 005, 016; coordinator-trained codec artifact restoration and scoring parity | Previously reviewed |
| 6 | Packets 006, 012–018, 026, 033–034, 042, 053; real three-owner partial-ack and post-ack/pre-pointer recovery converges to one active epoch | Review packet 053 follow-up |
| 7 | Packets 019, 021–024, 027–028, 033; scan-token RAII, retirement fencing, force-retire and audited abandonment | Previously reviewed/follow-up remediated |
| 8 | Packets 020, 025, 029, 031, 035, 046, 049–050; registered physical generations, frozen materialization/quals/system columns, direct graph reads | Review 046 and 049–050 follow-ups |
| 9 | Packets 008, 014, 025, 031–032, 053 and Task 172 current matrix; exact/disjoint ownership, zero residue/orphans, in/out-roster and remote materialization | Follow-up/current evidence |
| 10 | Packets 031–032, 053–054; real owner shells, streamed handoff, no full replica or delete/tombstone pruning | Follow-up/current evidence |
| 11 | Packets 031–032 and Task 172 packets 002–003; replicated-control rows remain explicitly labeled and are not promoted | Outside AC-13 decision required |
| 12 | Packets 005, 007, 009, 055, 058; physical-control DML/DDL gate fails closed while legacy behavior and unrelated-table DML remain positive controls | Review 055/058 follow-ups |
| 13 | Task 172 packet 002 accepted recall/storage/topology axes plus packet 003's current post-fix warmed-latency matrix sourced from Task 179 packet 052 | Outside decision required |

## Carried-finding reconciliation after packet 038

- Packet 033 endpoint/cancelled-recovery findings: 039, 041, and 042.
- Packet 036 cancellation P1, serial-fanout P2, and prompt-cancel P3: 040,
  043/045, and 051/052 respectively.
- Packet 035 direct-read P2-3b: 049/050.
- Packet 038's remaining removed O(N) seed-scan A/B: 047/048.
- Production system-column projections: 046.
- Real physical publish crash windows: 053.
- Deferred active+Ready `DROP EXTENSION` cleanup drill: 054.
- Packet 007 utility-lock/ATTACH/TRUNCATE/EXPLAIN findings and DML hot-path
  cost: 055, 057, and 058; packet 056 supplies the raw-suite result plumbing
  and is included in this outside-review request.

## Current benchmark decision

Task 172's current immutable matrix reports physical recall at or above the
same-run single control at every scale:

| Scale | Physical / single recall@10 | Physical warmed mean / p50 / p95 / p99 ms | Single mean / p50 / p95 / p99 ms | Physical generation bytes |
| --- | ---: | ---: | ---: | ---: |
| 10k | 1.0000 / 1.0000 | 43.50 / 44.00 / 55.70 / 56.10 | 2.83 / 2.77 / 3.43 / 3.57 | 242,761,728 |
| 50k | 0.9800 / 0.9750 | 54.50 / 54.20 / 67.90 / 72.30 | 3.38 / 3.43 / 3.98 / 4.15 | 1,242,734,592 |
| 100k | 0.9500 / 0.9450 | 49.50 / 46.90 / 67.40 / 75.90 | 3.56 / 3.39 / 4.55 / 4.88 | 2,496,634,880 |

The 50k/100k storage amplification remains a valid negative product finding
and is not promoted as NFR-018 success. Task 179 requires measured and accepted
evidence, not a fabricated win.

Packet 057 separately shows the installed/no-active-gate DML path below its
declared microbenchmark bars: median `0.988x` / `-0.085 us`, p95 `0.990x`,
with 9/9 thresholds passing.

## Reviewer focus

1. Audit the two cross-task decisions above against their immutable
   manifests/results rather than this summary table alone.
2. Check packet 058's transactional relcache invalidation argument and its
   warmed-negative cross-backend regression before accepting packet 057's fast
   steady-state measurement.
3. Confirm packet 053 is the real physical three-process fault proof, not the
   older replicated-control drill.
4. Confirm packet 054 proves hidden physical relations disappear after DROP
   EXTENSION while preloaded-hook ordinary DML still works.
5. Leave this request open until a feedback file explicitly says AC-1, AC-13,
   and aggregate Task 179 closeout are accepted.

## Validation policy

No new code or measurement was produced for this aggregation packet. It cites
the owning packet-local logs/manifests/results without copying them. No tests
were rerun merely to assemble the index.
