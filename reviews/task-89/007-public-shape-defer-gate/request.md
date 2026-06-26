# Task 89 Review Request: Public Shape Defer Gate

## Summary

This packet asks the reviewer to accept the Task 89 Phase 6 public-shape gate
decision:

**Defer TQ+ in its current IVF experimental shape. Do not introduce a public
`turboquant_tqplus` storage format, do not promote a public TurboQuant
calibration option, and do not start SPIRE/HNSW/DiskANN ports from this
evidence.**

This is a gate recommendation, not a task closeout. If accepted, the next
packet can be a closeout packet that records Task 89 as complete-deferred.

## Evidence Considered

### Phase 1 / ADR

- ADR-081 is accepted as the IVF-only experimental profile direction.
- Public DDL shape remains deferred until evidence justifies it.
- Packet 001 reviewer feedback approved the architecture direction and blocked
  closeout only on latency comparability plus cross-corpus evidence.

### IVF DBPedia no-QJL

Packet: `reviews/task-89/003-ivf-tqplus-dbpedia-suite/`

Representative cells:

| scale | nprobe | TQ recall@10 | TQ+ recall@10 | Recall delta | Storage delta |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 48 | 0.9770 | 0.9720 | -0.50 pp | +1.6 B/index row |
| 50k | 64 | 0.9430 | 0.9460 | +0.30 pp | +0.3 B/index row |
| 100k | 96 | 0.9490 | 0.9430 | -0.60 pp | +0.1 B/index row |

This is not a reliable quality win. It is mixed at best, with recall loss at
10k and 100k on the representative no-QJL cells.

### IVF projected QJL/gamma

Packet: `reviews/task-89/004-ivf-tqplus-qjl-projected-suite/`

At projected DBPedia 10k, `nprobe=48`:

| variant | recall@10 | index bytes/row |
| --- | ---: | ---: |
| TQ baseline | 0.9070 | 535.8 B |
| TQ+ | 0.9100 | 536.6 B |

The QJL/gamma lane shows only a small recall gain (+0.30 pp) and near-neutral
storage. That is not enough to justify a public format or cross-AM port after
the no-QJL and cross-corpus results.

### Insert Drift

Packet: `reviews/task-89/005-ivf-tqplus-insert-drift/`

| insert ratio | live recall@10 | rebuild recall@10 | live-minus-rebuild | threshold |
| --- | ---: | ---: | ---: | ---: |
| 10% | 0.9265 | 0.9310 | -0.45 pp | informational |
| 25% | 0.9230 | 0.9235 | -0.05 pp | <= 0.5 pp |
| 50% | 0.9245 | 0.9220 | +0.25 pp | <= 1.0 pp |

Drift passes the Task 89 threshold for the measured DBPedia live-insert surface.
This removes one risk, but it does not create a promotion case.

### Cross-Corpus Synthetic

Packet: `reviews/task-89/006-ivf-tqplus-cross-corpus/`

On the deterministic synthetic non-DBPedia unit-sphere corpus:

| nprobe | TQ recall@10 | TQ+ recall@10 | Delta |
| ---: | ---: | ---: | ---: |
| 16 | 0.3800 | 0.3755 | -0.45 pp |
| 32 | 0.6075 | 0.5780 | -2.95 pp |
| 48 | 0.7675 | 0.7175 | -5.00 pp |
| 64 | 0.8610 | 0.7880 | -7.30 pp |

This satisfies the Task 89 stop condition: cross-corpus measurement reveals a
systematic regression on a non-DBPedia distribution.

## Latency Treatment

Per reviewer feedback in packet 001, the existing latency numbers are **not**
used as the basis for this gate decision. Current TQ+ scoring is scalar-only,
while baseline TurboQuant can use tiled/SIMD scoring, so the latency rows are
diagnostic rather than comparable decision evidence.

This gate decision uses recall quality, storage, and drift:

- Recall: mixed on DBPedia no-QJL, small gain on projected QJL, systematic
  regression on synthetic non-DBPedia.
- Storage: near-neutral, but not a positive differentiator.
- Drift: acceptable on the measured DBPedia live-insert surface.

## Decision Requested

Approve this Phase 6 decision:

- **Outcome:** Defer TQ+ in its current form.
- **Public shape:** no public `turboquant_tqplus` storage format and no public
  TurboQuant calibration option.
- **AM scope:** no SPIRE/HNSW/DiskANN ports from this evidence.
- **Future work:** only revisit after either a scoring-quality redesign or a
  new calibration approach that clears cross-corpus recall before latency is
  reconsidered with a comparable scorer.

## Not Claimed

This packet does not itself flip Task 89 to complete. It asks for reviewer
approval of the public-shape gate. A separate closeout packet should cite this
gate if accepted.
