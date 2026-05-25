use clap::Args;
use color_eyre::eyre::{eyre, Result};
use std::path::PathBuf;

use crate::{aws, profiles::Profile, ssm, terraform::Terraform};

#[derive(Args, Debug)]
pub struct CleanupScratchArgs {
    #[arg(long)]
    pub profile: Profile,

    /// Task-scoped dataset scratch directory to remove.
    ///
    /// Safety guard: the path must live directly under
    /// `/var/lib/pgsql/18/datasets/` and its basename must start with
    /// `staged-task`.
    #[arg(long)]
    pub path: String,

    /// Restart PostgreSQL after cleanup.
    #[arg(long)]
    pub restart_postgres: bool,
}

impl CleanupScratchArgs {
    pub async fn run(self, repo_root: PathBuf) -> Result<()> {
        aws::ensure_credentials().await?;
        validate_cleanup_path(&self.path)?;
        let tf = Terraform::new(self.profile, &repo_root)?;
        if !tf.state_exists() {
            return Err(eyre!("no stack for profile {}", self.profile));
        }
        let out = tf.outputs().await?;

        let restart = if self.restart_postgres {
            "sudo systemctl restart postgresql\nsudo systemctl is-active postgresql"
        } else {
            ":"
        };
        let script = format!(
            r#"#!/usr/bin/env bash
set -euxo pipefail
target={target}
sudo du -sh "$target" || true
sudo rm -rf "$target"
sudo mkdir -p "$target"
sudo chown postgres:postgres "$target"
df -h /var/lib/pgsql /var/lib/pgsql/18/datasets || true
{restart}
"#,
            target = shell_escape(&self.path),
            restart = restart
        );
        let stdout = ssm::run_shell(&out.region, &out.db_instance_id, &script, 600).await?;
        print!("{stdout}");
        Ok(())
    }
}

fn validate_cleanup_path(path: &str) -> Result<()> {
    let prefix = "/var/lib/pgsql/18/datasets/";
    if !path.starts_with(prefix) {
        return Err(eyre!("cleanup path must be under {prefix}"));
    }
    let name = &path[prefix.len()..];
    if name.contains('/') || !name.starts_with("staged-task") {
        return Err(eyre!(
            "cleanup path basename must start with staged-task and contain no slash"
        ));
    }
    Ok(())
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::validate_cleanup_path;

    #[test]
    fn accepts_task_staging_path() {
        validate_cleanup_path("/var/lib/pgsql/18/datasets/staged-task59-diskann-final").unwrap();
    }

    #[test]
    fn rejects_non_task_path() {
        assert!(validate_cleanup_path("/var/lib/pgsql/18/datasets/staged-1m").is_err());
        assert!(validate_cleanup_path("/var/lib/pgsql/18/datasets/staged-task59/x").is_err());
        assert!(validate_cleanup_path("/tmp/staged-task59").is_err());
    }
}
