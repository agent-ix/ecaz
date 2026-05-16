use clap::Args;
use color_eyre::eyre::Result;
use std::path::PathBuf;

use crate::{aws, profiles::Profile, terraform::Terraform};

#[derive(Args, Debug)]
pub struct DownArgs {
    #[arg(long)]
    pub profile: Profile,

    /// Skip interactive confirmation.
    #[arg(long)]
    pub yes: bool,
}

impl DownArgs {
    pub async fn run(self, repo_root: PathBuf) -> Result<()> {
        let tf = Terraform::new(self.profile, &repo_root)?;
        if !tf.state_exists() {
            println!(
                "down: no terraform state for {}; nothing to do.",
                self.profile
            );
            return Ok(());
        }
        if !self.yes {
            eprintln!(
                "About to destroy stack for profile={}. Re-run with --yes to confirm.",
                self.profile
            );
            return Ok(());
        }
        aws::ensure_credentials().await?;
        tf.destroy().await?;
        println!("down: profile={} destroyed", self.profile);
        Ok(())
    }
}
