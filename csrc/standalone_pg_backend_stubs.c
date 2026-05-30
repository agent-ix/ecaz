#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct MemoryContextData {
    uintptr_t opaque;
    struct MemoryContextData *parent;
} MemoryContextData;

typedef MemoryContextData *MemoryContext;

typedef struct ErrorContextCallback {
    void (*callback)(void *);
    void *arg;
    struct ErrorContextCallback *next;
} ErrorContextCallback;

typedef struct ErrorData {
    int elevel;
    bool output_to_server;
    bool output_to_client;
    bool hide_stmt;
    bool hide_ctx;
    const char *filename;
    int lineno;
    const char *funcname;
    const char *domain;
    const char *context_domain;
    int sqlerrcode;
    char *message;
    char *detail;
    char *detail_log;
    char *hint;
    char *context;
    char *backtrace;
    const char *message_id;
    char *schema_name;
    char *table_name;
    char *column_name;
    char *datatype_name;
    char *constraint_name;
    int cursorpos;
    int internalpos;
    char *internalquery;
    int saved_errno;
    MemoryContext assoc_context;
} ErrorData;

extern void ecaz_test_pg_backend_panic(const char *message);

/*
 * Standalone cargo-test loader stubs have a strict contract:
 *
 * - inert helper symbols may return minimal defaults when they only let
 *   pure-Rust tests load pgrx-linked code;
 * - backend execution symbols must panic through tqvector_backend_only() so a
 *   direct cargo test cannot fake SPI, heap, catalog, or executor behavior.
 *
 * If a new unresolved PostgreSQL symbol appears, classify it deliberately in
 * one of those two groups. Anything that would read or write backend state
 * belongs in the pgrx/pg_test lane, not in a fake standalone implementation.
 */
static const uintptr_t TQVECTOR_TEST_ALLOCATED_MEMORY_CONTEXT = 0xecaa0001U;

static MemoryContextData tqvector_top_memory_context_storage = {0};
static MemoryContextData tqvector_error_context_storage = {0};
static MemoryContextData tqvector_cache_memory_context_storage = {0};
static MemoryContextData tqvector_message_context_storage = {0};
static MemoryContextData tqvector_top_transaction_context_storage = {0};
static MemoryContextData tqvector_cur_transaction_context_storage = {0};
static MemoryContextData tqvector_portal_context_storage = {0};
static MemoryContextData tqvector_postmaster_context_storage = {0};

MemoryContext TopMemoryContext = &tqvector_top_memory_context_storage;
MemoryContext CurrentMemoryContext = &tqvector_top_memory_context_storage;
MemoryContext ErrorContext = &tqvector_error_context_storage;
MemoryContext CacheMemoryContext = &tqvector_cache_memory_context_storage;
MemoryContext MessageContext = &tqvector_message_context_storage;
MemoryContext TopTransactionContext = &tqvector_top_transaction_context_storage;
MemoryContext CurTransactionContext = &tqvector_cur_transaction_context_storage;
MemoryContext PortalContext = &tqvector_portal_context_storage;
MemoryContext PostmasterContext = &tqvector_postmaster_context_storage;
ErrorContextCallback *error_context_stack = NULL;
void *PG_exception_stack = NULL;

char *BufferBlocks = NULL;
uint32_t CheckXidAlive = 0;
bool bsysscan = false;
void **LocalBufferBlockPointers = NULL;
uint64_t SPI_processed = 0;
void *SPI_tuptable = NULL;
int NBuffers = 0;
int NLocBuffer = 0;

static __thread ErrorData tqvector_current_error = {0};
static __thread bool tqvector_current_error_active = false;

static char *tqvector_strdup(const char *value) {
    if (value == NULL) {
        return NULL;
    }

    size_t len = strlen(value) + 1;
    char *copy = (char *)malloc(len);
    if (copy == NULL) {
        abort();
    }

    memcpy(copy, value, len);
    return copy;
}

