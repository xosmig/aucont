#include <stdio.h>

int redirect_stdin(const char* path) {
    return (freopen(path, "r", stdin) == NULL) ? -1 : 0;
}

int redirect_stdout(const char* path) {
    return (freopen(path, "w", stdout) == NULL) ? -1 : 0;
}

int redirect_stderr(const char* path) {
    return (freopen(path, "w", stdout) == NULL) ? -1 : 0;
}
