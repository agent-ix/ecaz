use clap::Args;
use color_eyre::eyre::{eyre, Result};
use std::path::PathBuf;

use crate::{aws, profiles::Profile, state, terraform::Terraform};

#[derive(Args, Debug)]
pub struct SnapshotArgs {
    #[arg(long)]
    pub profile: Profile,

    /// Snapshot description tag.
    #[arg(long, default_value = "ecaz cloud snapshot")]
    pub description: String,
}

impl SnapshotArgs {
    pub async fn run(self, repo_root: PathBuf) -> Result<()> {
        aws::ensure_credentials().await?;
        let tf = Terraform::new(self.profile, &repo_root)?;
        if !tf.state_exists() {
            return Err(eyre!("no stack for profile {}", self.profile));
        }
        let out = tf.outputs().await?;
        let id = aws::create_snapshot(&out.region, &out.db_volume_id, &self.description).await?;
        let mut st = state::load(self.profile).await?;
        st.last_snapshot_id = Some(id.clone());
        state::save(self.profile, &st).await?;
        println!("snapshot: profile={} id={id}", self.profile);
        Ok(())
    }
}
