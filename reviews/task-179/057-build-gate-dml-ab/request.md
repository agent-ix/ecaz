# Review request: inactive build-gate DML A/B

## Scope

Please review the canonical post-fix A/B for packet 007's remaining P2-4
ordinary-DML hot-path concern. The measured implementation is
`a4d374c2f294dc209b1b0f499bd527e52b375b06` (packet 058); structured raw
suite results are provided by `0474ef90983de8acfc64022e3d548ec0bcae7062`
(packet 056).

This packet supplies the owed A/B data point and explicit acceptance bars. It
does not independently close Task 179 or make a broader transaction-throughput
claim.

## A/B design

The nine-step `ecaz bench suite` creates two fresh databases from `template0`
on one PG18 postmaster with `shared_preload_libraries=ecaz`:

- **control:** the library/hook is preloaded, but the `ecaz` extension is not
  installed, so `extension_is_installed()` returns false;
- **installed:** `CREATE EXTENSION ecaz` has run, no durable registration is
  active, and the new backend-local no-active-gate fast path is eligible.

Both arms use the same unlogged two-bigint heap, `synchronous_commit=off`,
`jit=off`, and a cached static PL/pgSQL single-row INSERT. Each round discards
one 25,000-statement warmup and records four 25,000-statement trials. ABBA
ordering (`control-1`, `installed-1`, `installed-2`, `control-2`) yields eight
measured trials / 200,000 measured INSERTs per arm.

Every persisted duration must be positive; the SQL fixture aborts if the host
wall clock steps backward across a trial. The two-round ABBA median and p95
reduce order and isolated outlier sensitivity.

## Result

```text
control:   median 6.988 us/statement, p95 7.489 us
installed: median 6.903 us/statement, p95 7.416 us
delta:    -0.085 us/statement
ratio:     0.988x median, 0.990x p95
```

The negative point estimate is treated as noise, not as a speedup claim. The
finding is that installed/no-active-gate overhead is below the suite's
resolution and passes all three declared performance bars:

- median ratio `<= 1.10x` (actual `0.988x`);
- median delta `<= +1.0 us` (actual `-0.085 us`);
- p95 ratio `<= 1.15x` (actual `0.990x`).

All six setup/sample-integrity thresholds also pass: the control has no
extension, the installed arm has exactly one, both report the preloaded
library, and both contribute exactly eight samples. Overall: 9/9 steps, 9/9
thresholds, zero failed/missing/stale artifacts.

## Reviewer focus

1. Confirm the database-level extension difference isolates the installed
   no-active-registration hook path while holding the preloaded postmaster,
   schema, SQL, and execution order constant.
2. Confirm each PL/pgSQL loop executes 25,000 individual cached INSERT
   statements, rather than one set-based statement that would pay the hook
   only once.
3. Confirm the warmup exclusion, ABBA ordering, positive-duration check, and
   eight-sample aggregate support the stated narrow conclusion.
4. Confirm packet 058's cross-backend invalidation regression plus this A/B
   discharge packet 007 P2-4 without weakening fail-closed behavior.

## Validation

- exact-head production PG18 extension install: pass;
- exact-head release CLI build: pass with one pre-existing unrelated dead-code
  warning;
- suite audit: pass, 9 steps;
- canonical suite: 9/9 steps and 9/9 thresholds pass;
- status: zero failed, missing, or stale artifacts.
