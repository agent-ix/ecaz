# Review Request: Pin `hnsw_rs` Version

Scope:
- `Cargo.toml`

What changed:
- Replaced the wildcard `hnsw_rs = "*"` dependency with the currently locked `hnsw_rs = "0.3.4"` version.
- No code paths changed; this slice only narrows dependency resolution for reproducibility.

Review focus:
- Whether pinning to the currently locked major.minor.patch is the right dependency boundary for this repository
- Whether any related manifest or lockfile follow-up is needed for this narrow reproducibility fix

Questions to answer:
- Is `0.3.4` the right level of pinning here, or should the manifest allow a broader semver range?
- Is there any reason to update `Cargo.lock` as part of this slice, or is the manifest-only change sufficient?
- Are there any other wildcard or overly broad dependency specs in the repo that should be brought into the same review thread later?
