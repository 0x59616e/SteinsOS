#include "libc.h"
#include<stdarg.h>

int fork()
{
    asm("svc " SYS_FORK);
}

int exec(const char *pathname, char *const argv[])
{
    asm("svc " SYS_EXEC);
}

int open(const char *pathname, int flags) {
    asm("svc " SYS_OPEN);
}

int write(int fd, const void *buf, int count)
{
    asm("svc " SYS_WRITE);
}

int read(int fd, void *buf, int count)
{
    asm("svc " SYS_READ);
}

int waitpid(int pid, int *wstatus)
{
    asm("svc " SYS_WAITPID);
}

char *fgets(char *s, int size, int fd)
{
    int len = read(fd, s, size - 1);
    if (len == -1) {
        return NULL;
    }
    s[len] = '\0';
    return s;
}

int fputs(const char *s, int fd) {
    int i = 0;
    while(s[i] != '\0') i++;

    return write(fd, s, i);
}

void put_buffer(char *buf, int *pos, int len, char c)
{
    buf[*pos] = c;
    if (*pos == len - 1) {
        write(STDOUT_FILENO, buf, len);
        memset(buf, 0, len);
    }
    *pos = (*pos + 1) % len;
}

int printf(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);

    char buf[256] = {0};
    int curr = 0;

    for (int i = 0; fmt[i] != '\0'; i++) {
        if (fmt[i] == '%' && fmt[i + 1] == 's') {
            i++;
            // string
            char *s = va_arg(ap, char *);
            for (int j = 0; s[j] != '\0'; j++) {
                put_buffer(buf, &curr, 256, s[j]);
            }
        } else if (fmt[i] == '%' && fmt[i + 1] == 'd') {
            i++;
            // int
            int v = va_arg(ap, int);
            int i = 1;
            while (v / (i * 10)) i *= 10;
            do {
                put_buffer(buf, &curr, 256, v / i + '0');
                v %= i;
                i /= 10;
            } while (i > 0);
        } else {
            put_buffer(buf, &curr, 256, fmt[i]);
        }
    }

    va_end(ap);
    fputs(buf, STDOUT_FILENO);
}

void *memset(void *s, int c, size_t n) {
    for (int i = 0; i < n; i++) {
        *(char *)(s + i) = (char)c;
    }
}

DIR *fdopendir(int fd)
{
    DIR *dir = malloc(sizeof(DIR));
    if (dir == NULL) {
        return NULL;
    }
    dir->fd = fd;
    dir->offset = 0;
    dir->len = 0;
    dir->buffer = NULL;
    return dir;
}

struct dirent *readdir(DIR *stream) {
    if (stream->len == 0) {
        // FIXME: File system doesn't allow directory of size over 1024 bytes
        stream->buffer = malloc(1024);
        stream->len = getdents(stream->fd, stream->buffer, 1024);
    } else if (stream->offset == stream->len) {
        return NULL;
    }

    struct dirent *dir = (struct dirent *)(stream->buffer + stream->offset);
    stream->offset += sizeof(struct dirent);
    return dir;
}

int getdents(unsigned int fd, struct dirent *dirp, unsigned int count) {
    asm("svc " SYS_GETDENTS);
}

void *sbrk(size_t inc) {
    asm("svc " SYS_SBRK);
}
