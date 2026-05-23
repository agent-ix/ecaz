use clap::Args;
use color_eyre::eyre::{eyre, Result};
use std::path::PathBuf;

use crate::{aws, profiles::Profile, terraform::Terraform};

#[derive(Args, Debug)]
pub struct DownArgs {
    #[arg(long)]
    pub profile: Profile,

    /// Skip interactive confirmation.
    #[arg(long)]
    pub yes: bool,

    /// Allow destroy when the data volume has no recent EBS
    /// snapshot. Use only when the volume contents are intentionally
    /// disposable (e.g., one-shot smoke runs that produced no
    /// reusable corpus or index). Default behavior refuses to
    /// destroy without a snapshot to prevent re-paying for corpus
    /// load + index build on the next round.
    #[arg(long)]
    pub no_snapshot_required: bool,
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

        // Snapshot-before-destroy invariant (docs/aws-bench-workflow.md).
        // Refuse to tear the stack down if the data volume has no EBS
        // snapshot covering it — losing the volume costs hours of
        // corpus + index work. The user can opt out with
        // `--no-snapshot-required` for genuinely disposable runs.
        if !self.no_snapshot_required {
            let out = tf.outputs().await?;
            match aws::latest_snapshot_for_volume(&out.region, &out.db_volume_id).await? {
                Some((id, started)) => {
                    println!(
                        "down: profile={} data volume {} has snapshot {id} (started {started}); proceeding.",
                        self.profile, out.db_volume_id
                    );
                }
                None => {
                    return Err(eyre!(
                        "ec_cloud down refused: data volume {} for profile={} has no EBS snapshot. \
                         Run `ecaz cloud snapshot --profile {}` first, or re-run with \
                         `--no-snapshot-required` if the volume contents are intentionally disposable.",
                        out.db_volume_id,
                        self.profile,
                        self.profile,
                    ));
                }
            }
        }

        tf.destroy().await?;
        println!("down: profile={} destroyed", self.profile);
        Ok(())
    }
}
