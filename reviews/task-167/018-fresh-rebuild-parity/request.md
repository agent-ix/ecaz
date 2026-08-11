# Task 167 checkpoint: post-insert fresh-rebuild parity

The physical benchmark now performs the required post-insert oracle check.
After the physical-vs-local insert throughput A/B, it creates a fresh local
`ec_distann` index from the physical table's current rows, queries both indexes
for the configured held-out queries, computes distinct top-10 ID overlap, and
requires complete ten-row results from both surfaces. It emits
`physical_benchmark_post_insert_fresh_rebuild` with the mean, minimum, and
maximum distinct recall.

`cargo check -p ecaz-cli` passed; see `artifacts/validation.log`. Runtime
execution remains pending because this host lacks the installed `ecaz`
operator binary and `data/staged-current`. This packet is review-open and does
not claim Task 167 closeout.
