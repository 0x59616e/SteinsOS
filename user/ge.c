#include "libc.h"

void int_handler(int sig)
{

}

int main()
{
    struct sigaction sig = {
        .sa_handler = int_handler,
    };

    sigaction(SIGINT, &sig, NULL);

    return 0;
}