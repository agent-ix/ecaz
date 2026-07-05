# Task 148 Packet 001: TurboQuant 4-bit No-QJL Bias Audit

## Scope

Slice 1 only. This packet adds an offline operator audit command and measures TurboQuant 4-bit no-QJL estimator bias over the staged corpora in `data/staged-current/`. No product scoring path or on-disk format changed.

Code checkpoint:

- `edafc52d1e6726fc032a3c6ef32b53d83be173d9` adds `ecaz bench tq-bias-audit`.

Artifacts:

- `artifacts/manifest.md`
- `artifacts/tq-bias-audit-10k.{log,json}`
- `artifacts/tq-bias-audit-50k.{log,json}`
- `artifacts/tq-bias-audit-100k.{log,json}`
- `artifacts/tq-bias-audit-anchor-990k.{log,json}`

## Results

| scale | rows | norm mean | norm p01 | norm p99 | self-dot mean | self-dot p01 | self-dot p99 | filtered query-dot mean | filtered query-dot p01 | filtered query-dot p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 10,000 | 0.99264737 | 0.98168747 | 1.00272913 | 0.98741150 | 0.97532244 | 0.99807919 | 0.98637467 | 0.89654055 | 1.06605646 |
| 50k | 50,000 | 0.99273127 | 0.98158664 | 1.00266888 | 0.98751405 | 0.97522872 | 0.99803592 | 0.98672958 | 0.85926145 | 1.11615593 |
| 100k | 100,000 | 0.99278999 | 0.98167054 | 1.00271736 | 0.98758200 | 0.97532644 | 0.99806855 | 0.98681072 | 0.85721613 | 1.11723377 |
| anchor 990k | 990,000 | 0.99281619 | 0.98174221 | 1.00268438 | 0.98761515 | 0.97540358 | 0.99802266 | 0.98661024 | 0.84083837 | 1.13011197 |

The paired-query ratio is reported both unfiltered and filtered in the JSON/logs. The table uses the filtered distribution with `abs(<q,x>) >= 0.02` to avoid denominator noise.

## Verdict

The per-vector spread is not negligible. The renorm branch should stay open and proceed to Slice 2: apply length renormalization in the no-QJL per-candidate epilogue and A/B it independently.

## Validation Notes

- Passed: `cargo check -p ecaz-cli`
- Passed: `cargo build -p ecaz-cli --release`
- Blocked existing test compile: `cargo test -p ecaz-cli distribution_percentiles_are_interpolated` failed in existing `suite.rs` test initializers before reaching the new audit test (`runner_git_commit`, `ivf_stage_counters` fields).

No push was performed per handoff.
