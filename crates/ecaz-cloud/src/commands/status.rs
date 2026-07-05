use clap::Args;
use color_eyre::eyre::Result;
use serde::Serialize;
use std::path::PathBuf;

use crate::{aws, profiles::Profile, state, terraform::Terraform};

#[derive(Args, Debug)]
pub struct StatusArgs {
    #[arg(long)]
    pub profile: Profile,

    /// Emit JSON instead of a human table (FR-044-AC-3).
    #[arg(long)]
    pub json: bool,
}

#[derive(Serialize)]
struct StatusReport {
    profile: String,
    state: String,
    db_instance_id: Option<String>,
    db_instance_state: Option<String>,
    db_private_ip: Option<String>,
    s3_bucket: Option<String>,
    last_snapshot_id: Option<String>,
    paused_at: Option<chrono::DateTime<chrono::Utc>>,
    estimated_hourly_usd: f64,
    retained_monthly_usd: f64,
    recommendation: Option<String>,
}

impl StatusArgs {
    pub async fn run(self, repo_root: PathBuf) -> Result<()> {
        let tf = Terraform::new(self.profile, &repo_root)?;
        let st = state::load(self.profile).await?;

        let mut report = StatusReport {
            profile: self.profile.to_string(),
            state: "down".into(),
            db_instance_id: None,
            db_instance_state: None,
            db_private_ip: None,
            s3_bucket: None,
            last_snapshot_id: st.last_snapshot_id.clone(),
            paused_at: st.paused_at,
            estimated_hourly_usd: 0.0,
            retained_monthly_usd: self.profile.estimated_monthly_storage_usd(),
            recommendation: None,
        };

        if tf.state_exists() {
            // ensure_credentials is best-effort here — status should still
            // print local state if the user is offline.
            if aws::ensure_credentials().await.is_ok() {
                if let Ok(out) = tf.outputs().await {
                    report.db_instance_id = Some(out.db_instance_id.clone());
                    report.db_private_ip = Some(out.db_private_ip);
                    report.s3_bucket = Some(out.s3_bucket);
                    report.db_instance_state =
                        aws::describe_instance_state(&out.region, &out.db_instance_id)
                            .await
                            .ok();
                    report.state = match report.db_instance_state.as_deref() {
                        Some("running") => {
                            report.estimated_hourly_usd = self.profile.estimated_hourly_usd();
                            "running"
                        }
                        Some("stopped") => "paused",
                        Some(other) => other,
                        None => "unknown",
                    }
                    .into();
                }
            } else {
                report.state = "unknown".into();
            }
        }

        if let Some(paused_at) = st.paused_at {
            let days = (chrono::Utc::now() - paused_at).num_days();
            if days > 7 {
                report.recommendation = Some(format!(
                    "paused {} days; consider `ecaz cloud snapshot && ecaz cloud down --yes` to drop EBS charges",
                    days
                ));
            }
        }

        if self.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            println!("profile:  {}", report.profile);
            println!("state:    {}", report.state);
            if let Some(id) = &report.db_instance_id {
                println!(
                    "db:       {} ({})",
                    report.db_private_ip.as_deref().unwrap_or("?"),
                    id
                );
            }
            if let Some(b) = &report.s3_bucket {
                println!("bucket:   {b}");
            }
            if let Some(s) = &report.last_snapshot_id {
                println!("snapshot: {s}");
            }
            println!(
                "cost:     ~${:.2}/hr running, ~${:.2}/mo retained storage",
                report.estimated_hourly_usd, report.retained_monthly_usd
            );
            if let Some(rec) = &report.recommendation {
                println!("note:     {rec}");
            }
        }
        Ok(())
    }
}
