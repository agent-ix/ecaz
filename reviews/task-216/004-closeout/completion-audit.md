# Task 216 completion audit

| Requirement | Evidence | Current state |
|---|---|---|
| Accepted attribution before candidate work | `001-attribution/feedback/2026-08-06-02-reviewer.md` | proved |
| One candidate isolated from traversal/default changes | `002-isolated-candidate/request.md`, source commit `6662b302f` | proved |
| Fresh release control and candidate A/B | `002-isolated-candidate/artifacts/{control,candidate}/suite-manifest.json` | proved |
| Recall and confidence intervals | matching `results.jsonl` and summary logs | proved |
| Latency mean/p50/p95/p99/max | matching `results.jsonl` and summary logs | proved |
| Storage and topology gates | summary logs; zero orphans and two remote probes | proved |
| Release/corpus/query provenance | packet 002 manifest and suite manifests | proved |
| Candidate usefulness gate | MAT-15 ceiling is 0.076/40.60 ms = 0.19%; bytes unchanged | failed, correctly STOPped |
| Ordered identity/reproducibility gate | physical arrays differ in 2/200 rows because arms rebuilt generations | failed; lane carry-in recorded |
| Full-scale 10k/50k/100k matrix | packet 002 says conditional on useful 100k result | not required after STOP |
| Productionization/default change | no production commit or default change | not applicable |
| Closeout review acceptance | `004-closeout/feedback/2026-08-07-02-reviewer.md` | proved |

The reviewer accepted the negative STOP after the rationale and lane carry-ins
were corrected. Task 216 is review-closed as a negative STOP; MAT-21 remains a
separate future candidate rather than unfinished scope in this task.
