# NFR-021 fixed-roster contradiction

## Normative conflict

`spec/non-functional/NFR-021-distann-distribution-invariant.md` states:

- lines 25-27: O(N) index state must be sharded across the serving roster;
- lines 55-60: the disjoint row tier is not a violation, and a genuinely
  sharded O(N) structure satisfies the requirement; and
- line 98: maximum single-node resident index bytes must grow by at most 2.0×
  from 10k to 100k.

For a fixed roster of `R` balanced owners, a genuine shard contains
approximately `N / R` records. With fixed graph degree and vector dimension,
its resident bytes are O(N/R). Holding `R=3` constant while increasing N from
10k to 100k therefore implies approximately 10× raw per-owner growth.

The raw `<=2.0` threshold can be met only by:

- increasing the roster by roughly 5× or more over the sweep;
- reducing bytes per stored record by roughly 5× or more; or
- ceasing to store the required owner partition.

None is part of Task 172's fixed-three-node topology.

## Task 205 measurements

Source:
`reviews/task-205/003-ab/artifacts/run-candidate-stage2/results.jsonl`.

The owner-control rows report:

| Metric | 10k | 100k |
| --- | ---: | ---: |
| max single-node graph-side bytes | 25,706,496 | 277,372,928 |
| cluster graph-side bytes | 75,907,072 | 830,144,512 |
| max owner published records | 3,391 | 33,432 |
| non-owner records | 0 | 0 |
| orphan records | 0 | 0 |

Derived:

```text
raw_growth =
  277,372,928 / 25,706,496
  = 10.7899936265

normalized_growth =
  (277,372,928 / 100,000) / (25,706,496 / 10,000)
  = 1.0789993627

max_cluster_share_100k =
  277,372,928 / 830,144,512
  = 0.3341260756
```

The maximum 100k owner holds 33,432 of 100,000 published graph records
(`0.33432`), matching the byte share and the expected one-third hash balance.
The coordinator's graph, directory, row-tier, derived-relation, and total
resident bytes are all zero for the owner-traversal control.

## Finding

The measured owner lane satisfies the structural distribution rule:

- authoritative graph state is partitioned, not replicated;
- the coordinator holds no O(N) traversal copy;
- owners hold approximately one-third each;
- no owner holds non-owned records; and
- per-record density is approximately stable across scale.

It fails only the unnormalized raw-growth threshold, which is mathematically
incompatible with a fixed roster and NFR-021's explicit permission for genuine
O(N) shards.

## Impact on Task 205

Task 205's measured Algorithm 1 result remains useful:

- recall is identical;
- request bytes are identical;
- response bytes are nearly identical; and
- transport wait is not improved.

However, NFR-022 lines 19-25 and 88-93 prohibit a decision derived from an
inadmissible control. Therefore the packet may report the observed negative
effect, but its formal NFR-based `do not advance` decision cannot become the
program disposition until the contradictory admissibility metric is resolved.

## Impact on Task 172

Task 172 cannot pre-register a conforming fixed-three-node control under the
literal raw threshold. Running the final matrix now would either:

- knowingly use a control labeled inadmissible;
- ignore NFR-022; or
- fabricate a flat-growth property a real shard cannot have.

The correct action is to keep the final run open and route this finding to Task
208's gate implementation/reconciliation work.
