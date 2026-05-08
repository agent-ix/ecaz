use clap::Args;
use color_eyre::eyre::{eyre, Result};
use std::path::PathBuf;

use crate::{aws, profiles::Profile, state, terraform::Terraform};

#[derive(Args, Debug)]
pub struct PauseArgs {
    #[arg(long)]
    pub profile: Profile,
}

impl PauseArgs {
    pub async fn run(self, repo_root: PathBuf) -> Result<()> {
        aws::ensure_credentials().await?;
        let tf = Terraform::new(self.profile, &repo_root)?;
        if !tf.state_exists() {
            return Err(eyre!("no stack to pause for profile {}", self.profile));
        }
        let out = tf.outputs().await?;
        aws::stop_instances(
            &out.region,
            &[&out.db_instance_id, &out.loader_instance_id],
        )
        .await?;
        let mut st = state::load(self.profile).await?;
        st.paused_at = Some(chrono::Utc::now());
        state::save(self.profile, &st).await?;
        println!("pause: profile={} stopped (db + loader)", self.profile);
        Ok(())
    }
}
