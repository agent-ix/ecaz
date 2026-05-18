# Task 47 Cost Gate Fixtures

`baseline.json` is the authoritative small-fixture cost baseline for
`make cost-gate`. It is generated from `target/gates/cost-small/results.jsonl`
with:

```sh
python3 scripts/check_cost_baseline.py \
  target/gates/cost-small/results.jsonl \
  fixtures/cost-queries/baseline.json \
  --accept-drift
```

Only update it in an explicit Task 47 review packet that includes the raw
planner-cost logs and explains the drift.