static char *tqvector_vformat(const char *fmt, va_list args) {
    if (fmt == NULL) {
        return NULL;
    }

    va_list probe;
    va_copy(probe, args);
    int needed = vsnprintf(NULL, 0, fmt, probe);
    va_end(probe);

    if (needed < 0) {
        return tqvector_strdup(fmt);
    }

    char *buffer = (char *)malloc((size_t)needed + 1);
    if (buffer == NULL) {
        abort();
    }

    vsnprintf(buffer, (size_t)needed + 1, fmt, args);
    return buffer;
}

static void tqvector_free_error(ErrorData *edata) {
    free((char *)edata->filename);
    free((char *)edata->funcname);
    free((char *)edata->domain);
    free((char *)edata->context_domain);
    free(edata->message);
    free(edata->detail);
    free(edata->detail_log);
    free(edata->hint);
    free(edata->context);
    free(edata->backtrace);
    free(edata->schema_name);
    free(edata->table_name);
    free(edata->column_name);
    free(edata->datatype_name);
    free(edata->constraint_name);
    free(edata->internalquery);

    memset(edata, 0, sizeof(*edata));
}

static void tqvector_set_text(char **slot, const char *fmt, va_list args) {
    free(*slot);
    *slot = tqvector_vformat(fmt, args);
}

static void tqvector_backend_only(const char *symbol) {
    ecaz_test_pg_backend_panic(symbol);
    abort();
}

void ecaz_test_pg_backend_stubs_anchor(void) {}

int errstart(int elevel, const char *domain) {
    tqvector_free_error(&tqvector_current_error);
    tqvector_current_error_active = true;
    tqvector_current_error.elevel = elevel;
    tqvector_current_error.output_to_server = true;
    tqvector_current_error.domain = tqvector_strdup(domain);
    tqvector_current_error.context_domain = tqvector_strdup(domain);
    tqvector_current_error.assoc_context = ErrorContext;
    return 1;
}

int errstart_cold(int elevel, const char *domain) {
    return errstart(elevel, domain);
}

int errcode(int sqlerrcode) {
    tqvector_current_error.sqlerrcode = sqlerrcode;
    return 0;
}

int errmsg(const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);
    tqvector_set_text(&tqvector_current_error.message, fmt, args);
    va_end(args);
    return 0;
}

int errmsg_internal(const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);
    tqvector_set_text(&tqvector_current_error.message, fmt, args);
    va_end(args);
    return 0;
}

int errdetail(const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);
    tqvector_set_text(&tqvector_current_error.detail, fmt, args);
    va_end(args);
    return 0;
}

int errhint(const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);
    tqvector_set_text(&tqvector_current_error.hint, fmt, args);
    va_end(args);
    return 0;
}

int errcontext_msg(const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);
    tqvector_set_text(&tqvector_current_error.context, fmt, args);
    va_end(args);
    return 0;
}

void errfinish(const char *filename, int lineno, const char *funcname) {
    free((char *)tqvector_current_error.filename);
    free((char *)tqvector_current_error.funcname);
    tqvector_current_error.filename = tqvector_strdup(filename);
    tqvector_current_error.lineno = lineno;
    tqvector_current_error.funcname = tqvector_strdup(funcname);

    if (tqvector_current_error.elevel >= 21) {
        const char *message = tqvector_current_error.message;
        ecaz_test_pg_backend_panic(message != NULL ? message : "Postgres ERROR");
        abort();
    }
}

void pg_re_throw(void) {
    const char *message = tqvector_current_error.message;
    ecaz_test_pg_backend_panic(message != NULL ? message : "Postgres ERROR");
    abort();
}

