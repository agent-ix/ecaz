# Task 84 Closeout Analysis Commands

Source data:

`reviews/task-84/001-enriched-block-context-diagnostic/artifacts/selected-leaf-miss-enriched-context.tsv`

The TSV rows are the `81` selected-leaf truth misses from the AWS 1M/q500
retained `global1152` baseline. The closeout uses it only as an idealized upper
bound: a real rescue trigger would add blocks for non-missing queries too.

## Idealized Rescue Coverage

```sh
awk -F '\t' '{d=$7-$8; q=$1; if (!(q in seen)) qcount++; seen[q]=1; if (d<=64){r64++; q64[q]=1}; if (d<=128){r128++; q128[q]=1}; if (d<=256){r256++; q256[q]=1}; if (d<=512){r512++; q512[q]=1}; if (d<=1024){r1024++; q1024[q]=1}; if (d<=2048){r2048++; q2048[q]=1}; m=-$11; if (m<=0.001){m001++; qm001[q]=1}; if (m<=0.0025){m0025++; qm0025[q]=1}; if (m<=0.005){m005++; qm005[q]=1}; if (m<=0.01){m01++; qm01[q]=1}; total++} END {for (q in q64) cq64++; for (q in q128) cq128++; for(q in q256)cq256++; for(q in q512)cq512++; for(q in q1024)cq1024++; for(q in q2048)cq2048++; for(q in qm001)cqm001++; for(q in qm0025)cqm0025++; for(q in qm005)cqm005++; for(q in qm01)cqm01++; print "axis\ttruth_rows\tdistinct_queries\trecall_if_all_recovered"; printf "rank<=64\t%d\t%d\t%.4f\n", r64,cq64,(4916+r64)/5000; printf "rank<=128\t%d\t%d\t%.4f\n", r128,cq128,(4916+r128)/5000; printf "rank<=256\t%d\t%d\t%.4f\n", r256,cq256,(4916+r256)/5000; printf "rank<=512\t%d\t%d\t%.4f\n", r512,cq512,(4916+r512)/5000; printf "rank<=1024\t%d\t%d\t%.4f\n", r1024,cq1024,(4916+r1024)/5000; printf "rank<=2048\t%d\t%d\t%.4f\n", r2048,cq2048,(4916+r2048)/5000; printf "margin<=0.001\t%d\t%d\t%.4f\n", m001,cqm001,(4916+m001)/5000; printf "margin<=0.0025\t%d\t%d\t%.4f\n", m0025,cqm0025,(4916+m0025)/5000; printf "margin<=0.005\t%d\t%d\t%.4f\n", m005,cqm005,(4916+m005)/5000; printf "margin<=0.01\t%d\t%d\t%.4f\n", m01,cqm01,(4916+m01)/5000; printf "all_selected_leaf\t%d\t%d\t%.4f\n", total,qcount,(4916+total)/5000 }' reviews/task-84/001-enriched-block-context-diagnostic/artifacts/selected-leaf-miss-enriched-context.tsv
```

Output captured in `idealized-rescue-coverage.tsv`.

## Per-Query Rank Upper Bound

```sh
awk -F '\t' '{q=$1; d=$7-$8; if (!(q in max) || d>max[q]) max[q]=d; count[q]++} END {print "extra_blocks\tqueries_covered\ttruth_rows_covered\test_candidate_add"; for (b=64; b<=2048; b*=2){qcov=0; rcov=0; for(q in max){if(max[q]<=b){qcov++; rcov+=count[q]}}; printf "%d\t%d\t%d\t%d\n", b,qcov,rcov,qcov*b*16} }' reviews/task-84/001-enriched-block-context-diagnostic/artifacts/selected-leaf-miss-enriched-context.tsv
```

Output captured in `per-query-rank-rescue-upper-bound.tsv`.
