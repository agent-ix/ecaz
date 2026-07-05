#define _GNU_SOURCE

#include <dlfcn.h>
#include <errno.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/syscall.h>
#include <time.h>
#include <unistd.h>

static unsigned long long fault_counter = 0;

struct open_how {
    uint64_t flags;
    uint64_t mode;
    uint64_t resolve;
};

static int enabled(void) {
    const char *value = getenv("ECAZ_FAULT_PROVIDER_ENABLE");
    return value && strcmp(value, "1") == 0;
}

static int mode_is(const char *mode) {
    const char *value = getenv("ECAZ_FAULT_PROVIDER_MODE");
    return value && strcmp(value, mode) == 0;
}

static unsigned long long after_count(void) {
    const char *value = getenv("ECAZ_FAULT_PROVIDER_AFTER");
    if (!value || !*value) {
        return 1;
    }
    unsigned long long parsed = strtoull(value, NULL, 10);
    return parsed == 0 ? 1 : parsed;
}

static int path_matches(const char *path) {
    const char *needle = getenv("ECAZ_FAULT_PROVIDER_MATCH");
    if (!needle || !*needle) {
        return 1;
    }
    return path && strstr(path, needle) != NULL;
}

static void append_marker_line(const char *line, size_t len) {
    const char *marker = getenv("ECAZ_FAULT_PROVIDER_MARKER");
    if (!marker || !*marker) {
        return;
    }
    int fd = (int)syscall(
        SYS_openat,
        AT_FDCWD,
        marker,
        O_CREAT | O_WRONLY | O_APPEND,
        0600);
    if (fd >= 0) {
        (void)syscall(SYS_write, fd, line, len);
        (void)syscall(SYS_close, fd);
    }
}

static void record_fault_event(
    const char *mode,
    const char *op,
    const char *target,
    unsigned long long count,
    int errnum) {
    char line[512];
    int len = snprintf(
        line,
        sizeof(line),
        "fault=1 pid=%ld mode=%s op=%s count=%llu errno=%d target=%s\n",
        (long)getpid(),
        mode ? mode : "unset",
        op ? op : "unset",
        count,
        errnum,
        target ? target : "unset");
    if (len > 0) {
        if ((size_t)len >= sizeof(line)) {
            len = (int)sizeof(line) - 1;
        }
        append_marker_line(line, (size_t)len);
    }
}

static int fd_target_matches(int fd, char *target, size_t target_size) {
    char link_path[64];
    snprintf(link_path, sizeof(link_path), "/proc/self/fd/%d", fd);
    ssize_t len = readlink(link_path, target, target_size - 1);
    if (len < 0) {
        if (target_size > 0) {
            target[0] = '\0';
        }
        return path_matches("");
    }
    target[len] = '\0';
    return path_matches(target);
}

static int should_fault_path(const char *mode, const char *op, const char *path, int errnum) {
    if (!enabled() || !mode_is(mode) || !path_matches(path)) {
        return 0;
    }
    unsigned long long count = __atomic_add_fetch(&fault_counter, 1, __ATOMIC_RELAXED);
    if (count < after_count()) {
        return 0;
    }
    record_fault_event(mode, op, path, count, errnum);
    return 1;
}

static int should_fault_fd(const char *mode, const char *op, int fd, int errnum) {
    char target[4096];
    if (!enabled() || !mode_is(mode) || !fd_target_matches(fd, target, sizeof(target))) {
        return 0;
    }
    unsigned long long count = __atomic_add_fetch(&fault_counter, 1, __ATOMIC_RELAXED);
    if (count < after_count()) {
        return 0;
    }
    record_fault_event(mode, op, target, count, errnum);
    return 1;
}

static void maybe_sleep(void) {
    if (!enabled() || !mode_is("slow-disk")) {
        return;
    }
    const char *value = getenv("ECAZ_FAULT_PROVIDER_LATENCY_MS");
    long millis = value ? strtol(value, NULL, 10) : 0;
    if (millis <= 0) {
        return;
    }
    struct timespec ts;
    ts.tv_sec = millis / 1000;
    ts.tv_nsec = (millis % 1000) * 1000000L;
    nanosleep(&ts, NULL);
}

static void *real_symbol(const char *name) {
    void *symbol = dlsym(RTLD_NEXT, name);
    if (!symbol) {
        errno = ENOSYS;
    }
    return symbol;
}

__attribute__((constructor)) static void ecaz_fault_provider_loaded(void) {
    const char *mode = getenv("ECAZ_FAULT_PROVIDER_MODE");
    const char *match = getenv("ECAZ_FAULT_PROVIDER_MATCH");
    char line[256];
    int len = snprintf(
        line,
        sizeof(line),
        "pid=%ld mode=%s match=%s\n",
        (long)getpid(),
        mode ? mode : "unset",
        match ? match : "unset");
    if (len <= 0) {
        return;
    }
    if ((size_t)len >= sizeof(line)) {
        len = (int)sizeof(line) - 1;
    }
    append_marker_line(line, (size_t)len);
}

