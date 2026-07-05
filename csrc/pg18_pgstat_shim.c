#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "postgres.h"

#include "miscadmin.h"
#include "pgstat.h"
#include "utils/pgstat_internal.h"
#include "utils/pgstat_kind.h"

typedef struct TqVectorPgStatCounters
{
	uint64_t	total_distance_calcs;
	uint64_t	total_graph_hops;
	uint64_t	total_linear_pages;
	uint64_t	total_scans_started;
	uint64_t	total_scans_bootstrap_only;
	uint64_t	quantizer_cache_hits;
	uint64_t	quantizer_cache_misses;
} TqVectorPgStatCounters;

typedef struct TqVectorPgStatShared
{
	LWLock		lock;
	uint32		changecount;
	TqVectorPgStatCounters stats;
	TqVectorPgStatCounters reset_offset;
} TqVectorPgStatShared;

/*
 * Claim a stable ecaz-specific custom kind instead of the shared
 * PGSTAT_KIND_EXPERIMENTAL slot so preload-time registration does not collide
 * with other extensions using the experimental ID.
 */
#define ECAZ_PGSTAT_KIND	((PgStat_Kind) (PGSTAT_KIND_CUSTOM_MIN + 1))

static void ecaz_pgstat_init_shmem(void *stats);
static void ecaz_pgstat_reset_all(TimestampTz ts);
static void ecaz_pgstat_snapshot(void);

static const PgStat_KindInfo ecaz_pgstat_kind = {
	.name = "ecaz",
	.fixed_amount = true,
	.accessed_across_databases = true,
	.write_to_file = true,
	.shared_size = sizeof(TqVectorPgStatShared),
	.shared_data_off = offsetof(TqVectorPgStatShared, stats),
	.shared_data_len = sizeof(((TqVectorPgStatShared *) 0)->stats),
	.init_shmem_cb = ecaz_pgstat_init_shmem,
	.reset_all_cb = ecaz_pgstat_reset_all,
	.snapshot_cb = ecaz_pgstat_snapshot,
};

static bool ecaz_pgstat_loaded = false;

void
ecaz_pg18_pgstat_anchor(void)
{
}

static TqVectorPgStatShared *
ecaz_pgstat_shared(void)
{
	return (TqVectorPgStatShared *)
		pgstat_get_custom_shmem_data(ECAZ_PGSTAT_KIND);
}

static void
ecaz_pgstat_init_shmem(void *stats)
{
	TqVectorPgStatShared *stats_shmem = (TqVectorPgStatShared *) stats;

	LWLockInitialize(&stats_shmem->lock, LWTRANCHE_PGSTATS_DATA);
}

static void
ecaz_pgstat_reset_all(TimestampTz ts)
{
	TqVectorPgStatShared *stats_shmem = ecaz_pgstat_shared();
	(void) ts;

	LWLockAcquire(&stats_shmem->lock, LW_EXCLUSIVE);
	/*
	 * Readers already use the changecount protocol for the live stats snapshot.
	 * Keep the reset offset behind the same helper while we hold the lock so the
	 * reset baseline and the live counters advance in a single ordered step.
	 */
	pgstat_copy_changecounted_stats(&stats_shmem->reset_offset,
									&stats_shmem->stats,
									sizeof(stats_shmem->stats),
									&stats_shmem->changecount);
	LWLockRelease(&stats_shmem->lock);
}

static void
ecaz_pgstat_snapshot(void)
{
	TqVectorPgStatShared *stats_shmem = ecaz_pgstat_shared();
	TqVectorPgStatCounters *stat_snap =
		(TqVectorPgStatCounters *)
		pgstat_get_custom_snapshot_data(ECAZ_PGSTAT_KIND);
	TqVectorPgStatCounters reset;

	pgstat_copy_changecounted_stats(stat_snap,
									&stats_shmem->stats,
									sizeof(stats_shmem->stats),
									&stats_shmem->changecount);

	LWLockAcquire(&stats_shmem->lock, LW_SHARED);
	memcpy(&reset, &stats_shmem->reset_offset, sizeof(reset));
	LWLockRelease(&stats_shmem->lock);

#define TQVECTOR_FIXED_COMP(fld) stat_snap->fld -= reset.fld;
	TQVECTOR_FIXED_COMP(total_distance_calcs);
	TQVECTOR_FIXED_COMP(total_graph_hops);
	TQVECTOR_FIXED_COMP(total_linear_pages);
	TQVECTOR_FIXED_COMP(total_scans_started);
	TQVECTOR_FIXED_COMP(total_scans_bootstrap_only);
	TQVECTOR_FIXED_COMP(quantizer_cache_hits);
	TQVECTOR_FIXED_COMP(quantizer_cache_misses);
#undef TQVECTOR_FIXED_COMP
}

bool
ecaz_pg18_pgstat_register_kind(void)
{
	if (ecaz_pgstat_loaded)
		return true;

	if (!process_shared_preload_libraries_in_progress)
		return false;

	pgstat_register_kind(ECAZ_PGSTAT_KIND, &ecaz_pgstat_kind);
	ecaz_pgstat_loaded = true;
	return true;
}

bool
ecaz_pg18_pgstat_is_registered(void)
{
	return ecaz_pgstat_loaded;
}

bool
ecaz_pg18_pgstat_record(const TqVectorPgStatCounters *delta)
{
	TqVectorPgStatShared *stats_shmem;

	if (!ecaz_pgstat_loaded || delta == NULL)
		return false;

	stats_shmem = ecaz_pgstat_shared();

	LWLockAcquire(&stats_shmem->lock, LW_EXCLUSIVE);
	pgstat_begin_changecount_write(&stats_shmem->changecount);
	stats_shmem->stats.total_distance_calcs += delta->total_distance_calcs;
	stats_shmem->stats.total_graph_hops += delta->total_graph_hops;
	stats_shmem->stats.total_linear_pages += delta->total_linear_pages;
	stats_shmem->stats.total_scans_started += delta->total_scans_started;
	stats_shmem->stats.total_scans_bootstrap_only += delta->total_scans_bootstrap_only;
	stats_shmem->stats.quantizer_cache_hits += delta->quantizer_cache_hits;
	stats_shmem->stats.quantizer_cache_misses += delta->quantizer_cache_misses;
	pgstat_end_changecount_write(&stats_shmem->changecount);
	LWLockRelease(&stats_shmem->lock);

	return true;
}

bool
ecaz_pg18_pgstat_snapshot(TqVectorPgStatCounters *out)
{
	TqVectorPgStatCounters *snapshot;

	if (!ecaz_pgstat_loaded || out == NULL)
		return false;

	pgstat_snapshot_fixed(ECAZ_PGSTAT_KIND);
	snapshot = (TqVectorPgStatCounters *)
		pgstat_get_custom_snapshot_data(ECAZ_PGSTAT_KIND);
	memcpy(out, snapshot, sizeof(*out));

	return true;
}
