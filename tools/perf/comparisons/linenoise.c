/* Minimal consumer of the pinned upstream C API; blocking and edit-feed paths. */
#include "linenoise.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>
static void exit_gate(void) {
    const char *fd = getenv("P0_EXIT_FD");
    if (!fd) return;
    fputs("EXIT_READY\n", stderr); fflush(stderr);
    char byte; while (read(atoi(fd), &byte, 1) < 0 && errno == EINTR) {}
}
static void complete(const char *line, linenoiseCompletions *c) { (void)line; linenoiseAddCompletion(c, "replacement"); }
int main(int argc, char **argv) {
    atexit(exit_gate);
    int feed = argc > 1 && strcmp(argv[1], "feed") == 0;
    linenoiseSetCompletionCallback(complete);
    linenoiseHistorySetMaxLen(100);
    linenoiseHistoryAdd("history first");
    linenoiseHistoryAdd("history second");
    linenoiseSetMultiLine(1);
    for (;;) {
        char *line;
        if (feed) {
            struct linenoiseState state;
            char buffer[1024 * 1024 + 1];
            if (linenoiseEditStart(&state, 0, 1, buffer, sizeof(buffer), "p> ") != 0) return 2;
            do { line = linenoiseEditFeed(&state); } while (line == linenoiseEditMore);
            linenoiseEditStop(&state);
        } else line = linenoise("p> ");
        if (!line) break;
        fputs("SUBMITTED:", stderr);
        for (const unsigned char *p=(const unsigned char *)line; *p; p++) fprintf(stderr, "%02x", *p);
        fputc('\n', stderr);
        int done = strcmp(line, "exit") == 0;
        linenoiseFree(line);
        (void)done; break;
    }
    return 0;
}
