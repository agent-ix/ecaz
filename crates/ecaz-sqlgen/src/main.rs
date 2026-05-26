//! `ecaz-sqlgen` — Task 46 §Exit Criteria #2: ECAZ-grammar SQL
//! generator that biases toward `<-> ` / `<#>` / `<=>` operators,
//! ec_diskann/ec_hnsw/ec_ivf indexes, REINDEX/VACUUM interleavings,
//! prepared statements with bound vector parameters, and partial /
//! expression indexes over the vector column.
//!
//! Two modes:
//!
//!   - `generate` (default): emit N statements to stdout (or to a
//!     committed seed-corpus file via `--out PATH`).
//!   - `execute`: stream statements to a live PG18 cluster via the
//!     `--dsn DSN` flag; collect SQL ERRORs and surface any PANIC.

use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use color_eyre::eyre::{eyre, Context, Result};
use ecaz_sqlgen::Generator;

#[derive(Parser, Debug)]
#[command(version, about = "ECAZ-grammar SQL generator (Task 46 sqlsmith-ecaz lane)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Emit N generated statements to stdout (or to --out PATH).
    Generate(GenerateArgs),
    /// Generate N statements and stream them to a PG18 cluster.
    Execute(ExecuteArgs),
}

#[derive(Args, Debug)]
struct GenerateArgs {
    /// Deterministic seed.
    #[arg(long, default_value_t = 42)]
    seed: u64,
    /// Number of statements (or statement groups, since templates 2/3/5
    /// emit a sequence). Default 64 keeps corpus files reviewable.
    #[arg(long, default_value_t = 64)]
    count: usize,
    /// Output path; defaults to stdout.
    #[arg(long)]
    out: Option<PathBuf>,
    /// Target table name; bound by `clap`'s default ident validation.
    #[arg(long, default_value = "ecaz_sqlgen_t")]
    table: String,
    /// Target vector column.
    #[arg(long, default_value = "embedding")]
    column: String,
}

#[derive(Args, Debug)]
struct ExecuteArgs {
    /// libpq-format DSN for the target cluster.
    #[arg(long, env = "SQLSMITH_DSN")]
    dsn: String,
    /// Deterministic seed (same semantics as `generate`).
    #[arg(long, default_value_t = 42)]
    seed: u64,
    /// Number of statements (groups) to run.
    #[arg(long, default_value_t = 64)]
    count: usize,
    /// Target table.
    #[arg(long, default_value = "ecaz_sqlgen_t")]
    table: String,
    /// Target column.
    #[arg(long, default_value = "embedding")]
    column: String,
    /// Stop on first cluster-killing failure (broken connection /
    /// PANIC). Clean SQL ERRORs are tolerated by default.
    #[arg(long)]
    fail_fast: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    match cli.command {
        Command::Generate(args) => run_generate(args),
        Command::Execute(args) => {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;
            rt.block_on(run_execute(args))
        }
    }
}

fn run_generate(args: GenerateArgs) -> Result<()> {
    let mut g = Generator::from_seed(args.seed);
    let mut out: Box<dyn Write> = match args.out {
        Some(p) => Box::new(File::create(&p).wrap_err_with(|| format!("create {}", p.display()))?),
        None => Box::new(io::stdout().lock()),
    };
    writeln!(out, "-- ecaz-sqlgen seed={} count={}", args.seed, args.count)?;
    for _ in 0..args.count {
        for stmt in g.one_statement(&args.table, &args.column) {
            writeln!(out, "{stmt}")?;
        }
    }
    Ok(())
}

async fn run_execute(args: ExecuteArgs) -> Result<()> {
    let (client, connection) = tokio_postgres::connect(&args.dsn, tokio_postgres::NoTls)
        .await
        .wrap_err("connect to --dsn")?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {e}");
        }
    });

    let mut g = Generator::from_seed(args.seed);
    let mut clean_errors = 0usize;
    let mut hard_failures = 0usize;

    for i in 0..args.count {
        for stmt in g.one_statement(&args.table, &args.column) {
            match client.simple_query(&stmt).await {
                Ok(_) => {}
                Err(e) => {
                    let msg = format!("{e}");
                    let lower = msg.to_lowercase();
                    if lower.contains("panic") || lower.contains("server closed") {
                        hard_failures += 1;
                        eprintln!("hard failure on stmt #{i}: {msg}\n  -- {stmt}");
                        if args.fail_fast {
                            return Err(eyre!("hard failure (panic / server closed): {msg}"));
                        }
                    } else {
                        clean_errors += 1;
                    }
                }
            }
        }
    }
    eprintln!(
        "ecaz-sqlgen execute summary: clean_errors={clean_errors} hard_failures={hard_failures}"
    );
    if hard_failures > 0 {
        return Err(eyre!(
            "{hard_failures} hard failures during ecaz-sqlgen execute; \
             see stderr for details"
        ));
    }
    Ok(())
}
