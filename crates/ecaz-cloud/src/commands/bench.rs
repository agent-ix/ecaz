use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use std::path::PathBuf;
use tokio::process::Command;

use crate::{aws, profiles::Profile, ssm, terraform::Terraform};

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

        let artifacts_dir = suite_artifacts_dir(&repo_root, &config, self.profile, &run_id).await?;
        tokio::fs::create_dir_all(&artifacts_dir).await?;

        let dest = format!(
            "s3://{}/bench-artifacts/{}/{}/",
            out.s3_bucket, self.suite, run_id
        );
        let script = remote_suite_script(
            &repo_root,
            &config,
            &artifacts_dir,
            &self.database,
            &dest,
            &out.region,
            self.skip_upload,
        )
        .await?;
        tracing::info!(
            db_instance = %out.db_instance_id,
            artifacts = %artifacts_dir.display(),
            "ssm: remote bench suite"
        );
        ssm::run_shell(&out.region, &out.db_instance_id, &script, 21600).await?;

        if !self.skip_upload {
            let s3 = Command::new("aws")
                .args([
                    "s3",
                    "sync",
                    &dest,
                    artifacts_dir.to_str().expect("utf8 artifacts dir"),
                    "--region",
                    &out.region,
                    "--only-show-errors",
                ])
                .status()
                .await?;
            if !s3.success() {
                return Err(eyre!("aws s3 sync from {dest} failed"));
            }
            println!("bench: synced artifacts from {dest}");
        }

        println!(
            "bench: profile={} suite={} run={} log={}",
            self.profile,
            self.suite,
            run_id,
            artifacts_dir.join("suite-run.log").display()
        );
        Ok(())
    }
}

async fn suite_artifacts_dir(
    repo_root: &std::path::Path,
    config: &std::path::Path,
    profile: Profile,
    run_id: &str,
) -> Result<PathBuf> {
    let text = tokio::fs::read_to_string(config)
        .await
        .with_context(|| format!("read suite config {}", config.display()))?;
    let json: serde_json::Value = serde_json::from_str(&text)
        .with_context(|| format!("parse suite config {}", config.display()))?;
    if let Some(dir) = json.get("artifact_dir").and_then(|v| v.as_str()) {
        let path = PathBuf::from(dir);
        return Ok(if path.is_absolute() {
            path
        } else {
            repo_root.join(path)
        });
    }
    Ok(repo_root
        .join("review")
        .join(format!("cloud-{}-{}", profile, run_id))
        .join("artifacts"))
}

async fn remote_suite_script(
    repo_root: &std::path::Path,
    config: &std::path::Path,
    artifacts_dir: &std::path::Path,
    database: &str,
    s3_dest: &str,
    region: &str,
    skip_upload: bool,
) -> Result<String> {
    let config_text = tokio::fs::read_to_string(config)
        .await
        .with_context(|| format!("read suite config {}", config.display()))?;
    let remote_root = "/var/lib/pgsql/build/ecaz";
    let remote_config = relative_to_repo(repo_root, config)?;
    let remote_artifacts = relative_to_repo(repo_root, artifacts_dir)?;
    let upload = if skip_upload {
        String::new()
    } else {
        format!(
            "aws s3 sync {} {} --region {} --only-show-errors",
            shell_escape(&remote_artifacts),
            shell_escape(s3_dest),
            shell_escape(region)
        )
    };
    let run_cmd = format!(
        "cd {root}; export PATH=$HOME/.cargo/bin:$PATH; target/release/ecaz --database {db} --host /var/run/postgresql --user postgres --log-file {log} bench suite run --config {config} --manifest-output {manifest} --results-output {results}",
        root = shell_escape(remote_root),
        db = shell_escape(database),
        log = shell_escape(&format!("{remote_artifacts}/suite-run.log")),
        config = shell_escape(&remote_config),
        manifest = shell_escape(&format!("{remote_artifacts}/suite-manifest.json")),
        results = shell_escape(&format!("{remote_artifacts}/results.jsonl")),
    );

    Ok(format!(
        r#"#!/usr/bin/env bash
set -euxo pipefail
cd {root}
mkdir -p "$(dirname {config_path})" {artifacts}
cat > {config_path} <<'ECAZ_SUITE_CONFIG'
{config_text}
ECAZ_SUITE_CONFIG
chown -R postgres:postgres "$(dirname {config_path})" {artifacts}
sudo -u postgres bash -lc {run_cmd}
{upload}
"#,
        root = shell_escape(remote_root),
        config_path = shell_escape(&remote_config),
        artifacts = shell_escape(&remote_artifacts),
        run_cmd = shell_escape(&run_cmd),
    ))
}

fn relative_to_repo(repo_root: &std::path::Path, path: &std::path::Path) -> Result<String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    };
    let relative = absolute
        .strip_prefix(repo_root)
        .with_context(|| format!("{} is outside {}", absolute.display(), repo_root.display()))?;
    Ok(relative.to_string_lossy().into_owned())
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
