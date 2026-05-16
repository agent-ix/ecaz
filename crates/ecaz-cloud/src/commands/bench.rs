use clap::Args;
use color_eyre::eyre::{eyre, Result};
use std::path::PathBuf;
use tokio::process::Command;

use crate::{aws, profiles::Profile, terraform::Terraform};

#[derive(Args, Debug)]
pub struct BenchArgs {
    #[arg(long)]
    pub profile: Profile,

    /// Path to a `bench suite run` config (JSON). When omitted, runs the
    /// built-in smoke suite from `crates/ecaz-cli/benches/smoke.json` if
    /// present, otherwise errors with a remediation message.
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Override the suite name used for the artifact prefix in S3.
    #[arg(long, default_value = "smoke")]
    pub suite: String,

    /// Database to connect to on the remote host.
    #[arg(long, default_value = "postgres")]
    pub database: String,

    /// Path to the local `ecaz` binary. Defaults to whichever `ecaz` is
    /// on $PATH.
    #[arg(long, default_value = "ecaz")]
    pub ecaz_bin: String,

    /// Skip the S3 upload step. Useful for offline iteration.
    #[arg(long)]
    pub skip_upload: bool,
}

impl BenchArgs {
    pub async fn run(self, repo_root: PathBuf) -> Result<()> {
        aws::ensure_credentials().await?;
        let tf = Terraform::new(self.profile, &repo_root)?;
        if !tf.state_exists() {
            return Err(eyre!("no stack for profile {}", self.profile));
        }
        let out = tf.outputs().await?;

        let run_id = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let artifacts_dir = repo_root
            .join("review")
            .join(format!("cloud-{}-{}", self.profile, run_id))
            .join("artifacts");
        tokio::fs::create_dir_all(&artifacts_dir).await?;
        let log_file = artifacts_dir.join("suite.log");

        let config = self
            .config
            .clone()
            .unwrap_or_else(|| repo_root.join("crates/ecaz-cli/benches/smoke.json"));
        if !config.exists() {
            return Err(eyre!(
                "bench suite config not found at {}; pass --config",
                config.display()
            ));
        }

        let mut cmd = Command::new(&self.ecaz_bin);
        cmd.env("PGHOST", &out.db_private_ip)
            .env("PGPORT", "5432")
            .env("PGUSER", "postgres")
            .env("PGDATABASE", &self.database)
            .arg("--log-file")
            .arg(&log_file)
            .arg("bench")
            .arg("suite")
            .arg("run")
            .arg("--config")
            .arg(&config);
        tracing::info!(
            db = %out.db_private_ip,
            log = %log_file.display(),
            "running ecaz bench suite against remote DSN"
        );
        let status = cmd.status().await?;
        if !status.success() {
            return Err(eyre!("ecaz bench suite exited {status}"));
        }

        if !self.skip_upload {
            let dest = format!(
                "s3://{}/bench-artifacts/{}/{}/",
                out.s3_bucket, self.suite, run_id
            );
            let s3 = Command::new("aws")
                .args([
                    "s3",
                    "sync",
                    artifacts_dir.to_str().expect("utf8 artifacts dir"),
                    &dest,
                    "--region",
                    &out.region,
                    "--only-show-errors",
                ])
                .status()
                .await?;
            if !s3.success() {
                return Err(eyre!("aws s3 sync to {dest} failed"));
            }
            println!("bench: uploaded artifacts to {dest}");
        }

        println!(
            "bench: profile={} suite={} run={} log={}",
            self.profile,
            self.suite,
            run_id,
            log_file.display()
        );
        Ok(())
    }
}