ErrorData *CopyErrorData(void) {
    ErrorData *copy = (ErrorData *)calloc(1, sizeof(ErrorData));
    if (copy == NULL) {
        abort();
    }

    *copy = tqvector_current_error;
    copy->filename = tqvector_strdup(tqvector_current_error.filename);
    copy->funcname = tqvector_strdup(tqvector_current_error.funcname);
    copy->domain = tqvector_strdup(tqvector_current_error.domain);
    copy->context_domain = tqvector_strdup(tqvector_current_error.context_domain);
    copy->message = tqvector_strdup(tqvector_current_error.message);
    copy->detail = tqvector_strdup(tqvector_current_error.detail);
    copy->detail_log = tqvector_strdup(tqvector_current_error.detail_log);
    copy->hint = tqvector_strdup(tqvector_current_error.hint);
    copy->context = tqvector_strdup(tqvector_current_error.context);
    copy->backtrace = tqvector_strdup(tqvector_current_error.backtrace);
    copy->schema_name = tqvector_strdup(tqvector_current_error.schema_name);
    copy->table_name = tqvector_strdup(tqvector_current_error.table_name);
    copy->column_name = tqvector_strdup(tqvector_current_error.column_name);
    copy->datatype_name = tqvector_strdup(tqvector_current_error.datatype_name);
    copy->constraint_name = tqvector_strdup(tqvector_current_error.constraint_name);
    copy->internalquery = tqvector_strdup(tqvector_current_error.internalquery);
    return copy;
}

void FreeErrorData(ErrorData *edata) {
    if (edata == NULL) {
        return;
    }

    tqvector_free_error(edata);
    free(edata);
}

void *palloc0(size_t size) {
    if (size == 0) {
        size = 1;
    }

    void *allocation = calloc(1, size);
    if (allocation == NULL) {
        abort();
    }
    return allocation;
}

void *palloc(size_t size) {
    if (size == 0) {
        size = 1;
    }

    void *allocation = malloc(size);
    if (allocation == NULL) {
        abort();
    }
    return allocation;
}

void pfree(void *pointer) {
    free(pointer);
}

MemoryContext AllocSetContextCreateInternal(
    MemoryContext parent,
    const char *name,
    size_t minContextSize,
    size_t initBlockSize,
    size_t maxBlockSize
) {
    (void)name;
    (void)minContextSize;
    (void)initBlockSize;
    (void)maxBlockSize;

    MemoryContext context = (MemoryContext)calloc(1, sizeof(MemoryContextData));
    if (context == NULL) {
        abort();
    }

    context->opaque = TQVECTOR_TEST_ALLOCATED_MEMORY_CONTEXT;
    context->parent = parent != NULL ? parent : TopMemoryContext;
    return context;
}

void MemoryContextDelete(MemoryContext context) {
    if (context != NULL && context->opaque == TQVECTOR_TEST_ALLOCATED_MEMORY_CONTEXT) {
        free(context);
    }
}

MemoryContext MemoryContextGetParent(MemoryContext context) {
    if (context == NULL || context->parent == NULL) {
        return TopMemoryContext;
    }

    return context->parent;
}

uint32_t GetCurrentTransactionId(void) {
    return 1;
}

uint32_t GetCurrentTransactionIdIfAny(void) {
    return 1;
}

bool IsBinaryCoercible(uint32_t srctype, uint32_t targettype) {
    return srctype == targettype;
}

int SPI_connect(void) {
    tqvector_backend_only("SPI_connect is unavailable outside a PostgreSQL backend");
    return -1;
}

int SPI_finish(void) {
    tqvector_backend_only("SPI_finish is unavailable outside a PostgreSQL backend");
    return -1;
}

int SPI_execute(const char *src, bool read_only, long tcount) {
    (void)src;
    (void)read_only;
    (void)tcount;
    tqvector_backend_only("SPI_execute is unavailable outside a PostgreSQL backend");
    return -1;
}

int SPI_execute_with_args(
    const char *src,
    int nargs,
    uint32_t *argtypes,
    uintptr_t *values,
    const char *nulls,
    bool read_only,
    long tcount
) {
    (void)src;
    (void)nargs;
    (void)argtypes;
    (void)values;
    (void)nulls;
    (void)read_only;
    (void)tcount;
    tqvector_backend_only("SPI_execute_with_args is unavailable outside a PostgreSQL backend");
    return -1;
}

