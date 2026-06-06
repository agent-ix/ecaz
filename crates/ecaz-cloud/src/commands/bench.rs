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

    /// Set ECAZ_SIMD in the remote PostgreSQL systemd environment and restart
    /// PostgreSQL before running the suite.
    #[arg(long)]
    pub simd_mode: Option<String>,
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
        let config_size = tokio::fs::metadata(&config)
            .await
            .with_context(|| format!("stat suite config {}", config.display()))?
            .len();
        let config_s3_uri = if self.skip_upload || config_size <= 7_000 {
            None
        } else {
            let uri = format!("{dest}suite-config.json");
            let status = Command::new("aws")
                .args([
                    "s3",
                    "cp",
                    config.to_str().expect("utf8 suite config path"),
                    &uri,
                    "--region",
                    &out.region,
                    "--only-show-errors",
                ])
                .status()
                .await?;
            if !status.success() {
                return Err(eyre!("aws s3 cp suite config to {uri} failed"));
            }
            Some(uri)
        };
        let script = remote_suite_script(
            &repo_root,
            &config,
            &artifacts_dir,
            &self.database,
            &self.ecaz_bin,
            &dest,
            config_s3_uri.as_deref(),
            &out.region,
            self.skip_upload,
            self.simd_mode.as_deref(),
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
    ecaz_bin: &str,
    s3_dest: &str,
    config_s3_uri: Option<&str>,
    region: &str,
    skip_upload: bool,
    simd_mode: Option<&str>,
) -> Result<String> {
    let config_text = tokio::fs::read_to_string(config)
        .await
        .with_context(|| format!("read suite config {}", config.display()))?;
    let remote_root = "/var/lib/pgsql/build/ecaz";
    let remote_config = relative_to_repo(repo_root, config)?;
    let remote_artifacts = relative_to_repo(repo_root, artifacts_dir)?;
    let upload = if skip_upload {
        ":".to_string()
    } else {
        format!(
            "aws s3 sync {} {} --region {} --only-show-errors || true",
            shell_escape(&remote_artifacts),
            shell_escape(s3_dest),
            shell_escape(region)
        )
    };
    let simd_export = simd_mode
        .map(|mode| format!("export ECAZ_SIMD={}; ", shell_escape(mode)))
        .unwrap_or_default();
    let run_cmd = format!(
        "cd {root}; export PATH=/usr/local/bin:$HOME/.cargo/bin:$PATH; {simd_export}{ecaz_bin} --database {db} --host /var/run/postgresql --user postgres --log-file {log} bench suite run --config {config} --manifest-output {manifest} --results-output {results}",
        root = shell_escape(remote_root),
        simd_export = simd_export,
        ecaz_bin = shell_escape(ecaz_bin),
        db = shell_escape(database),
        log = shell_escape(&format!("{remote_artifacts}/suite-run.log")),
        config = shell_escape(&remote_config),
        manifest = shell_escape(&format!("{remote_artifacts}/suite-manifest.json")),
        results = shell_escape(&format!("{remote_artifacts}/results.jsonl")),
    );
    let write_config = if let Some(uri) = config_s3_uri {
        format!(
            "aws s3 cp {} {} --region {} --only-show-errors\ntest -s {}",
            shell_escape(uri),
            shell_escape(&remote_config),
            shell_escape(region),
            shell_escape(&remote_config),
        )
    } else {
        format!(
            "cat > {} <<'ECAZ_SUITE_CONFIG'\n{}\nECAZ_SUITE_CONFIG",
            shell_escape(&remote_config),
            config_text
        )
    };
    let simd_setup = if let Some(mode) = simd_mode {
        format!(
            "sudo systemctl set-environment ECAZ_SIMD={mode}\n\
             sudo systemctl restart postgresql\n\
             sudo systemctl show postgresql -p Environment\n\
             sudo -u postgres psql -Atqc 'SHOW shared_preload_libraries;'",
            mode = shell_escape(mode),
        )
    } else {
        ":".to_string()
    };

    Ok(format!(
        r#"#!/usr/bin/env bash
set -euxo pipefail
cd {root}
mkdir -p "$(dirname {config_path})" {artifacts}
trap 'status=$?; set +e; {upload}; exit $status' EXIT
{write_config}
chown -R postgres:postgres "$(dirname {config_path})" {artifacts}
{simd_setup}
sudo -u postgres bash -lc {run_cmd}
"#,
        root = shell_escape(remote_root),
        config_path = shell_escape(&remote_config),
        artifacts = shell_escape(&remote_artifacts),
        write_config = write_config,
        simd_setup = simd_setup,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn remote_suite_script_exports_simd_for_cli_and_postgres() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let repo_root = std::env::temp_dir().join(format!("ecaz-cloud-bench-test-{unique}"));
        let config = repo_root.join("reviews/task-67/packet/artifacts/suite.json");
        fs::create_dir_all(config.parent().expect("config parent")).expect("create config parent");
        fs::write(&config, r#"{"name":"test","schema_version":1,"steps":[]}"#)
            .expect("write config");
        let artifacts = repo_root.join("reviews/task-67/packet/artifacts/scalar");

        let script = remote_suite_script(
            &repo_root,
            &config,
            &artifacts,
            "postgres",
            "/usr/local/bin/ecaz",
            "s3://bucket/suite/run/",
            None,
            "us-west-2",
            false,
            Some("scalar"),
        )
        .await
        .expect("build remote script");

        assert!(script.contains("sudo systemctl set-environment ECAZ_SIMD='scalar'"));
        assert!(script.contains("sudo systemctl restart postgresql"));
        assert!(
            script.contains(
                "export PATH=/usr/local/bin:$HOME/.cargo/bin:$PATH; export ECAZ_SIMD='\\''scalar'\\''; '\\''/usr/local/bin/ecaz'\\''"
            ),
            "{script}"
        );

        fs::remove_dir_all(repo_root).expect("remove temp repo root");
    }

    #[tokio::test]
    async fn remote_suite_script_downloads_uploaded_config_unconditionally() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let repo_root = std::env::temp_dir().join(format!("ecaz-cloud-bench-test-{unique}"));
        let config = repo_root.join("reviews/task-84/packet/suite.json");
        fs::create_dir_all(config.parent().expect("config parent")).expect("create config parent");
        fs::write(&config, r#"{"name":"test","schema_version":1,"steps":[]}"#)
            .expect("write config");
        let artifacts = repo_root.join("reviews/task-84/packet/artifacts/run");

        let script = remote_suite_script(
            &repo_root,
            &config,
            &artifacts,
            "postgres",
            "/usr/local/bin/ecaz",
            "s3://bucket/suite/run/",
            Some("s3://bucket/suite/run/suite-config.json"),
            "us-west-2",
            false,
            None,
        )
        .await
        .expect("build remote script");

        assert!(
            script.contains(
                "aws s3 cp 's3://bucket/suite/run/suite-config.json' 'reviews/task-84/packet/suite.json' --region 'us-west-2' --only-show-errors"
            ),
            "{script}"
        );
        assert!(
            script.contains("test -s 'reviews/task-84/packet/suite.json'"),
            "{script}"
        );
        assert!(
            !script.contains("if [ ! -f 'reviews/task-84/packet/suite.json' ]"),
            "{script}"
        );

        fs::remove_dir_all(repo_root).expect("remove temp repo root");
    }
}
