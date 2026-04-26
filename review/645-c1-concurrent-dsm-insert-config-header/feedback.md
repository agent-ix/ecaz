# Feedback: 645 Concurrent DSM Insert Config Header

## Verdict: Accept

Storing insert/scoring metadata (`dimensions`, `bits`, `seed`, `m`,
`ef_construction`) in the graph header is the correct worker-attach contract.
Workers cannot read `BuildState`; all scoring and insertion parameters must be
available from the DSM base pointer alone.

`None` for empty graphs is correct — no workers will attempt graph insertion
on a zero-node graph, so the validation path through
`concurrent_dsm_insert_config_from_image` correctly returns early.

Validation rejecting non-empty graphs with missing metadata is correctly placed
at attach time.

## No Issues
