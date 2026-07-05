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

/// Look up the most recent `completed`/`pending` EBS snapshot of
/// `volume_id`. Returns `Ok(Some((snapshot_id, start_time_iso)))`
/// if found, `Ok(None)` if the volume has no snapshots at all.
///
/// Used by `ecaz cloud down` to refuse destruction when the data
/// volume has no snapshot covering its current state. Per the
/// snapshot-before-destroy invariant (see
/// `docs/aws-bench-workflow.md`), losing the volume without a
/// snapshot loses hours of corpus + index build work.
pub async fn latest_snapshot_for_volume(
    region: &str,
    volume_id: &str,
) -> Result<Option<(String, String)>> {
    let output = Command::new("aws")
        .args([
            "ec2",
            "describe-snapshots",
            "--region",
            region,
            "--owner-ids",
            "self",
            "--filters",
            &format!("Name=volume-id,Values={}", volume_id),
            "Name=status,Values=completed,pending",
            "--query",
            "sort_by(Snapshots, &StartTime) | [-1].[SnapshotId, StartTime]",
            "--output",
            "text",
        ])
        .output()
        .await
        .wrap_err("invoke aws ec2 describe-snapshots")?;
    if !output.status.success() {
        return Err(eyre!(
            "describe-snapshots for {volume_id} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    if trimmed.is_empty() || trimmed == "None" {
        return Ok(None);
    }
    let mut parts = trimmed.split_whitespace();
    let id = parts
        .next()
        .ok_or_else(|| eyre!("describe-snapshots returned empty row for {volume_id}"))?;
    let started = parts.next().unwrap_or("").to_string();
    Ok(Some((id.to_string(), started)))
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
