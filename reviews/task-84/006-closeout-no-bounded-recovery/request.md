# Task 84 Review Request: Closeout With No Bounded Recovery Policy

## Summary

This packet closes Task 84 without landing a recovery policy.

Task 84 investigated the credible AWS 1M/q500 selected-block recovery paths
after Tasks 79-83 established the retained candidate surface:

- retained baseline: `recall@10=0.9832`, `candidate_sum=9,213,846`;
- miss split: `3` routing misses, `81` selected-leaf block-pruning or
  candidate-cap misses;
- Task 83 blanket-cap controls recovered recall only by growing q500
  candidates to `10.24M-13.31M`.

## Accepted Code

The only code change retained from Task 84 is prerequisite tooling:

- `07974586f` allows non-PK SPIRE KNN reads to pass through the ADR-069 DML
  front-door when a table has multiple SPIRE indexes.
- PG18 validation passed for both the multi-index KNN pass-through and the
  existing fail-closed context-error behavior.

This does not change production SPIRE candidate selection.

## Rejected Recovery Paths

### Route Prior

Packet `002` tested route-prior weights `0.02`, `0.05`, `0.10`, and `0.20` at
the retained AWS 1M/q500 surface.

All four rows remained at:

- `recall@10=0.9832`
- miss split `4916/3/81`

Route prior preserved or slightly reduced candidate counts, but it recovered
zero selected-leaf misses.

### k > 2 Summary Representatives

Packet `005` measured the retained k=3 SPIRE index after the multi-index KNN
front-door fix.

At `global1152`:

- `recall@10=0.9832`
- `candidate_sum=9,213,742`
- miss split unchanged: `3` routing misses, `81` selected-leaf misses

At `global1280`, k=3 matched the Task 83 blanket-cap neighborhood rather than
beating it:

- `recall@10=0.9846`
- `candidate_sum=10,237,430`

### Selective Near-Cap Rescue

Packet `001` enriched the selected-leaf miss context with target block rank and
score margin to the retained cap. Packet `006` adds an idealized rescue upper
bound from that packet-local AWS evidence.

Score-margin rescue is too small:

| rescue predicate | max selected-leaf truth rows recovered | ideal recall |
| --- | ---: | ---: |
| margin `<=0.001` | 3 | 0.9838 |
| margin `<=0.0025` | 6 | 0.9844 |
| margin `<=0.005` | 11 | 0.9854 |
| margin `<=0.01` | 26 | 0.9884 |

Rank-window rescue is not a bounded product policy:

| extra blocks | oracle queries covered | truth rows covered | minimum added candidates |
| ---: | ---: | ---: | ---: |
| 128 | 6 | 6 | 12,288 |
| 256 | 11 | 12 | 45,056 |
| 512 | 20 | 24 | 163,840 |
| 1024 | 31 | 35 | 507,904 |
| 2048 | 40 | 47 | 1,310,720 |

Those are oracle lower bounds that trigger only on queries known to miss. A
real ambiguity/score-margin trigger would also add blocks for non-missing
queries, so the actual candidate cost would be higher. The narrow predicates do
not recover enough recall; the wide predicates become another form of the
rejected blanket-cap sweep.

## Decision

No bounded Task 84 recovery policy is justified.

Task 84 should close with the negative evidence above and hand off to Task 85,
which treats SPIRE as a broader product-scale Pareto program rather than more
single-knob recall rescue work.

## Artifacts

- `artifacts/manifest.md`
- `artifacts/idealized-rescue-coverage.tsv`
- `artifacts/per-query-rank-rescue-upper-bound.tsv`
- `artifacts/analysis-commands.md`
- `artifacts/cloud-status-final-paused-closeout.log`

AWS `1m` final status: `paused`, running cost `~$0.00/hr`, retained storage
`~$8.00/mo`.