uintptr_t SPI_getbinval(void *tuple, void *tupdesc, int fnumber, bool *isnull) {
    (void)tuple;
    (void)tupdesc;
    (void)fnumber;
    if (isnull != NULL) {
        *isnull = true;
    }
    tqvector_backend_only("SPI_getbinval is unavailable outside a PostgreSQL backend");
    return 0;
}

uint32_t SPI_gettypeid(void *tupdesc, int fnumber) {
    (void)tupdesc;
    (void)fnumber;
    tqvector_backend_only("SPI_gettypeid is unavailable outside a PostgreSQL backend");
    return 0;
}

char *format_type_extended(uint32_t type_oid, int32_t typemod, uint16_t flags) {
    (void)type_oid;
    (void)typemod;
    (void)flags;
    return tqvector_strdup("unknown");
}

char *format_type_be(uint32_t type_oid) {
    (void)type_oid;
    return tqvector_strdup("unknown");
}

static void *tqvector_backend_only_ptr(const char *symbol) {
    tqvector_backend_only(symbol);
    return NULL;
}

static uintptr_t tqvector_backend_only_uintptr(const char *symbol) {
    tqvector_backend_only(symbol);
    return 0;
}

void ExceptionalCondition(const char *condition_name, const char *error_type, const char *file_name, int line_number) {
    (void)condition_name;
    (void)error_type;
    (void)file_name;
    (void)line_number;
    tqvector_backend_only("ExceptionalCondition is unavailable outside a PostgreSQL backend");
}

void *BuildIndexInfo(void *index) {
    (void)index;
    return tqvector_backend_only_ptr("BuildIndexInfo is unavailable outside a PostgreSQL backend");
}

uint32_t IndexGetRelation(uint32_t index_id, bool missing_ok) {
    (void)index_id;
    (void)missing_ok;
    return (uint32_t)tqvector_backend_only_uintptr("IndexGetRelation is unavailable outside a PostgreSQL backend");
}

void *index_open(uint32_t relation_id, int lockmode) {
    (void)relation_id;
    (void)lockmode;
    return tqvector_backend_only_ptr("index_open is unavailable outside a PostgreSQL backend");
}

void index_close(void *relation, int lockmode) {
    (void)relation;
    (void)lockmode;
    tqvector_backend_only("index_close is unavailable outside a PostgreSQL backend");
}

void *relation_open(uint32_t relation_id, int lockmode) {
    (void)relation_id;
    (void)lockmode;
    return tqvector_backend_only_ptr("relation_open is unavailable outside a PostgreSQL backend");
}

void relation_close(void *relation, int lockmode) {
    (void)relation;
    (void)lockmode;
    tqvector_backend_only("relation_close is unavailable outside a PostgreSQL backend");
}

void *table_open(uint32_t relation_id, int lockmode) {
    (void)relation_id;
    (void)lockmode;
    return tqvector_backend_only_ptr("table_open is unavailable outside a PostgreSQL backend");
}

void table_close(void *relation, int lockmode) {
    (void)relation;
    (void)lockmode;
    tqvector_backend_only("table_close is unavailable outside a PostgreSQL backend");
}

void *RelationGetIndexList(void *relation) {
    (void)relation;
    return tqvector_backend_only_ptr("RelationGetIndexList is unavailable outside a PostgreSQL backend");
}

void list_free(void *list) {
    (void)list;
}

void CacheRegisterRelcacheCallback(void *callback, uintptr_t arg) {
    (void)callback;
    (void)arg;
}

void *GetActiveSnapshot(void) {
    return tqvector_backend_only_ptr("GetActiveSnapshot is unavailable outside a PostgreSQL backend");
}

int GetDatabaseEncoding(void) {
    return 6;
}

uint32_t getBaseType(uint32_t typid) {
    return typid;
}

uint32_t get_array_type(uint32_t typid) {
    (void)typid;
    return 0;
}

uint32_t get_index_am_oid(const char *amname, bool missing_ok) {
    (void)amname;
    (void)missing_ok;
    return 0;
}

