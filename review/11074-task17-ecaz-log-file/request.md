# Review Request: First-class `ecaz --log-file` packet capture

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/main.rs`
- `crates/ecaz-cli/src/cli.rs`
- `crates/ecaz-cli/src/output.rs`
- `crates/ecaz-cli/src/commands/corpus/list.rs`
- `crates/ecaz-cli/src/commands/corpus/inspect.rs`
- `crates/ecaz-cli/src/commands/corpus/load.rs`
- `crates/ecaz-cli/src/commands/corpus/prepare.rs`
- `crates/ecaz-cli/src/commands/corpus/generate.rs`
- `crates/ecaz-cli/src/commands/corpus/fetch.rs`
- `crates/ecaz-cli/src/commands/bench/recall.rs`
- `crates/ecaz-cli/src/commands/bench/latency.rs`
- `crates/ecaz-cli/src/commands/bench/overhead.rs`
- `crates/ecaz-cli/src/commands/bench/storage.rs`
- `crates/ecaz-cli/src/commands/compare/pgvector.rs`
- `crates/ecaz-cli/src/commands/stress/vacuum.rs`
- `crates/ecaz-cli/src/commands/dev/install.rs`
- `crates/ecaz-cli/src/commands/dev/scratch.rs`
- `crates/ecaz-cli/src/commands/dev/test.rs`
- `crates/ecaz-cli/README.md`

## What this packet is

This is a generic `ecaz-cli` improvement that needs to stand alone so it can
merge to `main` independently of the DiskANN closeout lane.

The immediate blocker was review-packet capture. I hit the approval gate twice
in one session because the only way to archive `ecaz` output was shell
plumbing (`tee` / redirection) around `cargo run -p ecaz-cli -- ...`.

That is the wrong boundary. If packet-local raw logs are part of the normal
operator workflow, `ecaz-cli` needs a first-class output surface for them.

## What changed

### Global CLI surface

`crates/ecaz-cli/src/cli.rs` now adds:

```rust
#[arg(long, global = true)]
pub log_file: Option<PathBuf>,
```

with help text that makes the intended use explicit:

- mirror command stdout/stderr into a packet-local artifact file
- suppress transient progress bars so the artifact stays stable and diffable

Also added a parser test pinning:

```text
ecaz --log-file review/11074-task17-ecaz-log-file/artifacts/load.log corpus list
```

### Shared output plumbing

New module: `crates/ecaz-cli/src/output.rs`

It provides:

- `output::init(path)` to open the optional log file once at process startup
- mirrored stdout/stderr writers used by two macros:
  - `crate::ecaz_println!`
  - `crate::ecaz_eprintln!`
- `output::progress_bar(len)` so commands can suppress redraw-heavy progress
  bars when `--log-file` is active
- `output::StderrMirror` so tracing output follows the same sink

### Top-level error capture

`crates/ecaz-cli/src/main.rs` no longer returns `Result<()>` directly from
`main`. Instead it routes errors through the mirrored stderr path:

```rust
match try_main().await {
    Ok(()) => ExitCode::SUCCESS,
    Err(err) => {
        crate::ecaz_eprintln!("{err:?}");
        ExitCode::FAILURE
    }
}
```

That is important for packet capture: a failed `ecaz corpus load` or
`bench recall` now lands its final error report in `--log-file`, not only on
the terminal.

### Command tree updates

The command modules that previously used raw `println!` / `eprintln!` now go
through the shared mirrored-output macros instead. This keeps:

- terminal UX unchanged for normal runs
- packet capture first-class for `--log-file` runs

The affected command surfaces are:

- `corpus`
- `bench`
- `compare`
- `stress`
- `dev`

This is intentionally plumbing only. No SQL semantics, benchmark math, or
profile behavior changed.

### README

`crates/ecaz-cli/README.md` now documents `--log-file` in the install / usage
section and shows it in the canonical real-corpus flow example so packet
capture no longer depends on shell `tee`.

## Why this slice

- The need is generic, not DiskANN-specific. If the operator surface
  frequently needs shell wrapping just to archive its own output, the surface
  is incomplete.
- Keeping it isolated makes it safe to cherry-pick or merge directly to
  `main` without dragging along the task-17 measurement packet.
- This lets the next DiskANN slice run the canonical path directly:

```text
ecaz ... --log-file review/.../artifacts/load.log corpus load ...
ecaz ... --log-file review/.../artifacts/recall.log bench recall ...
```

No `tee`, no shell redirection, no approval surprises from wrapper syntax.

## Operator smoke

Direct CLI smoke, no shell capture:

```text
cargo run -p ecaz-cli -- \
  --host /home/peter/.pgrx \
  --port 28818 \
  --database postgres \
  --log-file review/11074-task17-ecaz-log-file/artifacts/corpus-list.log \
  corpus list
```

Terminal output:

```text
(no corpora loaded in postgres)
```

Captured artifact:

- `review/11074-task17-ecaz-log-file/artifacts/corpus-list.log`

Contents:

```text
(no corpora loaded in postgres)
```

## Test evidence

```text
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 219 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also passed for this checkpoint on `pg18`:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- The actual pg18 real-10k DiskANN load / Recall@10 artifact. That remains the
  next task-17 slice on top of this generic logging fix.
- Structured machine-readable report formats (`--json`, CSV export, etc.).
  This packet is only about making plain-text packet capture first-class.
- A richer progress-log surface that emits checkpoint lines for every progress
  bar tick. For now `--log-file` captures the stable textual output and
  suppresses transient redraw noise.
