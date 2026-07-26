#define _GNU_SOURCE

#include <dlfcn.h>
#include <errno.h>
#include <fcntl.h>
#include <arpa/inet.h>
#include <netinet/in.h>
#include <stdint.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/syscall.h>
#include <sys/un.h>
#include <sys/uio.h>
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
    if (!value || strcmp(value, "1") != 0) {
        return 0;
    }
    /*
     * Long-lived postmasters need to finish remote fixture setup before a
     * narrowly targeted transport fault is armed. When configured, file
     * existence is the operator-controlled gate; removing the file disarms
     * injection without restarting the postmaster.
     */
    const char *arm_file = getenv("ECAZ_FAULT_PROVIDER_ARM_FILE");
    return !arm_file || !*arm_file || access(arm_file, F_OK) == 0;
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

static int peer_target_matches(int fd, char *target, size_t target_size) {
    const char *expected = getenv("ECAZ_FAULT_PROVIDER_PEER");
    if (!expected || !*expected || target_size == 0) {
        return 0;
    }

    /*
     * getpeername(2) on a regular file sets errno. Preserve the caller's errno
     * when this descriptor is not the configured TCP peer so merely enabling a
     * socket mode cannot perturb unrelated file I/O.
     */
    int saved_errno = errno;
    struct sockaddr_storage address;
    socklen_t address_len = sizeof(address);
    if (getpeername(fd, (struct sockaddr *)&address, &address_len) != 0) {
        target[0] = '\0';
        errno = saved_errno;
        return 0;
    }

    if (address.ss_family == AF_UNIX) {
        const struct sockaddr_un *unix_address =
            (const struct sockaddr_un *)&address;
        /*
         * Accepted AF_UNIX sockets commonly report an unnamed peer. Abstract
         * peers also begin with NUL. Neither has a stable pathname identity,
         * so never turn either form into a matchable "unix:" key.
         */
        if (address_len <= offsetof(struct sockaddr_un, sun_path) ||
            unix_address->sun_path[0] == '\0') {
            target[0] = '\0';
            errno = saved_errno;
            return 0;
        }
        snprintf(target, target_size, "unix:%s", unix_address->sun_path);
    } else if (address.ss_family == AF_INET) {
        const struct sockaddr_in *inet_address =
            (const struct sockaddr_in *)&address;
        char host[INET_ADDRSTRLEN];
        if (!inet_ntop(AF_INET, &inet_address->sin_addr, host, sizeof(host))) {
            target[0] = '\0';
            errno = saved_errno;
            return 0;
        }
        snprintf(
            target,
            target_size,
            "tcp:%s:%u",
            host,
            (unsigned)ntohs(inet_address->sin_port));
    } else if (address.ss_family == AF_INET6) {
        const struct sockaddr_in6 *inet6_address =
            (const struct sockaddr_in6 *)&address;
        char host[INET6_ADDRSTRLEN];
        if (!inet_ntop(AF_INET6, &inet6_address->sin6_addr, host, sizeof(host))) {
            target[0] = '\0';
            errno = saved_errno;
            return 0;
        }
        snprintf(
            target,
            target_size,
            "tcp:[%s]:%u",
            host,
            (unsigned)ntohs(inet6_address->sin6_port));
    } else {
        target[0] = '\0';
        errno = saved_errno;
        return 0;
    }
    int matches = strcmp(target, expected) == 0;
    errno = saved_errno;
    return matches;
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

static int should_fault_socket(const char *mode, const char *op, int fd, int errnum) {
    char target[4096];
    if (!enabled() || !mode_is(mode) ||
        !peer_target_matches(fd, target, sizeof(target))) {
        return 0;
    }
    unsigned long long count =
        __atomic_add_fetch(&fault_counter, 1, __ATOMIC_RELAXED);
    if (count < after_count()) {
        return 0;
    }
    record_fault_event(mode, op, target, count, errnum);
    return 1;
}

static long latency_millis(void) {
    const char *value = getenv("ECAZ_FAULT_PROVIDER_LATENCY_MS");
    return value ? strtol(value, NULL, 10) : 0;
}

static void sleep_millis(long millis) {
    if (millis <= 0) {
        return;
    }
    struct timespec ts;
    ts.tv_sec = millis / 1000;
    ts.tv_nsec = (millis % 1000) * 1000000L;
    nanosleep(&ts, NULL);
}

static void maybe_sleep(void) {
    if (!enabled() || !mode_is("slow-disk")) {
        return;
    }
    sleep_millis(latency_millis());
}

static void maybe_sleep_socket(const char *op, int fd) {
    if (should_fault_socket("socket-slow", op, fd, 0)) {
        sleep_millis(latency_millis());
    }
}

static int maybe_reset_socket(const char *op, int fd) {
    if (!should_fault_socket("socket-reset", op, fd, ECONNRESET)) {
        return 0;
    }
    (void)syscall(SYS_shutdown, fd, SHUT_RDWR);
    errno = ECONNRESET;
    return 1;
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
    const char *peer = getenv("ECAZ_FAULT_PROVIDER_PEER");
    char line[512];
    int len = snprintf(
        line,
        sizeof(line),
        "pid=%ld mode=%s match=%s peer=%s latency_ms=%ld\n",
        (long)getpid(),
        mode ? mode : "unset",
        match ? match : "unset",
        peer ? peer : "unset",
        latency_millis());
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
    if (maybe_reset_socket("read", fd)) {
        return -1;
    }
    maybe_sleep_socket("read", fd);
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
    if (maybe_reset_socket("write", fd)) {
        return -1;
    }
    maybe_sleep_socket("write", fd);
    if (should_fault_fd("enospc-write", "write", fd, ENOSPC)) {
        errno = ENOSPC;
        return -1;
    }
    maybe_sleep();
    ssize_t (*real_write)(int, const void *, size_t) = real_symbol("write");
    return real_write ? real_write(fd, buf, count) : -1;
}

ssize_t recv(int fd, void *buf, size_t count, int flags) {
    if (maybe_reset_socket("recv", fd)) {
        return -1;
    }
    maybe_sleep_socket("recv", fd);
    ssize_t (*real_recv)(int, void *, size_t, int) = real_symbol("recv");
    return real_recv ? real_recv(fd, buf, count, flags) : -1;
}

ssize_t send(int fd, const void *buf, size_t count, int flags) {
    if (maybe_reset_socket("send", fd)) {
        return -1;
    }
    maybe_sleep_socket("send", fd);
    ssize_t (*real_send)(int, const void *, size_t, int) = real_symbol("send");
    return real_send ? real_send(fd, buf, count, flags) : -1;
}

ssize_t readv(int fd, const struct iovec *iov, int iovcnt) {
    if (maybe_reset_socket("readv", fd)) {
        return -1;
    }
    maybe_sleep_socket("readv", fd);
    ssize_t (*real_readv)(int, const struct iovec *, int) =
        real_symbol("readv");
    return real_readv ? real_readv(fd, iov, iovcnt) : -1;
}

ssize_t writev(int fd, const struct iovec *iov, int iovcnt) {
    if (maybe_reset_socket("writev", fd)) {
        return -1;
    }
    maybe_sleep_socket("writev", fd);
    ssize_t (*real_writev)(int, const struct iovec *, int) =
        real_symbol("writev");
    return real_writev ? real_writev(fd, iov, iovcnt) : -1;
}

ssize_t recvfrom(
    int fd,
    void *buf,
    size_t count,
    int flags,
    struct sockaddr *address,
    socklen_t *address_len) {
    if (maybe_reset_socket("recvfrom", fd)) {
        return -1;
    }
    maybe_sleep_socket("recvfrom", fd);
    ssize_t (*real_recvfrom)(
        int,
        void *,
        size_t,
        int,
        struct sockaddr *,
        socklen_t *) = real_symbol("recvfrom");
    return real_recvfrom
        ? real_recvfrom(fd, buf, count, flags, address, address_len)
        : -1;
}

ssize_t sendto(
    int fd,
    const void *buf,
    size_t count,
    int flags,
    const struct sockaddr *address,
    socklen_t address_len) {
    if (maybe_reset_socket("sendto", fd)) {
        return -1;
    }
    maybe_sleep_socket("sendto", fd);
    ssize_t (*real_sendto)(
        int,
        const void *,
        size_t,
        int,
        const struct sockaddr *,
        socklen_t) = real_symbol("sendto");
    return real_sendto
        ? real_sendto(fd, buf, count, flags, address, address_len)
        : -1;
}

ssize_t recvmsg(int fd, struct msghdr *message, int flags) {
    if (maybe_reset_socket("recvmsg", fd)) {
        return -1;
    }
    maybe_sleep_socket("recvmsg", fd);
    ssize_t (*real_recvmsg)(int, struct msghdr *, int) =
        real_symbol("recvmsg");
    return real_recvmsg ? real_recvmsg(fd, message, flags) : -1;
}

ssize_t sendmsg(int fd, const struct msghdr *message, int flags) {
    if (maybe_reset_socket("sendmsg", fd)) {
        return -1;
    }
    maybe_sleep_socket("sendmsg", fd);
    ssize_t (*real_sendmsg)(int, const struct msghdr *, int) =
        real_symbol("sendmsg");
    return real_sendmsg ? real_sendmsg(fd, message, flags) : -1;
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
