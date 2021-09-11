#include "libc.h"

// cat

int main(int argc, char *argv[])
{
    if (argc != 2) {
        printf("usage: cat file_path\n");
        return -1;
    }

    char *path = argv[argc - 1];

    int fd = open(path, O_RDONLY);
    char *buf = malloc(2048);
    int count;
    if ((count = read(fd, buf, 2048)) == -1) {
        printf("Can't read %s", path);
        return -1;
    }
    buf[count] = '\0';
    printf("%s\n", buf);
    return 0;
}