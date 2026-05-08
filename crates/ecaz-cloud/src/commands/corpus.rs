use clap::{Args, Subcommand};
use color_eyre::eyre::Result;
use std::path::PathBuf;

use crate::profiles::Profile;

#[derive(Subcommand, Debug)]
pub enum CorpusCommand {
    /// List datasets known to the registry (FR-046).
    ListDatasets {
        #[arg(long)]
        json: bool,
    },
    /// Upload parquet shards for a dataset to the profile's S3 bucket.
    Stage(StageArgs),
    /// Fan out parquet → COPY workers on the loader EC2 (FR-047).
    Load(LoadArgs),
}

#[derive(Args, Debug)]
pub struct StageArgs {
    #[arg(long)]
    pub profile: Profile,
    #[arg(long)]
    pub dataset: String,
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct LoadArgs {
    #[arg(long)]
    pub profile: Profile,
    #[arg(long)]
    pub dataset: String,
    #[arg(long)]
    pub resume: bool,
    /// Stop the loader EC2 after load completes.
    #[arg(long)]
    pub keep_loader: bool,
}

impl CorpusCommand {
    pub async fn run(self, _repo_root: PathBuf) -> Result<()> {
        // Registry + staging + fan-out land in checkpoints 5-7. This
        // skeleton makes the surface visible so docs and tests can target
        // a stable command tree.
        match self {
            CorpusCommand::ListDatasets { .. } => {
                eprintln!("corpus list-datasets: registry implementation lands in checkpoint 7");
            }
            CorpusCommand::Stage(args) => {
                eprintln!(
                    "corpus stage --profile {} --dataset {}: implementation in checkpoint 5",
                    args.profile, args.dataset
                );
            }
            CorpusCommand::Load(args) => {
                eprintln!(
                    "corpus load --profile {} --dataset {}: implementation in checkpoint 5",
                    args.profile, args.dataset
                );
            }
        }
        Ok(())
    }
}
