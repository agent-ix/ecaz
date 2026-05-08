use clap::Args;
use color_eyre::eyre::Result;
use std::path::PathBuf;

use crate::profiles::Profile;

#[derive(Args, Debug)]
pub struct InstallArgs {
    #[arg(long)]
    pub profile: Profile,

    /// Override the git ref used by the cloud-init build.
    #[arg(long)]
    pub git_ref: Option<String>,
}

impl InstallArgs {
    pub async fn run(self, _repo_root: PathBuf) -> Result<()> {
        // The DB host's cloud-init script (infra/cloud/terraform/cloud-init/db.sh.tftpl)
        // already builds and installs ecaz on first boot. This verb is the
        // re-install path: SSM-exec the same build commands at a new git ref.
        // Stub for checkpoint 4.
        eprintln!(
            "install: profile={} (cloud-init handles first boot; re-install via SSM is checkpoint 4)",
            self.profile
        );
        Ok(())
    }
}
