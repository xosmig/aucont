#include <stdio.h>
#include <unistd.h>

int redirect_stdin(const char* path) {
    return (freopen(path, "r", stdin) == NULL) ? -1 : 0;
}

int redirect_stdout(const char* path) {
    return (freopen(path, "w", stdout) == NULL) ? -1 : 0;
}

int redirect_stderr(const char* path) {
    return (freopen(path, "w", stderr) == NULL) ? -1 : 0;
}

int redirect_stderr_to_stdout() {
    return dup2(fileno(stdout), fileno(stderr));
}
