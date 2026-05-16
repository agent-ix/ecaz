use clap::Args;
use color_eyre::eyre::{eyre, Result};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

use crate::{aws, profiles::Profile, state, terraform::Terraform};

#[derive(Args, Debug)]
pub struct ResumeArgs {
    #[arg(long)]
    pub profile: Profile,

    /// How long to wait for the DB instance to reach `running`.
    #[arg(long, default_value = "300")]
    pub wait_secs: u64,
}

impl ResumeArgs {
    pub async fn run(self, repo_root: PathBuf) -> Result<()> {
        aws::ensure_credentials().await?;
        let tf = Terraform::new(self.profile, &repo_root)?;
        if !tf.state_exists() {
            return Err(eyre!("no stack to resume for profile {}", self.profile));
        }
        let out = tf.outputs().await?;
        aws::start_instances(&out.region, &[&out.db_instance_id, &out.loader_instance_id]).await?;

        let deadline = std::time::Instant::now() + Duration::from_secs(self.wait_secs);
        loop {
            let s = aws::describe_instance_state(&out.region, &out.db_instance_id).await?;
            if s == "running" {
                break;
            }
            if std::time::Instant::now() >= deadline {
                return Err(eyre!(
                    "DB instance did not reach `running` within {}s (current: {s})",
                    self.wait_secs
                ));
            }
            sleep(Duration::from_secs(5)).await;
        }

        let mut st = state::load(self.profile).await?;
        st.paused_at = None;
        state::save(self.profile, &st).await?;
        println!(
            "resume: profile={} db={} ready",
            self.profile, out.db_private_ip
        );
        Ok(())
    }
}
