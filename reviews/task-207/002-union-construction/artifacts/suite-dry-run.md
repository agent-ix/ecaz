# Suite audit and dry-run

Head SHA: `ea7f0af53d2ffb6c29fefde5fe9a3fc448237260`

`target/debug/ecaz bench suite audit --config reviews/task-207/002-union-construction/artifacts/task207-100k-union-ab.json`

Result: `audit passed: 2 steps`

The dry-run expanded two physical multinode commands, one with
`--build-shards 1` and one with `--build-shards 4`. Both hold
`--beam-width 128`, `--hop-rounds 5`, `--head-index-cap 4096`, `--top-k 200`,
and `head_seed_count=200` fixed, with persisted-head and owner-oracle variants.
No result rows were produced.
