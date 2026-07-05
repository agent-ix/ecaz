# Task 28 IVF Insert Centroid Cache Trial

## Scope

This packet records a narrow live-insert optimization trial for the
centroid-model reload item called out in the Task 28 follow-on plan.

The attempted code checkpoint was `f2314bb` (`ivf: cache insert centroid
model`). It added a backend-local cache for the trained centroid model during
`aminsert`.

## Result

The 10-second synthetic PG18 insert harness did **not** support keeping this
change:

| checkpoint | concurrency | inserted rows/s | packet 30057 reference |
|---|---:|---:|---:|
| `f2314bb` centroid cache | 1 | 249.60 | 275.30 |
| `f2314bb` centroid cache | 4 | 635.50 | 657.50 |

The change was backed out in `ce7a2b0` (`ivf: back out insert centroid cache`).

## Interpretation

Centroid reload is not the current nlists=16 insert bottleneck. The assigned-list
duplicate scan, per-row posting append, directory update, and metadata update
remain better candidates for the next live-insert slice. This packet is kept so
we do not retread the same cache idea without a larger-list-specific benchmark.

## Artifacts

- `artifacts/manifest.md`
- `artifacts/ivf_insert_centroidcache_c1.log`
- `artifacts/ivf_insert_centroidcache_c4.log`

## Validation

Before the attempted code checkpoint:

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::insert --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_insert`
- `cargo pgrx test pg18 test_ec_ivf_large_build_insert_directory_chain`
- `git diff --check`

After backing it out:

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::insert --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `git diff --check`

## Next Slice

Do not land centroid caching from this trial. Continue with the live-insert
fixed per-row writes: directory and metadata update frequency, then the
duplicate-check/read path if correctness permits a narrower guard.
