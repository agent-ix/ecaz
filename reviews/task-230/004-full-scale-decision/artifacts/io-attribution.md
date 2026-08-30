# Task 230 row-tier I/O attribution

This is a lossless consolidation of the 90
`physical_benchmark_row_tier_io` records and 20 summary records in the retained
per-arm `distann-multinode-summary.log` files. Counters are same-session
pre/post deltas after `pg_stat_clear_snapshot()`. `hot_or_row` means the hot
relation in the candidate and the single row relation in the control.

## Arm totals

| Arm | Elapsed ms | Iterations | Total accesses | Total hits | Shared-buffer hit ratio |
|---|---:|---:|---:|---:|---:|
| `10k/primary/pair-a/rowheap-first` | 391.367 | 50 | 300 | 300 | 1.000000 |
| `10k/primary/pair-a/hotcold-second` | 373.390 | 50 | 1653 | 1645 | 0.995160 |
| `10k/primary/pair-b/hotcold-first` | 368.482 | 50 | 1653 | 1645 | 0.995160 |
| `10k/primary/pair-b/rowheap-second` | 395.444 | 50 | 300 | 300 | 1.000000 |
| `50k/primary/pair-a/rowheap-first` | 465.570 | 50 | 300 | 300 | 1.000000 |
| `50k/primary/pair-a/hotcold-second` | 412.167 | 50 | 1744 | 1602 | 0.918578 |
| `50k/primary/pair-b/hotcold-first` | 436.422 | 50 | 1760 | 1595 | 0.906250 |
| `50k/primary/pair-b/rowheap-second` | 431.050 | 50 | 300 | 300 | 1.000000 |
| `100k/primary/pair-a/rowheap-first` | 498.498 | 50 | 300 | 300 | 1.000000 |
| `100k/primary/pair-a/hotcold-second` | 430.062 | 50 | 1673 | 1628 | 0.973102 |
| `100k/primary/pair-b/hotcold-first` | 436.030 | 50 | 1668 | 1633 | 0.979017 |
| `100k/primary/pair-b/rowheap-second` | 405.023 | 50 | 300 | 300 | 1.000000 |
| `100k/secondary/exact-vector/rowheap-first` | 559.127 | 50 | 6302 | 6274 | 0.995557 |
| `100k/secondary/exact-vector/hotcold-second` | 566.784 | 50 | 1668 | 1633 | 0.979017 |
| `100k/secondary/cold-only/hotcold-first` | 474.840 | 50 | 5055 | 5001 | 0.989318 |
| `100k/secondary/cold-only/rowheap-second` | 491.319 | 50 | 5052 | 5008 | 0.991291 |
| `100k/secondary/mixed/rowheap-first` | 421.456 | 50 | 5052 | 5008 | 0.991291 |
| `100k/secondary/mixed/hotcold-second` | 453.966 | 50 | 6723 | 6634 | 0.986762 |
| `100k/secondary/select-all/hotcold-first` | 867.906 | 50 | 13623 | 13517 | 0.992219 |
| `100k/secondary/select-all/rowheap-second` | 846.281 | 50 | 18852 | 18763 | 0.995279 |

## Per-node and per-relation deltas

Columns after tier are heap reads/hits, TOAST-heap reads/hits, TOAST-index
reads/hits, and the relation-level shared-buffer hit ratio.

