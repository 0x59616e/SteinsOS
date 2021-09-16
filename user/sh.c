#include "libc.h"

// shell

int main()
{
    for(;;) {
        fputs("$ ", STDOUT_FILENO);
        // read command
        char cmd[32];
        
        if (fgets(cmd, 32, STDIN_FILENO) != NULL) {
            if(cmd[0] == '\0')
                continue;

            if(cmd[0] == 'c' && cmd[1] == 'd' && cmd[2] == ' ') {
                // change working directory
                char *path = cmd + 3;
                if (chdir(path) == -1) {
                    printf("Can't change directory to %s\n", path);
                }
                continue;
            }

            int pid = fork();

            if (pid == 0) {
                // child process
                // parse command
                int i = 0;
                char *argv[10] = {NULL};

                for (int j = 0;; j++) {
                    argv[j] = cmd + i;
                    while(cmd[i] != ' ' && cmd[i] != '\0') i++;

                    if(cmd[i] == ' ')
                        cmd[i++] = '\0';
                    else if(cmd[i] == '\0')
                        break;
                }

                char *pathname = argv[0];

                exec(pathname, argv);
                char new_path[32] = {NULL};
                new_path[0] = '/';
                for (int i = 0; pathname[i] != '\0'; i++) {
                    new_path[i + 1] = pathname[i];
                }
                exec(new_path, argv);
                printf("%s: No such file or directory\n", pathname);
                return -1;
            }

            waitpid(pid, NULL);
        }
    }
}
