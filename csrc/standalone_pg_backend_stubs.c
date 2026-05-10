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

uint64_t SPI_processed = 0;
void *SPI_tuptable = NULL;

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
