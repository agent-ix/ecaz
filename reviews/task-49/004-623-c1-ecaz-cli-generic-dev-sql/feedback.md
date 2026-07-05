# Feedback: 623 Generic ecaz dev SQL Runner

## Verdict: Accept

`dev sql` is the right surface. The implementation is clean and the PG18
default migration is correct.

## Findings

**`dev sql` vs scratch extension**: Correct call. `dev scratch sql` is tied to
the scratch database lifecycle (start/stop/reset). `dev sql` is a general
psql driver against any pgrx install. They have different scope; sharing the
command would require a database-lifecycle flag that doesn't belong in a
generic runner.

**`--log-output` tee behavior**: Writing combined stdout/stderr to a file while
echoing to the terminal is the right semantic for review-packet artifact
capture. Avoids the shell redirection / `script` workarounds that the request
describes.

**`--pg 18` default**: Correct. PG18 is the current primary target. All dev
commands defaulting to the same version reduces accidental cross-version test
pollution. `--pg 17` remains available via explicit opt-in.

**`default_pgrx_port(pg: u16) = 28800 + pg`**: Matches the pgrx convention.
Shared in `support.rs` so `dev scratch` and `dev test pgrx` use the same
formula.

**Mutual exclusion of `--sql` and `--file`**: Correctly enforced with `bail!`.
Neither required when the other is present; both required to be absent only
together — the validation checks both cases.

**`ON_ERROR_STOP=1`**: Always set. Correct default for review-packet SQL runs
where a partial execution would produce misleading timing data.

**`-A -t -F "\t"` default output**: Tab-separated tuples-only is correct for
log output that gets stored as packet artifacts. `--raw` bypasses this for
interactive use.

**`cargo fmt --check` not clean**: The request notes pre-existing formatting
drift outside touched files. This is acceptable for a contained CLI packet —
the touched files are formatted, and sweeping unrelated files would pollute the
diff.

## No Issues
