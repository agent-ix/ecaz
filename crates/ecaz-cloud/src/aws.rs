//! AWS CLI wrappers used by `pause`, `resume`, `snapshot`, and
//! `status`. We shell out to `aws` rather than depending on the SDK to
//! keep the dependency tree small and to make failures reproducible at
//! the operator's shell prompt.

use color_eyre::eyre::{eyre, Context, Result};
use tokio::process::Command;

pub async fn ensure_credentials() -> Result<()> {
    // `aws sts get-caller-identity` is the cheap canonical credential
    // probe. If this fails, every other call would fail with a noisier
    // error — handle it once, here.
    let output = Command::new("aws")
        .args(["sts", "get-caller-identity"])
        .output()
        .await
        .wrap_err("invoke aws sts get-caller-identity")?;
    if !output.status.success() {
        return Err(eyre!(
            "AWS credentials are missing or invalid. \
             Set AWS_PROFILE or run `aws configure`. \
             stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

pub async fn stop_instances(region: &str, ids: &[&str]) -> Result<()> {
    let mut args = vec![
        "ec2".to_string(),
        "stop-instances".to_string(),
        "--region".to_string(),
        region.to_string(),
        "--instance-ids".to_string(),
    ];
    for id in ids {
        args.push((*id).to_string());
    }
    run_aws(&args).await
}

pub async fn start_instances(region: &str, ids: &[&str]) -> Result<()> {
    let mut args = vec![
        "ec2".to_string(),
        "start-instances".to_string(),
        "--region".to_string(),
        region.to_string(),
        "--instance-ids".to_string(),
    ];
    for id in ids {
        args.push((*id).to_string());
    }
    run_aws(&args).await
}

pub async fn describe_instance_state(region: &str, id: &str) -> Result<String> {
    let output = Command::new("aws")
        .args([
            "ec2",
            "describe-instances",
            "--region",
            region,
            "--instance-ids",
            id,
            "--query",
            "Reservations[0].Instances[0].State.Name",
            "--output",
            "text",
        ])
        .output()
        .await
        .wrap_err("invoke aws ec2 describe-instances")?;
    if !output.status.success() {
        return Err(eyre!(
            "describe-instances {id} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub async fn create_snapshot(region: &str, volume_id: &str, description: &str) -> Result<String> {
    let output = Command::new("aws")
        .args([
            "ec2",
            "create-snapshot",
            "--region",
            region,
            "--volume-id",
            volume_id,
            "--description",
            description,
            "--query",
            "SnapshotId",
            "--output",
            "text",
        ])
        .output()
        .await
        .wrap_err("invoke aws ec2 create-snapshot")?;
    if !output.status.success() {
        return Err(eyre!(
            "create-snapshot {volume_id} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn run_aws(args: &[String]) -> Result<()> {
    let status = Command::new("aws")
        .args(args)
        .status()
        .await
        .wrap_err("invoke aws cli")?;
    if !status.success() {
        return Err(eyre!("aws {:?} failed: {status}", args));
    }
    Ok(())
}
