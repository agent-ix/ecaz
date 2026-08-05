# Suite audit and dry-run

Head SHA: `74d752ddb`

The canonical runner reported `audit passed: 1 steps` for
`task206-10k-diagnostic.json`. Its dry-run emitted one physical multinode
command with `--build-shards 1`, `--beam-width 32`, `--hop-rounds 8`,
`--head-index-cap 4096`, `--top-k 200`, and the archived `ec_real_10k` corpus.

No benchmark result is claimed by this dry-run.

The 50k diagnostic also passed audit and expanded one physical multinode
command with BW32/H8, top-k 200, and the archived `ec_real_50k` corpus.