void get_typlenbyvalalign(uint32_t typid, int16_t *typlen, bool *typbyval, char *typalign) {
    (void)typid;
    if (typlen != NULL) {
        *typlen = -1;
    }
    if (typbyval != NULL) {
        *typbyval = false;
    }
    if (typalign != NULL) {
        *typalign = 'i';
    }
}

void getTypeInputInfo(uint32_t type, uint32_t *typInput, uint32_t *typIOParam) {
    (void)type;
    if (typInput != NULL) {
        *typInput = 0;
    }
    if (typIOParam != NULL) {
        *typIOParam = 0;
    }
}

void getTypeOutputInfo(uint32_t type, uint32_t *typOutput, bool *typIsVarlena) {
    (void)type;
    if (typOutput != NULL) {
        *typOutput = 0;
    }
    if (typIsVarlena != NULL) {
        *typIsVarlena = false;
    }
}

void getTypeBinaryInputInfo(uint32_t type, uint32_t *typReceive, uint32_t *typIOParam) {
    (void)type;
    if (typReceive != NULL) {
        *typReceive = 0;
    }
    if (typIOParam != NULL) {
        *typIOParam = 0;
    }
}

uintptr_t InputFunctionCall(void *flinfo, char *str, uint32_t typioparam, int32_t typmod) {
    (void)flinfo;
    (void)str;
    (void)typioparam;
    (void)typmod;
    return tqvector_backend_only_uintptr("InputFunctionCall is unavailable outside a PostgreSQL backend");
}

char *OutputFunctionCall(void *flinfo, uintptr_t val) {
    (void)flinfo;
    (void)val;
    return (char *)tqvector_backend_only_ptr("OutputFunctionCall is unavailable outside a PostgreSQL backend");
}

uintptr_t ReceiveFunctionCall(void *flinfo, void *buf, uint32_t typioparam, int32_t typmod) {
    (void)flinfo;
    (void)buf;
    (void)typioparam;
    (void)typmod;
    return tqvector_backend_only_uintptr("ReceiveFunctionCall is unavailable outside a PostgreSQL backend");
}

void fmgr_info(uint32_t functionId, void *finfo) {
    (void)functionId;
    (void)finfo;
}

void *pg_detoast_datum(void *datum) {
    return datum;
}

void *pg_detoast_datum_packed(void *datum) {
    return datum;
}

void *CreateTupleDescCopyConstr(void *tupdesc) {
    (void)tupdesc;
    return tqvector_backend_only_ptr("CreateTupleDescCopyConstr is unavailable outside a PostgreSQL backend");
}

void DecrTupleDescRefCount(void *tupdesc) {
    (void)tupdesc;
}

void *MakeSingleTupleTableSlot(void *tupdesc, const void *tts_ops) {
    (void)tupdesc;
    (void)tts_ops;
    return tqvector_backend_only_ptr("MakeSingleTupleTableSlot is unavailable outside a PostgreSQL backend");
}

void ExecDropSingleTupleTableSlot(void *slot) {
    (void)slot;
    tqvector_backend_only("ExecDropSingleTupleTableSlot is unavailable outside a PostgreSQL backend");
}

void ExecStoreVirtualTuple(void *slot) {
    (void)slot;
    tqvector_backend_only("ExecStoreVirtualTuple is unavailable outside a PostgreSQL backend");
}

void slot_getsomeattrs_int(void *slot, int natts) {
    (void)slot;
    (void)natts;
    tqvector_backend_only("slot_getsomeattrs_int is unavailable outside a PostgreSQL backend");
}

const void *table_slot_callbacks(void *relation) {
    (void)relation;
    return tqvector_backend_only_ptr("table_slot_callbacks is unavailable outside a PostgreSQL backend");
}

void *ExecInitExpr(void *node, void *parent) {
    (void)node;
    (void)parent;
    return tqvector_backend_only_ptr("ExecInitExpr is unavailable outside a PostgreSQL backend");
}

