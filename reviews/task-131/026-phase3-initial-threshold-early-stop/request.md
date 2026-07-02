# Review Request: Phase 3 Increment A Initial Threshold Early-Stop

- code commit: `dc5e54f7296adcd3977d370bf162951a568d13e9`
- packet: `reviews/task-131/026-phase3-initial-threshold-early-stop`
- status: implementation slice, not closeout

## What Changed

This implements the first real Phase 3 early-stop slice:

- adds default-off GUC `ec_spire.remote_search_initial_threshold_early_stop`;
- derives a conservative initial remote scan threshold from the coordinator-local merged kth heap candidate when local candidates fill `top_k`;
- sends that seed through production libpq candidate requests via new endpoint `ec_spire_remote_search_with_initial_threshold`;
- applies the threshold on the worker by reusing leaf block summaries and existing `selected_row_ranges`, skipping row segment reads for blocks whose sound upper bound cannot beat the threshold;
- preserves old behavior when the gate is off, no full local top-k seed exists, summaries are missing, or the payload format is not RaBitQ.

This is intentionally not a mid-scan threshold refresh channel. It is the reviewer-requested increment A: a single-variable, gated, recall-safe seed using already-known coordinator-local candidates.

## Validation

Packet-local logs:

- `artifacts/select-threshold-leaf-block-ranges.log`
- `artifacts/initial-threshold-seed.log`
- `artifacts/remote-search-initial-threshold-no-run.log`

Results:

- worker threshold selector test passed;
- coordinator-local kth seed test passed;
- focused library no-run compile passed.

## Still Open

This packet does not claim Phase 3 viability or task closeout. Required next evidence:

- result identity / matched recall with gate off vs on;
- rows scanned/scored avoided from production profile with the gate on;
- matched-recall latency A/B at 10k and 50k `n128/b4` first;
- later 100k and `n1024/b2` coverage if the 10k/50k result is promising.

