#include "libc.h"

struct list {
    int size;
    struct list *next;
};

struct list freelist = {
    .next = NULL,
    .size = 0,
};

void *malloc(size_t size)
{
    struct list *runner = &freelist;

    // 4 bytes align
    // and we need more 4 bytes to record the length
    size = (size + 7) & ~4;

    int *res = NULL;
    while (runner->next != NULL) {
        if (runner->next->size >= size) {
            // we use first-fit strategy
            res = (int *)runner->next;
            runner->next = runner->next->next;
            break;
        }

        runner = runner->next;
    }
    // can't find enough space
    // ask for more from OS
    if (res == NULL) {
        if ((res = (int *)sbrk(size)) == NULL) {
            return NULL;
        }
    }

    *res = size;
    return (void *)res + 4;
}

void free(void *ptr) {
    ptr -= 4;
    struct list *runner = &freelist;
    // find the place where it should be
    while (runner->next != NULL && ptr > runner->next)
    {
        runner = runner->next;
    }

    struct list *res = (struct list *)ptr;
    res->next = runner->next;
    runner->next = res;

    // merge block
    while ((void *)runner + runner->size == (void *)runner->next) {
        runner->size += runner->next->size;
        runner->next = runner->next->next;
    }
}