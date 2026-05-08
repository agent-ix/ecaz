use clap::Args;
use color_eyre::eyre::Result;
use std::path::PathBuf;

use crate::profiles::Profile;

#[derive(Args, Debug)]
pub struct BenchArgs {
    #[arg(long)]
    pub profile: Profile,

    /// Suite name passed through to `ecaz bench suite run`.
    #[arg(long, default_value = "smoke")]
    pub suite: String,
}

impl BenchArgs {
    pub async fn run(self, _repo_root: PathBuf) -> Result<()> {
        // Implementation lands in checkpoint 6: build a libpq DSN from
        // terraform outputs, invoke the existing `ecaz bench` entry
        // points against it, upload --log-file artifacts to S3.
        eprintln!(
            "bench --profile {} --suite {}: implementation in checkpoint 6",
            self.profile, self.suite
        );
        Ok(())
    }
}
