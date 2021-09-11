#include "libc.h"

// ls

int main()
{
    int fd = open("./", O_RDONLY | O_DIRECTORY);
    if (fd == -1) {
        printf("ls: error");
        return -1;
    }

    DIR *stream = fdopendir(fd);
    if(stream == NULL) {
        printf("ls: error");
        return -1;
    }

    struct dirent *dir;
    while ((dir = readdir(stream)) != NULL) {
        printf("%s\n", dir->name);
    }

    return 0;
}