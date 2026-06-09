# Existing CI Failure Source

- PR: https://github.com/agent-ix/ecaz/pull/19
- Existing failed check inspected; no CI rerun was requested.
- `pg18 / stable`: https://github.com/agent-ix/ecaz/actions/runs/27229478676/job/80405533632
- Run head: `20eacf1c287c37c144e9efdb35070c07d42309fe`
- Runner: `macos-14-arm64`

The job passed `cargo check` and `cargo clippy`, then failed during
`cargo build -p ecaz-cli --bin ecaz` because the grouped-PQ SVE `global_asm!`
used ELF directives rejected by the Mach-O assembler:

```text
error: unknown directive
note: instantiated into assembly here
--> <inline asm>:6:5
6 |     .hidden ecaz_grouped_pq_sve_accumulate_f32

error: unknown directive
--> <inline asm>:7:5
7 |     .type ecaz_grouped_pq_sve_accumulate_f32, %function
```

Root cause: SVE/SVE2 is required for the Graviton 4 Linux target, but the
SVE assembly was compiled for all aarch64 targets, including Apple ARM. Apple
ARM does not provide the target SVE runtime path for this task and its assembler
does not accept the ELF visibility/type directives.

Fix: restrict the grouped-PQ SVE assembly, extern declarations, runtime SVE
detection, and SVE implementation helper to
`#[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]`. Apple
aarch64 now follows the existing scalar fallback path; Linux aarch64 keeps the
Graviton 4 SVE/SVE2 path.
