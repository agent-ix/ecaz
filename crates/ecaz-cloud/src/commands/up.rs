use clap::Args;
use color_eyre::eyre::{eyre, Result};
use std::path::PathBuf;

use crate::{aws, profiles::Profile, state, terraform::Terraform};

#[derive(Args, Debug)]
pub struct UpArgs {
    /// Profile name (10k, dev, 1m, 10m, 100m).
    #[arg(long)]
    pub profile: Profile,

    /// Git ref of ecaz to install on the DB host.
    #[arg(long, default_value = "main")]
    pub git_ref: String,

    /// Restore the DB data volume from this EBS snapshot id, skipping the
    /// initial load. Pair with a previously-recorded `snapshot`.
    #[arg(long)]
    pub from_snapshot: Option<String>,

    /// Required for profiles larger than `dev`: must match
    /// `Profile::estimated_daily_usd().round()`. Confirms the operator
    /// understands the projected daily spend (NFR-010).
    #[arg(long)]
    pub confirm_cost: Option<u64>,

    /// `terraform plan` only — do not apply.
    #[arg(long)]
    pub dry_run: bool,
}

impl UpArgs {
    pub async fn run(self, repo_root: PathBuf) -> Result<()> {
        if matches!(self.profile, Profile::P1m | Profile::P10m | Profile::P100m) {
            let projected = self.profile.estimated_daily_usd().round() as u64;
            match self.confirm_cost {
                Some(c) if c == projected => {}
                _ => {
                    return Err(eyre!(
                        "profile {} costs ~${}/day; pass --confirm-cost {} to proceed",
                        self.profile,
                        projected,
                        projected
                    ));
                }
            }
        }

        aws::ensure_credentials().await?;
        let tf = Terraform::new(self.profile, &repo_root)?;

        if self.dry_run {
            tf.init().await?;
            tf.plan().await?;
            return Ok(());
        }

        tf.init().await?;
        let mut vars: Vec<(&str, &str)> = vec![("ecaz_git_ref", self.git_ref.as_str())];
        if let Some(snap) = self.from_snapshot.as_deref() {
            vars.push(("from_snapshot_id", snap));
        }
        tf.apply(&vars).await?;

        let outputs = tf.outputs().await?;
        let mut st = state::load(self.profile).await?;
        st.last_dsn = Some(format!(
            "host={} port=5432 dbname=postgres user=postgres",
            outputs.db_private_ip
        ));
        st.paused_at = None;
        state::save(self.profile, &st).await?;

        println!("up: profile={} db={} bucket={}", self.profile, outputs.db_private_ip, outputs.s3_bucket);
        println!(
            "next: ecaz cloud install --profile {} (or `ecaz cloud corpus stage`)",
            self.profile
        );
        Ok(())
    }
}
