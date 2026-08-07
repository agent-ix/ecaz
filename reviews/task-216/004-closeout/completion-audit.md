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
| Closeout review acceptance | packet 004 review feedback / PR review | pending |

The final row is the only incomplete requirement. Task 216 must not be marked
review-closed or complete until an outside reviewer accepts the STOP and the
conditional full-scale decision. The corrected rationale and lane carry-ins
are in packets 002–004.
