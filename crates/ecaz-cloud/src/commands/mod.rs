//! `ecaz cloud` subcommand surface.

use clap::Subcommand;
use color_eyre::eyre::Result;

mod bench;
mod corpus;
mod down;
mod install;
mod pause;
mod resume;
mod snapshot;
mod status;
mod up;

#[derive(Subcommand, Debug)]
pub enum CloudCommand {
    /// Provision the stack via terraform; install ecaz; create extension.
    Up(up::UpArgs),
    /// Re-run extension install on the DB host (idempotent).
    Install(install::InstallArgs),
    /// Stage and load corpora.
    Corpus {
        #[command(subcommand)]
        cmd: corpus::CorpusCommand,
    },
    /// Run the bench suite against the remote DSN; upload artifacts to S3.
    Bench(bench::BenchArgs),
    /// Stop EC2 instances; retain EBS data. Restore via `resume`.
    Pause(pause::PauseArgs),
    /// Start previously paused instances; wait for Postgres.
    Resume(resume::ResumeArgs),
    /// Create an EBS snapshot of the DB volume; record the id locally.
    Snapshot(snapshot::SnapshotArgs),
    /// `terraform destroy`; retains snapshots/bucket unless asked otherwise.
    Down(down::DownArgs),
    /// Print stack state, instance ids, $/hr, and recommended next verb.
    Status(status::StatusArgs),
}

impl CloudCommand {
    pub async fn run(self, repo_root: std::path::PathBuf) -> Result<()> {
        match self {
            CloudCommand::Up(args) => args.run(repo_root).await,
            CloudCommand::Install(args) => args.run(repo_root).await,
            CloudCommand::Corpus { cmd } => cmd.run(repo_root).await,
            CloudCommand::Bench(args) => args.run(repo_root).await,
            CloudCommand::Pause(args) => args.run(repo_root).await,
            CloudCommand::Resume(args) => args.run(repo_root).await,
            CloudCommand::Snapshot(args) => args.run(repo_root).await,
            CloudCommand::Down(args) => args.run(repo_root).await,
            CloudCommand::Status(args) => args.run(repo_root).await,
        }
    }
}