| Arm | Node | Tier | Heap R | Heap H | TOAST R | TOAST H | Tidx R | Tidx H | Hit ratio |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| `10k/primary/pair-a/rowheap-first` | 1 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `10k/primary/pair-a/rowheap-first` | 2 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `10k/primary/pair-a/rowheap-first` | 3 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `10k/primary/pair-a/hotcold-second` | 1 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `10k/primary/pair-a/hotcold-second` | 1 | hot_or_row | 8 | 545 | 0 | 0 | 0 | 0 | 0.985533 |
| `10k/primary/pair-a/hotcold-second` | 2 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `10k/primary/pair-a/hotcold-second` | 2 | hot_or_row | 0 | 550 | 0 | 0 | 0 | 0 | 1.000000 |
| `10k/primary/pair-a/hotcold-second` | 3 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `10k/primary/pair-a/hotcold-second` | 3 | hot_or_row | 0 | 550 | 0 | 0 | 0 | 0 | 1.000000 |
| `10k/primary/pair-b/hotcold-first` | 1 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `10k/primary/pair-b/hotcold-first` | 1 | hot_or_row | 8 | 545 | 0 | 0 | 0 | 0 | 0.985533 |
| `10k/primary/pair-b/hotcold-first` | 2 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `10k/primary/pair-b/hotcold-first` | 2 | hot_or_row | 0 | 550 | 0 | 0 | 0 | 0 | 1.000000 |
| `10k/primary/pair-b/hotcold-first` | 3 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `10k/primary/pair-b/hotcold-first` | 3 | hot_or_row | 0 | 550 | 0 | 0 | 0 | 0 | 1.000000 |
| `10k/primary/pair-b/rowheap-second` | 1 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `10k/primary/pair-b/rowheap-second` | 2 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `10k/primary/pair-b/rowheap-second` | 3 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `50k/primary/pair-a/rowheap-first` | 1 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `50k/primary/pair-a/rowheap-first` | 2 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `50k/primary/pair-a/rowheap-first` | 3 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `50k/primary/pair-a/hotcold-second` | 1 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `50k/primary/pair-a/hotcold-second` | 1 | hot_or_row | 63 | 527 | 0 | 0 | 0 | 0 | 0.893220 |
| `50k/primary/pair-a/hotcold-second` | 2 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `50k/primary/pair-a/hotcold-second` | 2 | hot_or_row | 55 | 534 | 0 | 0 | 0 | 0 | 0.906621 |
| `50k/primary/pair-a/hotcold-second` | 3 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `50k/primary/pair-a/hotcold-second` | 3 | hot_or_row | 24 | 541 | 0 | 0 | 0 | 0 | 0.957522 |
| `50k/primary/pair-b/hotcold-first` | 1 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `50k/primary/pair-b/hotcold-first` | 1 | hot_or_row | 63 | 527 | 0 | 0 | 0 | 0 | 0.893220 |
| `50k/primary/pair-b/hotcold-first` | 2 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `50k/primary/pair-b/hotcold-first` | 2 | hot_or_row | 55 | 534 | 0 | 0 | 0 | 0 | 0.906621 |
| `50k/primary/pair-b/hotcold-first` | 3 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `50k/primary/pair-b/hotcold-first` | 3 | hot_or_row | 47 | 534 | 0 | 0 | 0 | 0 | 0.919105 |
| `50k/primary/pair-b/rowheap-second` | 1 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `50k/primary/pair-b/rowheap-second` | 2 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `50k/primary/pair-b/rowheap-second` | 3 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `100k/primary/pair-a/rowheap-first` | 1 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `100k/primary/pair-a/rowheap-first` | 2 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `100k/primary/pair-a/rowheap-first` | 3 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `100k/primary/pair-a/hotcold-second` | 1 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `100k/primary/pair-a/hotcold-second` | 1 | hot_or_row | 27 | 538 | 0 | 0 | 0 | 0 | 0.952212 |
| `100k/primary/pair-a/hotcold-second` | 2 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `100k/primary/pair-a/hotcold-second` | 2 | hot_or_row | 11 | 544 | 0 | 0 | 0 | 0 | 0.980180 |
| `100k/primary/pair-a/hotcold-second` | 3 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `100k/primary/pair-a/hotcold-second` | 3 | hot_or_row | 7 | 546 | 0 | 0 | 0 | 0 | 0.987342 |
| `100k/primary/pair-b/hotcold-first` | 1 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `100k/primary/pair-b/hotcold-first` | 1 | hot_or_row | 27 | 538 | 0 | 0 | 0 | 0 | 0.952212 |
| `100k/primary/pair-b/hotcold-first` | 2 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `100k/primary/pair-b/hotcold-first` | 2 | hot_or_row | 1 | 549 | 0 | 0 | 0 | 0 | 0.998182 |
| `100k/primary/pair-b/hotcold-first` | 3 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `100k/primary/pair-b/hotcold-first` | 3 | hot_or_row | 7 | 546 | 0 | 0 | 0 | 0 | 0.987342 |
| `100k/primary/pair-b/rowheap-second` | 1 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `100k/primary/pair-b/rowheap-second` | 2 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `100k/primary/pair-b/rowheap-second` | 3 | hot_or_row | 0 | 100 | 0 | 0 | 0 | 0 | 1.000000 |
| `100k/secondary/exact-vector/rowheap-first` | 1 | hot_or_row | 0 | 100 | 10 | 490 | 0 | 1500 | 0.995238 |
| `100k/secondary/exact-vector/rowheap-first` | 2 | hot_or_row | 0 | 100 | 10 | 490 | 0 | 1501 | 0.995240 |
| `100k/secondary/exact-vector/rowheap-first` | 3 | hot_or_row | 0 | 100 | 8 | 492 | 0 | 1501 | 0.996192 |
| `100k/secondary/exact-vector/hotcold-second` | 1 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `100k/secondary/exact-vector/hotcold-second` | 1 | hot_or_row | 27 | 538 | 0 | 0 | 0 | 0 | 0.952212 |
| `100k/secondary/exact-vector/hotcold-second` | 2 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `100k/secondary/exact-vector/hotcold-second` | 2 | hot_or_row | 1 | 549 | 0 | 0 | 0 | 0 | 0.998182 |
| `100k/secondary/exact-vector/hotcold-second` | 3 | cold | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `100k/secondary/exact-vector/hotcold-second` | 3 | hot_or_row | 7 | 546 | 0 | 0 | 0 | 0 | 0.987342 |
| `100k/secondary/cold-only/hotcold-first` | 1 | cold | 2 | 99 | 13 | 637 | 1 | 749 | 0.989340 |
| `100k/secondary/cold-only/hotcold-first` | 1 | hot_or_row | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `100k/secondary/cold-only/hotcold-first` | 2 | cold | 2 | 99 | 9 | 491 | 2 | 599 | 0.989185 |
| `100k/secondary/cold-only/hotcold-first` | 2 | hot_or_row | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `100k/secondary/cold-only/hotcold-first` | 3 | cold | 2 | 99 | 21 | 1029 | 2 | 1199 | 0.989371 |
| `100k/secondary/cold-only/hotcold-first` | 3 | hot_or_row | 0 | 0 | 0 | 0 | 0 | 0 | not_observed |
| `100k/secondary/cold-only/rowheap-second` | 1 | hot_or_row | 0 | 100 | 13 | 637 | 1 | 749 | 0.990667 |
| `100k/secondary/cold-only/rowheap-second` | 2 | hot_or_row | 0 | 100 | 9 | 491 | 0 | 601 | 0.992506 |
| `100k/secondary/cold-only/rowheap-second` | 3 | hot_or_row | 0 | 100 | 21 | 1029 | 0 | 1201 | 0.991068 |
| `100k/secondary/mixed/rowheap-first` | 1 | hot_or_row | 0 | 100 | 13 | 637 | 1 | 749 | 0.990667 |
| `100k/secondary/mixed/rowheap-first` | 2 | hot_or_row | 0 | 100 | 9 | 491 | 0 | 601 | 0.992506 |
| `100k/secondary/mixed/rowheap-first` | 3 | hot_or_row | 0 | 100 | 21 | 1029 | 0 | 1201 | 0.991068 |
| `100k/secondary/mixed/hotcold-second` | 1 | cold | 2 | 99 | 13 | 637 | 1 | 749 | 0.989340 |
| `100k/secondary/mixed/hotcold-second` | 1 | hot_or_row | 27 | 538 | 0 | 0 | 0 | 0 | 0.952212 |
| `100k/secondary/mixed/hotcold-second` | 2 | cold | 2 | 99 | 9 | 491 | 2 | 599 | 0.989185 |
| `100k/secondary/mixed/hotcold-second` | 2 | hot_or_row | 1 | 549 | 0 | 0 | 0 | 0 | 0.998182 |
| `100k/secondary/mixed/hotcold-second` | 3 | cold | 2 | 99 | 21 | 1029 | 2 | 1199 | 0.989371 |
| `100k/secondary/mixed/hotcold-second` | 3 | hot_or_row | 7 | 546 | 0 | 0 | 0 | 0 | 0.987342 |
| `100k/secondary/select-all/hotcold-first` | 1 | cold | 2 | 99 | 19 | 1381 | 1 | 2249 | 0.994135 |
| `100k/secondary/select-all/hotcold-first` | 1 | hot_or_row | 27 | 538 | 0 | 0 | 0 | 0 | 0.952212 |
| `100k/secondary/select-all/hotcold-first` | 2 | cold | 2 | 99 | 17 | 1333 | 2 | 2099 | 0.994088 |
| `100k/secondary/select-all/hotcold-first` | 2 | hot_or_row | 1 | 549 | 0 | 0 | 0 | 0 | 0.998182 |
| `100k/secondary/select-all/hotcold-first` | 3 | cold | 2 | 99 | 24 | 1826 | 2 | 2699 | 0.993981 |
| `100k/secondary/select-all/hotcold-first` | 3 | hot_or_row | 7 | 546 | 0 | 0 | 0 | 0 | 0.987342 |
| `100k/secondary/select-all/rowheap-second` | 1 | hot_or_row | 0 | 100 | 29 | 2121 | 1 | 3749 | 0.995000 |
| `100k/secondary/select-all/rowheap-second` | 2 | hot_or_row | 0 | 100 | 27 | 2173 | 0 | 3601 | 0.995425 |
| `100k/secondary/select-all/rowheap-second` | 3 | hot_or_row | 0 | 100 | 32 | 2618 | 0 | 4201 | 0.995396 |

## Mechanism disposition

- Primary id-only/hot-scalar candidate: every cold-tier counter is zero.
- Exact-vector candidate: every cold-tier counter is zero.
- Cold-only candidate: every hot-tier counter is zero.

The frozen tier-laziness hard gate therefore passes.
