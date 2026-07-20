//! `ecaz dev` — setup, scratch-cluster, and validation helpers.
//!
//! This owns the old wrapper-script surface so operators get one coherent
//! CLI for local installs, scratch clusters, and extension validation.

use clap::Subcommand;
use color_eyre::eyre::Result;

use crate::psql::ConnectionOptions;

mod distann_multicluster;
mod fault;
mod install;
mod pg_upgrade;
mod relation_cache;
mod resource_test;
mod scratch;
mod spire_multicluster;
mod sql;
mod support;
mod test;

#[derive(Subcommand, Debug)]
pub enum DevCommand {
    /// Local install/setup helpers for ecaz, pgvector, and pgvectorscale development.
    Install {
        #[command(subcommand)]
        command: install::InstallCommand,
    },
    /// Scratch-cluster lifecycle and query helpers.
    Scratch {
        #[command(subcommand)]
        command: scratch::ScratchCommand,
    },
    /// PG-level fault-injection planning and smoke probes.
    Fault {
        #[command(subcommand)]
        command: fault::FaultCommand,
    },
    /// SPIRE local multi-cluster fixture helpers.
    SpireMulticluster {
        #[command(subcommand)]
        command: spire_multicluster::SpireMulticlusterCommand,
    },
    /// ec_distann real multi-instance fixture (spin N PG18 instances, replicate
    /// a deterministic corpus, run the multi-node distinct-recall gate).
    DistannMulticluster {
        #[command(subcommand)]
        command: distann_multicluster::DistannMulticlusterCommand,
    },
    /// Run SQL against local pgrx PostgreSQL or a global connection target.
    Sql(sql::SqlArgs),
    /// Run PG18-to-PG18 pg_upgrade with ECAZ data and post-upgrade checks.
    PgUpgradeSmoke(pg_upgrade::PgUpgradeSmokeArgs),
    /// Evict local PostgreSQL relation files from the OS page cache.
    EvictRelationCache(relation_cache::EvictRelationCacheArgs),
    /// Validation/test entry points.
    Test {
        #[command(subcommand)]
        command: test::TestCommand,
    },
    /// Resource-exhaustion sweep over the six Task 48 §Scope scenarios
    /// (max-locks, max-connections, work-mem-min, temp-file-limit,
    /// shared-buffers-thrash, disk-full).
    ResourceTest(resource_test::ResourceTestArgs),
}

impl DevCommand {
    pub async fn run(self, conn: &ConnectionOptions) -> Result<()> {
        match self {
            DevCommand::Install { command } => command.run(&conn.database).await,
            DevCommand::Scratch { command } => command.run(&conn.database).await,
            DevCommand::Fault { command } => command.run(conn).await,
            DevCommand::SpireMulticluster { command } => command.run(&conn.database).await,
            DevCommand::DistannMulticluster { command } => command.run().await,
            DevCommand::Sql(args) => sql::run(conn, args).await,
            DevCommand::PgUpgradeSmoke(args) => pg_upgrade::run(args).await,
            DevCommand::EvictRelationCache(args) => relation_cache::run(conn, args).await,
            DevCommand::Test { command } => command.run(conn).await,
            DevCommand::ResourceTest(args) => resource_test::run(conn, args).await,
        }
    }
}
