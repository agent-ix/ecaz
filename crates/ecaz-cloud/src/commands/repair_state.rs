use clap::Args;
use color_eyre::eyre::{eyre, Result};
use std::path::PathBuf;

use crate::{profiles::Profile, state, terraform::Terraform};

const DB_VOLUME_RESOURCES: &[&str] = &["aws_volume_attachment.db_data", "aws_ebs_volume.db"];

#[derive(Args, Debug)]
pub struct RepairStateArgs {
    /// Profile name whose local Terraform state should be repaired.
    #[arg(long)]
    pub profile: Profile,

    /// Forget the DB EBS volume resources from local Terraform state.
    ///
    /// Use this only after the data volume has a recorded snapshot; the next
    /// `cloud up --from-snapshot` can then create a fresh restored volume.
    #[arg(long)]
    pub forget_stale_db_volume: bool,

    /// Print the state entries that would be removed without changing state.
    #[arg(long)]
    pub dry_run: bool,
}

impl RepairStateArgs {
    pub async fn run(self, repo_root: PathBuf) -> Result<()> {
        if !self.forget_stale_db_volume {
            return Err(eyre!(
                "nothing to repair; pass --forget-stale-db-volume to remove stale DB volume state"
            ));
        }

        let st = state::load(self.profile).await?;
        let Some(snapshot_id) = st.last_snapshot_id.as_deref() else {
            return Err(eyre!(
                "refusing to forget DB volume state for profile {} without a recorded snapshot",
                self.profile
            ));
        };

        let tf = Terraform::new(self.profile, &repo_root)?;
        if !tf.state_exists() {
            println!(
                "repair-state: no Terraform state exists for profile={}",
                self.profile
            );
            return Ok(());
        }

        let existing = tf.state_list().await?;
        let targets = DB_VOLUME_RESOURCES
            .iter()
            .filter(|address| existing.iter().any(|entry| entry == **address))
            .map(|address| (*address).to_owned())
            .collect::<Vec<_>>();

        if targets.is_empty() {
            println!(
                "repair-state: no DB volume resources found for profile={} snapshot={}",
                self.profile, snapshot_id
            );
            return Ok(());
        }

        println!(
            "repair-state: profile={} snapshot={} targets={}",
            self.profile,
            snapshot_id,
            targets.join(",")
        );

        if self.dry_run {
            println!("repair-state: dry-run only; local Terraform state unchanged");
            return Ok(());
        }

        tf.state_rm(&targets).await?;
        println!(
            "repair-state: removed {} Terraform state entries",
            targets.len()
        );
        Ok(())
    }
}
