# Task 105: Production-Target Optimization + Full-Scale Benchmark Sweep

Status: COMPLETE — merged to `main` 2026-06-13 (PR #32, merge commit
`afa7fb28e`). Phase 1 merged earlier (`main=1345ca603`, PR #31); Phase 2
ran green on both lanes at all four scales (packets 004/005); Phase 3
published (packet 006: matrix, baseline comparison, handoff). The
2026-06-13 reviewer blockers on packets 002/003/006 were resolved
doc-only (packet 007). All five acceptance criteria verified, including
a live AWS AC5 teardown check: both bench stacks destroyed, no
non-terminated instances in us-west-2, G4 `snap-0f546929f70d60fb5` +
Intel `snap-0338adc6455257604` completed and retained, corpus base
`snap-0e9c7743263e61d70` retained. Honest scope: fresh Task 105 evidence
is 10k/50k/1M both lanes + the G4 100k NEON confirm column; the full
100k on/off A/B is delegated to Task 99 (008/009) by Phase 2 design.
Original scope note: operator-confirmed — "optimize before the full
sweep".
Owner: coder (the Task 99/102/103 author lane). One coder.
Priority: 1 (gates the safety/cleanup/release-readiness shift)

## Why

Task 99 closed the kernel-completeness initiative with a one-scale
(real 100k) profile and produced three measured-but-unimplemented
production optimizations (ADR-077 §4/§6). The operator's intent for
release readiness is **exhaustive quality evidence**: the full
10k / 50k / 100k / 1M sweep against all indexes × quants × options on
the production targets (Graviton 4 + AWS Intel), comparable against
the prior baselines (May 2026 snapshot-era IVF numbers; pinned
comparator baseline `94c02c682` — comparators are NOT re-run), so the
project can shift to safety/cleanup/release work on a measured
foundation.

Sequencing rule (operator-confirmed): **optimize first** — the
dispatch flip changes what production dispatch measures, so it lands
on main before the sweep collects scale evidence (single-trip
economics; the Task 99 G4 NEON-capped pass already measured post-flip
behavior at 100k, so no collected evidence is invalidated).

## Scope

### Phase 1 — Production optimization slices (land + validate before the trip)

1. **aarch64 dispatch flip** (ADR-077 §6): `select_highest_isa`
   prefers Neon over Sve/Sve2 (measured: SVE2 loses 2.0–3.3× on
   lut32, 1.1–1.35× grouped-pq, ~6× qjl32 block path; e2e −27/−45%
   recoverable on every TQ cell). SVE2 kernels stay in-tree; re-entry
   per family only by future measurement (which will need a
   preference override — out of scope here).
2. **IVF batch default flip** (ADR-077 §4):
   `ec_ivf.scratch_soa_batch_decode` default off → on (measured:
   −66/−69% local and −44% G4 on IVF TQ; wins pq_fastscan despite the
   suffix-max trade; off switch retained for diagnostics).
3. **rabitq32 strict-test contract fix** (packet 009 finding): the two
   strict bit-equality-vs-production assertions become the family
   envelope (1-ULP divergence on Sapphire Rapids under
   `target-cpu=native`; binding tolerance gates unaffected).
4. Local validation: focused tests + clippy; local Intel behavior is
   dispatch-unchanged (avx2 first), IVF default flip smoke-checked.

### Phase 2 — Full-scale sweep (both production lanes)

- Restore the post-Task-99 snapshots (`snap-097eb8a8e881384dd` G4,
  `snap-0dc395f4f6458c37b` Intel — corpus base at all four scales +
  built 100k/QJL fixtures), `cloud install` at the post-Phase-1 main.
- Extend the Task 99 profile to scales **10k / 50k / 1M** (the 100k
  and QJL fixtures already exist): per-scale per-variant fixture
  replication from the snapshot sources, same 45-cell matrix per
  scale, scale-appropriate iteration counts at 1M.
- Day-one smoke per lane (with `--skip pg_test_`), catalog refresh if
  needed, suite runs, snapshot-then-destroy.
- **Re-run the 100k cells on G4** under the flipped dispatch as the
  post-optimization confirmation column (cheap — fixtures exist;
  expected ≈ the Task 99 NEON-capped numbers).

### Phase 3 — Aggregation + baseline comparison

- Full-scale matrix document: scale × AM × quant × option × lane.
- **Prior-baseline comparison**: new IVF rabitq1 numbers at
  10k/50k/100k/1M vs the May snapshot-era baseline cells; map to the
  pinned comparator baseline (`94c02c682`) at matching scales/configs —
  no comparator re-runs.
- Recall parity (byte-equal per family contract) at every scale;
  scoring-share + e2e + storage per cell; honest markers throughout.
- Handoff note for the safety/cleanup/release-readiness track.

### Out of scope

- Comparator re-runs (pinned baseline stands).
- New kernels / quants / AMs; SVE2 re-entry measurements.
- The safety/cleanup/release work itself (next track).

## Acceptance criteria

1. The three optimization slices merged to main with tests/clippy
   green and review packets.
2. Both lanes run the full matrix at 10k/50k/100k/1M: every cell
   green or honestly marked; recall parity per family contract at
   every scale; counters attribute the flipped dispatch
   (`isa=neon` on G4 kernel cells, `isa=avx2` on Intel).
3. G4 100k confirmation column matches the Task 99 NEON-capped
   numbers within noise (the optimization shipped what was measured).
4. Full-scale matrix + prior-baseline comparison tables published in
   the task bucket; release-readiness handoff note written.
5. Snapshot-then-destroy on both lanes; snapshot retention per the
   corpus-base policy.

## References

- ADR-077 (ACCEPTED; §4/§6 are this task's Phase 1 mandate)
- `reviews/task-99/` packets 002–009 (profile machinery + trip
  precedent + findings: pg_test skip, catalog refresh, 1-ULP)
- May 2026 baseline: snapshot description of `snap-0e9c7743263e61d70`
  and its source packets; comparator baseline `94c02c682`

## Estimated size

Phase 1: ~half a day incl. review. Phase 2: ~1–2 instance-days per
lane (1M index builds dominate). Phase 3: ~half a day.
