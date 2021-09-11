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
                printf("%s: No such file or directory\n", cmd);
                return -1;
            }

            waitpid(pid, NULL);
        }
    }
}
