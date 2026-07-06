# Task 145 Packet 011: Remote Bound-Prune Engagement Rerun

## Request

Please review the counter-instrumented remote bound-prune A/B rerun. This packet addresses the outstanding review feedback from packets 008 and 010: use the dedicated `pre_materialization_pruned_sum` counter to determine whether the bound-prune mechanism actually fires on the remote production-read path.

## Result

The rerun proves the bound-prune mechanism is inert in this configuration.

Across 10k/n128, 50k/n1024, and 100k/n1024:

- `bound-prune-off` and `bound-prune-on` have identical `leaf_candidate_sum` and `truncated_sum`.
- `pre_materialization_pruned_sum=0` in both arms.
- `sound_bound_available_sum=0` in both arms.

Because the on arm did not engage, this packet makes no latency/recall economy conclusion. Any latency/recall comparison from these arms is null evidence for bound-prune value.

## Evidence

Primary summary: `artifacts/engagement-summary.txt`

```text
10k-n128-bound-r2 bound-prune-off rows=15 leaf_candidate_sum=3410123 truncated_sum=3380433 pre_materialization_pruned_sum=0 pre_materialization_pruned_max=0 sound_bound_available_sum=0 sound_bound_missing_sum=43200
10k-n128-bound-r2 bound-prune-on rows=15 leaf_candidate_sum=3410123 truncated_sum=3380433 pre_materialization_pruned_sum=0 pre_materialization_pruned_max=0 sound_bound_available_sum=0 sound_bound_missing_sum=43200
50k-n1024-bound-r2 bound-prune-off rows=15 leaf_candidate_sum=2221377 truncated_sum=2191497 pre_materialization_pruned_sum=0 pre_materialization_pruned_max=0 sound_bound_available_sum=0 sound_bound_missing_sum=43200
50k-n1024-bound-r2 bound-prune-on rows=15 leaf_candidate_sum=2221377 truncated_sum=2191497 pre_materialization_pruned_sum=0 pre_materialization_pruned_max=0 sound_bound_available_sum=0 sound_bound_missing_sum=43200
100k-n1024-bound-r2 bound-prune-off rows=15 leaf_candidate_sum=4167023 truncated_sum=4137373 pre_materialization_pruned_sum=0 pre_materialization_pruned_max=0 sound_bound_available_sum=0 sound_bound_missing_sum=43200
100k-n1024-bound-r2 bound-prune-on rows=15 leaf_candidate_sum=4167023 truncated_sum=4137373 pre_materialization_pruned_sum=0 pre_materialization_pruned_max=0 sound_bound_available_sum=0 sound_bound_missing_sum=43200
```

The summary was extracted only from each `Production selected-leaf scan profile` table, not from later numeric tables in the logs.

Release proof: `artifacts/release-profile-summary.txt`

- All three cells record `install_profile=release`.
- Coord and all three remote nodes record `profile=release`.
- All three cells record `HARNESS PASSED`.

## Non-Claims

- Packet 008 remains rejected/null. Its previous latency/recall conclusion is not used.
- This packet is not an engaged negative. The mechanism never fired.
- This packet does not prove bound-prune cannot work after a runtime fix; it proves the current remote path does not produce sound bounds or pre-materialization prune events under the tested release suite.

## Follow-Up

Task 145 still needs a decision packet that treats bound-prune as inert unless a runtime fix is implemented and rebenchmarked with `bound-prune-on` showing `pre_materialization_pruned_sum>0`.
