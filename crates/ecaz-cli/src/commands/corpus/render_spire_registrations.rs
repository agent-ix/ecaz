use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Args, Debug)]
pub struct RenderSpireRegistrationsArgs {
    /// Path to distributed-placement-plan.json from `ecaz corpus load`.
    #[arg(long)]
    pub plan_file: PathBuf,

    /// Directory containing one `node-{node_id}-identity.json` file per remote.
    ///
    /// Each file is the JSON text returned by the plan's
    /// `remote_identity_query_sql` against that remote database.
    #[arg(long)]
    pub identity_dir: PathBuf,

    /// Coordinator SQL file to write.
    #[arg(long)]
    pub output_file: PathBuf,

    /// Descriptor generation to register on the coordinator.
    #[arg(long, default_value_t = 1)]
    pub descriptor_generation: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DistributedPlacementPlan {
    coordinator_index_name: String,
    remotes: Vec<DistributedPlacementRemote>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DistributedPlacementRemote {
    node_id: u32,
    conninfo_secret_name: String,
    remote_index_regclass: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RemoteEndpointIdentity {
    remote_index_regclass: Option<String>,
    remote_index_identity_hex: String,
    last_served_epoch: i64,
    min_retained_epoch: i64,
    extension_version: String,
    endpoint_status: String,
    tuple_transport_status: String,
}

pub async fn run(args: RenderSpireRegistrationsArgs) -> Result<()> {
    let sql = render_registration_sql_from_files(
        &args.plan_file,
        &args.identity_dir,
        args.descriptor_generation,
    )?;
    if let Some(parent) = args
        .output_file
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .wrap_err_with(|| format!("creating {}", parent.display()))?;
    }
    std::fs::write(&args.output_file, sql)
        .wrap_err_with(|| format!("writing {}", args.output_file.display()))?;
    crate::ecaz_println!(
        "[corpus] wrote SPIRE descriptor registration SQL to {}",
        args.output_file.display()
    );
    Ok(())
}

fn render_registration_sql_from_files(
    plan_file: &Path,
    identity_dir: &Path,
    descriptor_generation: i64,
) -> Result<String> {
    if descriptor_generation < 0 {
        return Err(eyre!("--descriptor-generation must be non-negative"));
    }
    let raw_plan = std::fs::read_to_string(plan_file)
        .wrap_err_with(|| format!("reading {}", plan_file.display()))?;
    let plan: DistributedPlacementPlan = serde_json::from_str(&raw_plan)
        .wrap_err_with(|| format!("parsing {}", plan_file.display()))?;
    validate_plan(&plan).wrap_err_with(|| format!("validating {}", plan_file.display()))?;

    let mut sql = String::new();
    sql.push_str("\\set ON_ERROR_STOP on\n\n");
    for remote in &plan.remotes {
        let identity_path = identity_dir.join(format!("node-{}-identity.json", remote.node_id));
        let identity = read_remote_identity(&identity_path, remote)?;
        sql.push_str(&render_one_registration_sql(
            &plan.coordinator_index_name,
            remote,
            &identity,
            descriptor_generation,
        )?);
        sql.push_str("\n\n");
    }
    Ok(sql)
}

fn validate_plan(plan: &DistributedPlacementPlan) -> Result<()> {
    if plan.coordinator_index_name.is_empty() {
        return Err(eyre!("coordinator_index_name must not be empty"));
    }
    if plan.remotes.is_empty() {
        return Err(eyre!("distributed placement plan has no remotes"));
    }
    let mut seen_node_ids = std::collections::BTreeSet::new();
    for remote in &plan.remotes {
        if remote.node_id == 0 {
            return Err(eyre!("remote node_id must be greater than zero"));
        }
        if !seen_node_ids.insert(remote.node_id) {
            return Err(eyre!("duplicate remote node_id {}", remote.node_id));
        }
        if remote.conninfo_secret_name.is_empty() {
            return Err(eyre!("conninfo_secret_name must not be empty"));
        }
        if remote.remote_index_regclass.is_empty() {
            return Err(eyre!("remote_index_regclass must not be empty"));
        }
    }
    Ok(())
}

fn read_remote_identity(
    path: &Path,
    remote: &DistributedPlacementRemote,
) -> Result<RemoteEndpointIdentity> {
    let raw =
        std::fs::read_to_string(path).wrap_err_with(|| format!("reading {}", path.display()))?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(eyre!("remote identity file {} is empty", path.display()));
    }
    let identity: RemoteEndpointIdentity =
        serde_json::from_str(trimmed).wrap_err_with(|| format!("parsing {}", path.display()))?;
    validate_remote_identity(&identity, remote)
        .wrap_err_with(|| format!("validating {}", path.display()))?;
    Ok(identity)
}

fn validate_remote_identity(
    identity: &RemoteEndpointIdentity,
    remote: &DistributedPlacementRemote,
) -> Result<()> {
    if let Some(identity_regclass) = identity.remote_index_regclass.as_deref() {
        if identity_regclass != remote.remote_index_regclass {
            return Err(eyre!(
                "remote_index_regclass {:?} does not match plan {:?}",
                identity_regclass,
                remote.remote_index_regclass
            ));
        }
    }
    if identity.endpoint_status != "ready" {
        return Err(eyre!(
            "endpoint_status {:?} is not ready",
            identity.endpoint_status
        ));
    }
    if identity.tuple_transport_status != "ready" {
        return Err(eyre!(
            "tuple_transport_status {:?} is not ready",
            identity.tuple_transport_status
        ));
    }
    if identity.remote_index_identity_hex.is_empty() {
        return Err(eyre!("remote_index_identity_hex must not be empty"));
    }
    if identity.remote_index_identity_hex.len() % 2 != 0 {
        return Err(eyre!("remote_index_identity_hex must have even length"));
    }
    hex::decode(&identity.remote_index_identity_hex)
        .wrap_err("remote_index_identity_hex must be valid hex")?;
    if identity.last_served_epoch <= 0 {
        return Err(eyre!("last_served_epoch must be greater than zero"));
    }
    if identity.min_retained_epoch <= 0 {
        return Err(eyre!("min_retained_epoch must be greater than zero"));
    }
    if identity.extension_version.is_empty() {
        return Err(eyre!("extension_version must not be empty"));
    }
    Ok(())
}

fn render_one_registration_sql(
    coordinator_index_name: &str,
    remote: &DistributedPlacementRemote,
    identity: &RemoteEndpointIdentity,
    descriptor_generation: i64,
) -> Result<String> {
    Ok(format!(
        "SELECT ec_spire_register_remote_node_descriptor(\
            {coordinator_index}::regclass::oid, \
            {node_id}, \
            {descriptor_generation}, \
            {conninfo_secret}, \
            decode({remote_index_identity_hex}, 'hex'), \
            {remote_index}, \
            'active', \
            {last_served_epoch}, \
            {min_retained_epoch}, \
            {extension_version}, \
            'none'\
        ) AS registered_node_{node_id};",
        coordinator_index = sql_string_literal(coordinator_index_name),
        node_id = remote.node_id,
        conninfo_secret = sql_string_literal(&remote.conninfo_secret_name),
        remote_index_identity_hex = sql_string_literal(&identity.remote_index_identity_hex),
        remote_index = sql_string_literal(&remote.remote_index_regclass),
        last_served_epoch = identity.last_served_epoch,
        min_retained_epoch = identity.min_retained_epoch,
        extension_version = sql_string_literal(&identity.extension_version),
    ))
}

fn sql_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, body).unwrap();
    }

    fn plan_json() -> String {
        serde_json::json!({
            "coordinator_index_name": "public.coord_idx",
            "remotes": [{
                "node_id": 2,
                "conninfo_secret_name": "spire/remote/a",
                "remote_index_regclass": "public.remote_idx"
            }]
        })
        .to_string()
    }

    fn identity_json() -> String {
        serde_json::json!({
            "remote_index_regclass": "public.remote_idx",
            "remote_index_identity_hex": "aabbccdd",
            "last_served_epoch": 7,
            "min_retained_epoch": 5,
            "extension_version": "0.1.2",
            "endpoint_status": "ready",
            "tuple_transport_status": "ready"
        })
        .to_string()
    }

    #[test]
    fn parses_identity_shape_emitted_by_distributed_remote_identity_query() {
        let identity: RemoteEndpointIdentity = serde_json::from_str(&identity_json()).unwrap();

        assert_eq!(
            identity.remote_index_regclass.as_deref(),
            Some("public.remote_idx")
        );
        assert_eq!(identity.remote_index_identity_hex, "aabbccdd");
        assert_eq!(identity.last_served_epoch, 7);
        assert_eq!(identity.min_retained_epoch, 5);
    }

    #[test]
    fn rejects_unexpected_identity_fields() {
        let raw = serde_json::json!({
            "remote_index_regclass": "public.remote_idx",
            "remote_index_identity_hex": "aabbccdd",
            "active_epoch": 7,
            "last_served_epoch": 7,
            "min_retained_epoch": 5,
            "extension_version": "0.1.2",
            "endpoint_status": "ready",
            "tuple_transport_status": "ready"
        })
        .to_string();

        let err = serde_json::from_str::<RemoteEndpointIdentity>(&raw)
            .unwrap_err()
            .to_string();
        assert!(err.contains("active_epoch"), "err: {err}");
    }

    #[test]
    fn renders_descriptor_registration_sql_from_plan_and_identity() {
        let td = TempDir::new().unwrap();
        let plan_path = td.path().join("distributed-placement-plan.json");
        let identity_dir = td.path().join("identities");
        write(&plan_path, &plan_json());
        write(&identity_dir.join("node-2-identity.json"), &identity_json());

        let sql = render_registration_sql_from_files(&plan_path, &identity_dir, 11).unwrap();

        assert!(sql.contains("\\set ON_ERROR_STOP on"));
        assert!(sql.contains("ec_spire_register_remote_node_descriptor"));
        assert!(sql.contains("'public.coord_idx'::regclass::oid"));
        assert!(sql.contains("'spire/remote/a'"));
        assert!(sql.contains("decode('aabbccdd', 'hex')"));
        assert!(sql.contains("'public.remote_idx'"));
        assert!(sql.contains("11"));
        assert!(sql.contains("7"));
        assert!(sql.contains("5"));
        assert!(sql.contains("'0.1.2'"));
    }

    #[test]
    fn rejects_identity_that_is_not_ready() {
        let mut identity: RemoteEndpointIdentity = serde_json::from_str(&identity_json()).unwrap();
        identity.endpoint_status = "requires_rabitq_storage_format".to_owned();
        let remote = DistributedPlacementRemote {
            node_id: 2,
            conninfo_secret_name: "spire/remote/a".to_owned(),
            remote_index_regclass: "public.remote_idx".to_owned(),
        };

        let err = validate_remote_identity(&identity, &remote)
            .unwrap_err()
            .to_string();
        assert!(err.contains("endpoint_status"), "err: {err}");
    }

    #[test]
    fn rejects_identity_for_wrong_remote_index() {
        let mut identity: RemoteEndpointIdentity = serde_json::from_str(&identity_json()).unwrap();
        identity.remote_index_regclass = Some("public.other_idx".to_owned());
        let remote = DistributedPlacementRemote {
            node_id: 2,
            conninfo_secret_name: "spire/remote/a".to_owned(),
            remote_index_regclass: "public.remote_idx".to_owned(),
        };

        let err = validate_remote_identity(&identity, &remote)
            .unwrap_err()
            .to_string();
        assert!(err.contains("does not match plan"), "err: {err}");
    }

    #[test]
    fn rejects_non_hex_identity() {
        let mut identity: RemoteEndpointIdentity = serde_json::from_str(&identity_json()).unwrap();
        identity.remote_index_identity_hex = "zz".to_owned();
        let remote = DistributedPlacementRemote {
            node_id: 2,
            conninfo_secret_name: "spire/remote/a".to_owned(),
            remote_index_regclass: "public.remote_idx".to_owned(),
        };

        let err = validate_remote_identity(&identity, &remote)
            .unwrap_err()
            .to_string();
        assert!(err.contains("valid hex"), "err: {err}");
    }

    #[test]
    fn sql_literals_escape_quotes() {
        assert_eq!(sql_string_literal("a'b"), "'a''b'");
    }
}
