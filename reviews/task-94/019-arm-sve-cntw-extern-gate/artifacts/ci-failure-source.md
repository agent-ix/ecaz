# Existing CI Failure Source

- PR: https://github.com/agent-ix/ecaz/pull/19
- Existing failed checks inspected; no CI rerun was requested.
- `pg18 / stable / compile`: https://github.com/agent-ix/ecaz/actions/runs/27228790771/job/80403035467
- `pg18 / stable`: https://github.com/agent-ix/ecaz/actions/runs/27228791172/job/80403036447

Both failures reported the same warning-as-error on aarch64:

```text
error: function `ecaz_grouped_pq_sve_cntw` is never used
```

Root cause: packet 015 moved runtime vector-lane reporting behind `#[cfg(test)]`,
but the Rust extern declaration for `ecaz_grouped_pq_sve_cntw` remained visible
in non-test aarch64 builds. The symbol is still emitted by the SVE assembly, but
the Rust declaration is only needed by the test-only runtime lane helper.
