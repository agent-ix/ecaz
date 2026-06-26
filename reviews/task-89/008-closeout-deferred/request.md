# Task 89 Review Request: Deferred Closeout

## Summary

This packet requests reviewer approval to close Task 89 as:

**complete (deferred)** — TQ+ should remain unpromoted and shelved in its
current IVF experimental form because the measured cross-corpus recall
regression blocks public exposure and cross-AM ports.

This closeout is contingent on accepting the public-shape gate in
`reviews/task-89/007-public-shape-defer-gate/`.

## Requested Closeout Decision

- **Outcome:** Defer TQ+.
- **Public format:** do not introduce `storage_format = 'turboquant_tqplus'`.
- **Public option:** do not promote
  `turboquant_calibration = 'tqplus_experimental'` as an operator-facing
  production option.
- **Cross-AM scope:** do not port this TQ+ shape to SPIRE, HNSW, or DiskANN.
- **Reason:** recall evidence does not show a durable quality win, and the
  non-DBPedia synthetic corpus shows systematic recall regression.

Latency is intentionally not used as the closeout reason. Packet 001 feedback
identified current TQ+ scoring as scalar-only while baseline TurboQuant can use
tiled/SIMD scoring, so the latency rows remain diagnostic until scorer parity
exists.

## Acceptance-Criteria Map

| Task 89 criterion | Evidence | Closeout state |
| --- | --- | --- |
| ADR-081 recorded and reviewed | `spec/adr/ADR-081-tqplus-experimental-calibration-profile.md`; packet 001 feedback approves the direction | satisfied |
| IVF experimental TQ+ behind non-public option | Packets 001/003; option is `turboquant_calibration=tqplus_experimental` under `storage_format=turboquant` | satisfied |
| IVF mode matrix: no-QJL and reachable QJL/gamma | Packet 003 DBPedia 10k/50k/100k no-QJL; packet 004 projected QJL/gamma | satisfied |
| Cross-corpus evidence | Packet 006 deterministic synthetic non-DBPedia corpus | explicit stop condition |
| Streaming-insert drift evidence | Packet 005 10%, 25%, 50% live insert vs rebuild | satisfied |
| Public-shape gate | Packet 007 recommends defer/no public format/no cross-AM ports | pending reviewer approval |
| Closeout packet naming a Goal outcome | This packet names **Defer TQ+** | pending reviewer approval |
| Closeout only after pass/scoped-defer/explicit-stop | Cross-corpus regression in packet 006 is the explicit stop condition | satisfied if reviewer accepts defer gate |

## Evidence Summary

### No-QJL DBPedia

Packet 003 measured IVF DBPedia 10k/50k/100k. Representative cells were mixed:

- 10k `nprobe=48`: TQ+ recall -0.50 pp, storage +1.6 B/index row.
- 50k `nprobe=64`: TQ+ recall +0.30 pp, storage +0.3 B/index row.
- 100k `nprobe=96`: TQ+ recall -0.60 pp, storage +0.1 B/index row.

This does not justify promotion.

### QJL/Gamma

Packet 004 measured projected DBPedia 10k through the reachable QJL/gamma path.
At `nprobe=48`, TQ+ recall was +0.30 pp with near-neutral storage. This is a
small quality gain, but not enough to overcome no-QJL and cross-corpus results.

### Insert Drift

Packet 005 passed the measured drift thresholds:

- 25% insert: -0.05 pp live-minus-rebuild.
- 50% insert: +0.25 pp live-minus-rebuild.

Drift does not block TQ+, but it does not create a promotion case.

### Cross-Corpus

Packet 006 measured a deterministic synthetic non-DBPedia unit-sphere corpus.
TQ+ regressed recall at every measured probe:

- `nprobe=16`: -0.45 pp.
- `nprobe=32`: -2.95 pp.
- `nprobe=48`: -5.00 pp.
- `nprobe=64`: -7.30 pp.

This satisfies Task 89's cross-corpus stop condition and is the closeout reason.

## Follow-Up Guidance

Future TQ+ work should start as a redesign, not as a porting effort:

- investigate why the calibration hurts the synthetic distribution;
- add score-error diagnostics before more AM code;
- only revisit latency after a tiled/SIMD TQ+ scorer exists;
- require a new cross-corpus recall pass before any public format or cross-AM
  validation claim.

## Not Claimed

This request does not itself update `plan/tasks/89...` to complete. If the
reviewer accepts packet 007 and this closeout, the next mechanical step is to
mark Task 89 `complete (deferred)` and update the task index/status references.
