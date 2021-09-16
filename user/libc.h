#define SYS_FORK     "0x00"
#define SYS_EXEC     "0x01"
#define SYS_OPEN     "0x02"
#define SYS_READ     "0x03"
#define SYS_WRITE    "0x04"
#define SYS_CLOSE    "0x05"
#define SYS_WAITPID  "0x06"
#define SYS_EXIT     "0x07"
#define SYS_GETDENTS "0x08"
#define SYS_SBRK     "0x09"
#define SYS_GETCWD   "0x0A"
#define SYS_MKDIR    "0x0B"
#define SYS_CHDIR    "0x0C"

#define STDIN_FILENO 0
#define STDOUT_FILENO 1

#define O_RDONLY    1
#define O_WRONLY    2
#define O_RDWR      4
#define O_DIRECTORY 8

#define NULL (void *)0

typedef long long int size_t;
typedef struct DIR {
    int fd;
    int offset;
    int len;
    void *buffer;
} DIR;

struct dirent {
    unsigned int d_ino;
    char name[12];
};

// system call
int fork();
int exec(const char *, char *const argv[]);
int open(const char *, int flags);
int write(int fd, const void *buf, int count);
int  read(int fd, void *buf, int count);
int waitpid(int pid, int *wstatus);
int getdents(unsigned int, struct dirent *, unsigned int);
void *sbrk(size_t);
char *getcwd(char *, size_t);
int mkdir(char *);
int chdir(char *);


// library
char *fgets(char *s, int size, int fd);
int fputs(const char *s, int fd);
int printf(const char *fmt, ...);
void *memset(void *s, int c, size_t n);
DIR *fdopendir(int);
struct dirent *readdir(DIR *);
void *malloc(size_t);
void free(void *);