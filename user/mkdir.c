#include "libc.h"

int main(int argc, char *argv[])
{
    if (argc == 1) {
        printf("mkdir: no arguments\n");
        return -1;
    }

    char *path = argv[1];

    if (mkdir(path) == -1) {
        printf("mkdir: failed\n");
        return -1;
    }

    return 0;
}