#include "libc.h"

int main(int argc, char *argv[]) {
    char *buffer = malloc(1024);
    if (getcwd(buffer, 1024) == NULL) {
        printf("can't get cwd\n");
        return -1;
    }

    printf("%s\n", buffer);
    return 0;
}
