//! Thin wrapper around the `terraform` CLI.
//!
//! Each call runs in `infra/cloud/terraform/` (resolved relative to the
//! repository root the operator invokes from) with state placed under the
//! per-profile state directory so multiple profiles can coexist.

use color_eyre::eyre::{eyre, Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::process::Command;

use crate::profiles::Profile;
use crate::state;

pub struct Terraform {
    profile: Profile,
    module_dir: PathBuf,
    state_path: PathBuf,
}

impl Terraform {
    pub fn new(profile: Profile, repo_root: &Path) -> Result<Self> {
        let module_dir = repo_root.join("infra/cloud/terraform");
        if !module_dir.exists() {
            return Err(eyre!(
                "terraform module not found at {} \
                 (expected to run from the ecaz repo root)",
                module_dir.display()
            ));
        }
        let state_path = state::terraform_state_path(profile)?;
        state::ensure_dir(state_path.parent().unwrap())?;
        Ok(Self {
            profile,
            module_dir,
            state_path,
        })
    }

    fn base_args(&self) -> Vec<String> {
        // -input=false avoids interactive prompts; -no-color keeps logs
        // clean for packet-local artifact capture.
        vec![
            "-input=false".to_string(),
            "-no-color".to_string(),
            format!("-state={}", self.state_path.display()),
        ]
    }

    fn tfvars_path(&self) -> PathBuf {
        self.module_dir
            .join("profiles")
            .join(format!("{}.tfvars", self.profile.name()))
    }

    pub async fn init(&self) -> Result<()> {
        run_terraform(
            &self.module_dir,
            &["init", "-input=false", "-no-color", "-upgrade=false"],
        )
        .await
    }

    pub async fn plan(&self) -> Result<()> {
        let mut args: Vec<String> = vec!["plan".into()];
        args.extend(self.base_args());
        args.push(format!("-var-file={}", self.tfvars_path().display()));
        run_terraform(
            &self.module_dir,
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        )
        .await
    }

    pub async fn apply(&self, extra_vars: &[(&str, &str)]) -> Result<()> {
        let mut args: Vec<String> = vec!["apply".into(), "-auto-approve".into()];
        args.extend(self.base_args());
        args.push(format!("-var-file={}", self.tfvars_path().display()));
        for (k, v) in extra_vars {
            args.push(format!("-var={}={}", k, v));
        }
        run_terraform(
            &self.module_dir,
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        )
        .await
    }

    pub async fn destroy(&self) -> Result<()> {
        let mut args: Vec<String> = vec!["destroy".into(), "-auto-approve".into()];
        args.extend(self.base_args());
        args.push(format!("-var-file={}", self.tfvars_path().display()));
        run_terraform(
            &self.module_dir,
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        )
        .await
    }

    pub async fn outputs(&self) -> Result<Outputs> {
        let output = Command::new("terraform")
            .arg("output")
            .arg("-json")
            .arg(format!("-state={}", self.state_path.display()))
            .current_dir(&self.module_dir)
            .output()
            .await
            .wrap_err("invoke terraform output")?;
        if !output.status.success() {
            return Err(eyre!(
                "terraform output failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let raw: serde_json::Value = serde_json::from_slice(&output.stdout)
            .wrap_err("parse terraform output JSON")?;
        Outputs::from_json(&raw)
    }

    pub fn state_exists(&self) -> bool {
        self.state_path.exists()
    }
}

async fn run_terraform(cwd: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("terraform")
        .args(args)
        .current_dir(cwd)
        .status()
        .await
        .wrap_err("invoke terraform")?;
    if !status.success() {
        return Err(eyre!("terraform {:?} failed: {status}", args));
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize)]
pub struct Outputs {
    pub region: String,
    pub vpc_id: String,
    pub subnet_id: String,
    pub db_instance_id: String,
    pub db_private_ip: String,
    pub db_volume_id: String,
    pub loader_instance_id: String,
    pub s3_bucket: String,
}

impl Outputs {
    fn from_json(v: &serde_json::Value) -> Result<Self> {
        // `terraform output -json` emits {"key": {"value": ..., "type": ...}}.
        let pick = |k: &str| -> Result<String> {
            v.get(k)
                .and_then(|o| o.get("value"))
                .and_then(|s| s.as_str().map(str::to_owned))
                .ok_or_else(|| eyre!("terraform output missing key: {k}"))
        };
        Ok(Self {
            region: pick("region")?,
            vpc_id: pick("vpc_id")?,
            subnet_id: pick("subnet_id")?,
            db_instance_id: pick("db_instance_id")?,
            db_private_ip: pick("db_private_ip")?,
            db_volume_id: pick("db_volume_id")?,
            loader_instance_id: pick("loader_instance_id")?,
            s3_bucket: pick("s3_bucket")?,
        })
    }
}