int open(const char *path, int flags, ...) {
    mode_t mode = 0;
    if (flags & O_CREAT) {
        va_list ap;
        va_start(ap, flags);
        mode = (mode_t)va_arg(ap, int);
        va_end(ap);
    }
    if ((flags & O_CREAT) && should_fault_path("enospc-write", "open", path, ENOSPC)) {
        errno = ENOSPC;
        return -1;
    }
    maybe_sleep();
    int (*real_open)(const char *, int, ...) = real_symbol("open");
    if (!real_open) {
        return -1;
    }
    return (flags & O_CREAT) ? real_open(path, flags, mode) : real_open(path, flags);
}

int open64(const char *path, int flags, ...) {
    mode_t mode = 0;
    if (flags & O_CREAT) {
        va_list ap;
        va_start(ap, flags);
        mode = (mode_t)va_arg(ap, int);
        va_end(ap);
    }
    if ((flags & O_CREAT) && should_fault_path("enospc-write", "open64", path, ENOSPC)) {
        errno = ENOSPC;
        return -1;
    }
    maybe_sleep();
    int (*real_open64)(const char *, int, ...) = real_symbol("open64");
    if (!real_open64) {
        return -1;
    }
    return (flags & O_CREAT) ? real_open64(path, flags, mode) : real_open64(path, flags);
}

int openat(int dirfd, const char *path, int flags, ...) {
    mode_t mode = 0;
    if (flags & O_CREAT) {
        va_list ap;
        va_start(ap, flags);
        mode = (mode_t)va_arg(ap, int);
        va_end(ap);
    }
    if ((flags & O_CREAT) && should_fault_path("enospc-write", "openat", path, ENOSPC)) {
        errno = ENOSPC;
        return -1;
    }
    maybe_sleep();
    int (*real_openat)(int, const char *, int, ...) = real_symbol("openat");
    if (!real_openat) {
        return -1;
    }
    return (flags & O_CREAT)
        ? real_openat(dirfd, path, flags, mode)
        : real_openat(dirfd, path, flags);
}

int openat2(int dirfd, const char *path, const struct open_how *how, size_t size) {
    int flags = how ? (int)how->flags : 0;
    if ((flags & O_CREAT) && should_fault_path("enospc-write", "openat2", path, ENOSPC)) {
        errno = ENOSPC;
        return -1;
    }
    maybe_sleep();
    int (*real_openat2)(int, const char *, const struct open_how *, size_t) =
        real_symbol("openat2");
    return real_openat2 ? real_openat2(dirfd, path, how, size) : -1;
}

ssize_t read(int fd, void *buf, size_t count) {
    if (should_fault_fd("eio-read", "read", fd, EIO)) {
        errno = EIO;
        return -1;
    }
    maybe_sleep();
    ssize_t (*real_read)(int, void *, size_t) = real_symbol("read");
    return real_read ? real_read(fd, buf, count) : -1;
}

ssize_t pread(int fd, void *buf, size_t count, off_t offset) {
    if (should_fault_fd("eio-read", "pread", fd, EIO)) {
        errno = EIO;
        return -1;
    }
    maybe_sleep();
    ssize_t (*real_pread)(int, void *, size_t, off_t) = real_symbol("pread");
    return real_pread ? real_pread(fd, buf, count, offset) : -1;
}

ssize_t pread64(int fd, void *buf, size_t count, off64_t offset) {
    if (should_fault_fd("eio-read", "pread64", fd, EIO)) {
        errno = EIO;
        return -1;
    }
    maybe_sleep();
    ssize_t (*real_pread64)(int, void *, size_t, off64_t) = real_symbol("pread64");
    return real_pread64 ? real_pread64(fd, buf, count, offset) : -1;
}

ssize_t write(int fd, const void *buf, size_t count) {
    if (should_fault_fd("enospc-write", "write", fd, ENOSPC)) {
        errno = ENOSPC;
        return -1;
    }
    maybe_sleep();
    ssize_t (*real_write)(int, const void *, size_t) = real_symbol("write");
    return real_write ? real_write(fd, buf, count) : -1;
}

ssize_t pwrite(int fd, const void *buf, size_t count, off_t offset) {
    if (should_fault_fd("enospc-write", "pwrite", fd, ENOSPC)) {
        errno = ENOSPC;
        return -1;
    }
    maybe_sleep();
    ssize_t (*real_pwrite)(int, const void *, size_t, off_t) = real_symbol("pwrite");
    return real_pwrite ? real_pwrite(fd, buf, count, offset) : -1;
}

ssize_t pwrite64(int fd, const void *buf, size_t count, off64_t offset) {
    if (should_fault_fd("enospc-write", "pwrite64", fd, ENOSPC)) {
        errno = ENOSPC;
        return -1;
    }
    maybe_sleep();
    ssize_t (*real_pwrite64)(int, const void *, size_t, off64_t) = real_symbol("pwrite64");
    return real_pwrite64 ? real_pwrite64(fd, buf, count, offset) : -1;
}

int fsync(int fd) {
    if (should_fault_fd("enospc-write", "fsync", fd, ENOSPC)) {
        errno = ENOSPC;
        return -1;
    }
    maybe_sleep();
    int (*real_fsync)(int) = real_symbol("fsync");
    return real_fsync ? real_fsync(fd) : -1;
}

int fdatasync(int fd) {
    if (should_fault_fd("enospc-write", "fdatasync", fd, ENOSPC)) {
        errno = ENOSPC;
        return -1;
    }
    maybe_sleep();
    int (*real_fdatasync)(int) = real_symbol("fdatasync");
    return real_fdatasync ? real_fdatasync(fd) : -1;
}