void *ExecScan(void *scan_state, void *access_mtd, void *recheck_mtd) {
    (void)scan_state;
    (void)access_mtd;
    (void)recheck_mtd;
    return tqvector_backend_only_ptr("ExecScan is unavailable outside a PostgreSQL backend");
}

uint32_t exprType(const void *expr) {
    (void)expr;
    return 0;
}

void ExplainPropertyInteger(const char *qlabel, const char *unit, int64_t value, void *es) {
    (void)qlabel;
    (void)unit;
    (void)value;
    (void)es;
}

void ExplainPropertyUInteger(const char *qlabel, const char *unit, uint64_t value, void *es) {
    (void)qlabel;
    (void)unit;
    (void)value;
    (void)es;
}

void ExplainPropertyText(const char *qlabel, const char *value, void *es) {
    (void)qlabel;
    (void)value;
    (void)es;
}

uint32_t BufferGetBlockNumber(int buffer) {
    (void)buffer;
    return (uint32_t)tqvector_backend_only_uintptr("BufferGetBlockNumber is unavailable outside a PostgreSQL backend");
}

void LockBuffer(int buffer, int mode) {
    (void)buffer;
    (void)mode;
    tqvector_backend_only("LockBuffer is unavailable outside a PostgreSQL backend");
}

int ReadBufferExtended(void *relation, uint32_t forknum, uint32_t blocknum, int mode, void *strategy) {
    (void)relation;
    (void)forknum;
    (void)blocknum;
    (void)mode;
    (void)strategy;
    return (int)tqvector_backend_only_uintptr("ReadBufferExtended is unavailable outside a PostgreSQL backend");
}

void ReleaseBuffer(int buffer) {
    (void)buffer;
    tqvector_backend_only("ReleaseBuffer is unavailable outside a PostgreSQL backend");
}

void UnlockReleaseBuffer(int buffer) {
    (void)buffer;
    tqvector_backend_only("UnlockReleaseBuffer is unavailable outside a PostgreSQL backend");
}

void *read_stream_begin_relation(int flags, void *strategy, void *relation, uint32_t forknum, void *callback, void *callback_private, size_t per_buffer_data_size) {
    (void)flags;
    (void)strategy;
    (void)relation;
    (void)forknum;
    (void)callback;
    (void)callback_private;
    (void)per_buffer_data_size;
    return tqvector_backend_only_ptr("read_stream_begin_relation is unavailable outside a PostgreSQL backend");
}

int read_stream_next_buffer(void *stream, void **per_buffer_data) {
    (void)stream;
    (void)per_buffer_data;
    return (int)tqvector_backend_only_uintptr("read_stream_next_buffer is unavailable outside a PostgreSQL backend");
}

void read_stream_end(void *stream) {
    (void)stream;
    tqvector_backend_only("read_stream_end is unavailable outside a PostgreSQL backend");
}

int SPI_fnumber(void *tupdesc, const char *fname) {
    (void)tupdesc;
    (void)fname;
    tqvector_backend_only("SPI_fnumber is unavailable outside a PostgreSQL backend");
    return -1;
}

void *initArrayResult(uint32_t element_type, MemoryContext rcontext, bool subcontext) {
    (void)element_type;
    (void)rcontext;
    (void)subcontext;
    return tqvector_backend_only_ptr("initArrayResult is unavailable outside a PostgreSQL backend");
}

void *accumArrayResult(void *astate, uintptr_t dvalue, bool disnull, uint32_t element_type, MemoryContext rcontext) {
    (void)astate;
    (void)dvalue;
    (void)disnull;
    (void)element_type;
    (void)rcontext;
    return tqvector_backend_only_ptr("accumArrayResult is unavailable outside a PostgreSQL backend");
}

uintptr_t makeArrayResult(void *astate, MemoryContext rcontext) {
    (void)astate;
    (void)rcontext;
    return tqvector_backend_only_uintptr("makeArrayResult is unavailable outside a PostgreSQL backend");
}

size_t MemoryContextMemConsumed(MemoryContext context) {
    (void)context;
    return 0;
}
