/* SPDX-License-Identifier: MIT
 * Installed-header-only extension conformance. No Rust/private source access.
 */
#define _GNU_SOURCE
#include <replai.h>
#include <assert.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <termios.h>
#include <time.h>
#include <unistd.h>

static replai_span semantic(const char *text, uint32_t role) {
    replai_span s = {0};
    s.struct_size = sizeof s; s.extension_version = REPLAI_PRESENTATION_VERSION;
    s.text = (const uint8_t *)text; s.text_bytes = strlen(text); s.role = role;
    return s;
}
static replai_text text(const replai_span *s, size_t count) {
    replai_text t = {0}; t.struct_size = sizeof t;
    t.extension_version = REPLAI_PRESENTATION_VERSION; t.spans = s; t.span_count = count;
    return t;
}
static replai_block block(uint32_t kind) {
    replai_block b = {0}; b.struct_size = sizeof b;
    b.extension_version = REPLAI_PRESENTATION_VERSION; b.kind = kind;
    b.text = text(NULL, 0); b.label = text(NULL, 0); return b;
}
static replai_event event(void) {
    replai_event e = {0}; e.struct_size = sizeof e; e.abi_version = REPLAI_C_ABI_VERSION; return e;
}
static void drain(int master) {
    uint8_t bytes[8192]; while (read(master, bytes, sizeof bytes) > 0) {}
    assert(errno == EAGAIN || errno == EWOULDBLOCK);
}
static void feed(replai_handle *h, int master, const char *s) {
    assert(write(master, s, strlen(s)) == (ssize_t)strlen(s));
    replai_event e = event(); assert(replai_advance(h, REPLAI_WAKE_INPUT, 0, &e) == REPLAI_OK);
    drain(master);
}
static double milliseconds(void) {
    struct timespec t; assert(clock_gettime(CLOCK_MONOTONIC, &t) == 0);
    return (double)t.tv_sec * 1000.0 + (double)t.tv_nsec / 1000000.0;
}
int main(void) {
    uint32_t version = 0; size_t required = 0, cells = 0;
    assert(replai_presentation_version(&version) == REPLAI_OK && version == 1);
    const char *unicode = "é界👩‍💻";
    assert(replai_text_cells((const uint8_t *)unicode, strlen(unicode), &cells) == REPLAI_OK && cells == 5);
    assert(replai_text_cells((const uint8_t *)"a\n", 2, &cells) == REPLAI_INVALID_TEXT && cells == 5);
    replai_span title = semantic("Object 界", REPLAI_ROLE_STRONG);
    replai_span key = semantic("state", REPLAI_ROLE_DIM), value = semantic("READY", REPLAI_ROLE_SUCCESS);
    replai_block b[3] = {block(REPLAI_BLOCK_PARAGRAPH), block(REPLAI_BLOCK_FIELD), block(REPLAI_BLOCK_LITERAL)};
    b[0].text = text(&title, 1); b[1].label = text(&key, 1); b[1].text = text(&value, 1);
    replai_span literal = semantic("  x\ty\n  界", REPLAI_ROLE_DEFAULT); b[2].text = text(&literal, 1);
    replai_render options = {0}; options.struct_size = sizeof options;
    options.extension_version = REPLAI_PRESENTATION_VERSION; options.columns = 80; options.indent = 2;
    for (uint32_t width = 4; width <= 120; width++) {
        options.columns = width;
        assert(replai_document_render(b, 3, &options, NULL, 0, &required) == REPLAI_OK);
        uint8_t *out = malloc(required + 1); assert(out); memset(out, 0xa5, required + 1);
        assert(replai_document_render(b, 3, &options, out, required - 1, &required) == REPLAI_BUFFER_TOO_SMALL);
        for (size_t i = 0; i <= required; i++) assert(out[i] == 0xa5);
        assert(replai_document_render(b, 3, &options, out, required, &required) == REPLAI_OK);
        assert(out[required] == 0xa5 && !memchr(out, 27, required)); free(out);
    }
    options.columns = 80; options.styled = 1;
    uint8_t output[4096]; assert(replai_document_render(b, 3, &options, output, sizeof output, &required) == REPLAI_OK);
    assert(memchr(output, 27, required));
    options.extension_version++;
    assert(replai_document_render(b, 3, &options, output, sizeof output, &required) == REPLAI_ABI_MISMATCH);
    options.extension_version--; options.styled = 0;
    title.role = UINT32_MAX;
    assert(replai_document_render(b, 3, &options, output, sizeof output, &required) == REPLAI_INVALID_ARGUMENT);
    title.role = REPLAI_ROLE_STRONG;
    title.text = (const uint8_t *)"\x1b"; title.text_bytes = 1;
    assert(replai_document_render(b, 3, &options, output, sizeof output, &required) == REPLAI_INVALID_TEXT);
    title = semantic("Object 界", REPLAI_ROLE_STRONG);
    assert(replai_document_render(NULL, 4097, &options, output, sizeof output, &required) == REPLAI_CAPACITY);
    replai_text headings[2] = {text(&key, 1), text(&title, 1)};
    replai_text fields[2] = {text(&value, 1), text(&literal, 1)};
    replai_block table[2] = {block(REPLAI_BLOCK_TABLE_HEADER), block(REPLAI_BLOCK_TABLE_ROW)};
    table[0].cells = headings; table[0].cell_count = 2;
    table[0].level = 2; /* Independent generic column alignment, not caller padding. */
    table[1].cells = fields; table[1].cell_count = 2;
    for (uint32_t width = 4; width <= 120; width++) {
        options.columns = width;
        assert(replai_document_render(table, 2, &options, output, sizeof output, &required) == REPLAI_OK);
        assert(!memchr(output, 27, required));
    }
    table[1].cell_count = 1;
    assert(replai_document_render(table, 2, &options, output, sizeof output, &required) == REPLAI_INVALID_RANGE);
    table[1].cell_count = 2; table[1].reserved[0] = 1;
    assert(replai_document_render(table, 2, &options, output, sizeof output, &required) == REPLAI_INVALID_ARGUMENT);
    table[1].reserved[0] = 0;
    table[0].level = 4;
    assert(replai_document_render(table, 2, &options, output, sizeof output, &required) == REPLAI_INVALID_ARGUMENT);
    table[0].level = 2;
    assert(replai_document_render(table + 1, 1, &options, output, sizeof output, &required) == REPLAI_INVALID_ARGUMENT);
    options.columns = 80;
    puts("DOCUMENT widths=4..120 plain/styled exact-buffer unsafe-text/version/tag/oversized refusal PASS");

    int master = posix_openpt(O_RDWR | O_NOCTTY | O_NONBLOCK);
    assert(master >= 0 && grantpt(master) == 0 && unlockpt(master) == 0);
    int slave = open(ptsname(master), O_RDWR | O_NOCTTY); assert(slave >= 0);
    struct winsize size = {.ws_row = 24, .ws_col = 80}; assert(ioctl(slave, TIOCSWINSZ, &size) == 0);
    struct termios original; assert(tcgetattr(slave, &original) == 0);
    replai_config config = {0}; config.struct_size = sizeof config; config.abi_version = REPLAI_C_ABI_VERSION;
    config.max_input_bytes = 1024; config.history_entries = 4;
    replai_handle *h = NULL; assert(replai_create(&config, &h) == REPLAI_OK);
    replai_span prompts[2] = {semantic("host", REPLAI_ROLE_ACCENT), semantic(" › ", REPLAI_ROLE_DIM)};
    replai_text primary = text(prompts, 2), continuation = text(NULL, 0);
    assert(replai_prompt_composed(h, &primary, &continuation) == REPLAI_OK);
    for (int iteration = 0; iteration < 16; iteration++) {
        assert(replai_set_draft(h, (const uint8_t *)"a", 1) == REPLAI_OK);
        size_t cursor = 0; uint64_t ticket = 0;
        assert(replai_completion_snapshot(h, output, sizeof output, &required, &cursor, &ticket) == REPLAI_OK);
        assert(required == 1 && cursor == 1 && ticket);
        assert(replai_set_draft(h, (const uint8_t *)"b", 1) == REPLAI_OK);
        assert(replai_open(h, slave, slave) == REPLAI_OK); drain(master);
        uint32_t disposition = UINT32_MAX;
        assert(replai_completions_present(h, ticket, NULL, SIZE_MAX, &disposition) == REPLAI_OK && disposition == REPLAI_STALE);
        assert(read(master, output, sizeof output) == -1 && errno == EAGAIN);
        assert(replai_completion_snapshot(h, output, sizeof output, &required, &cursor, &ticket) == REPLAI_OK);
        replai_candidate c[2] = {{0}, {0}};
        for (size_t i = 0; i < 2; i++) {
            c[i].struct_size = sizeof c[i]; c[i].extension_version = REPLAI_PRESENTATION_VERSION;
            c[i].start = 0; c[i].end = 1;
            c[i].insertion = (const uint8_t *)(i ? "beta" : "bravo"); c[i].insertion_bytes = i ? 4 : 5;
            c[i].label = c[i].insertion; c[i].label_bytes = c[i].insertion_bytes;
            c[i].annotation = (const uint8_t *)"candidate"; c[i].annotation_bytes = 9;
        }
        assert(replai_completions_present(h, ticket, c, 2, &disposition) == REPLAI_OK && disposition == REPLAI_APPLIED);
        drain(master);
        double began = milliseconds(); feed(h, master, "\x1b"); double escape_ms = milliseconds() - began;
        // Preserve fragmented CSI: Escape preview does not destroy candidates.
        feed(h, master, "["); feed(h, master, "Z"); feed(h, master, "\r");
        assert(replai_draft_copy(h, output, sizeof output, &required, &cursor) == REPLAI_OK);
        assert(required == 4 && !memcmp(output, "beta", 4));
        assert(replai_document_output(h, b, 3) == REPLAI_OK); drain(master);
        replai_interest interest = {0}; interest.struct_size = sizeof interest; interest.extension_version = REPLAI_PRESENTATION_VERSION;
        assert(replai_wait_interest(h, &interest) == REPLAI_OK && interest.kind == REPLAI_WAIT_INPUT && interest.timeout_ms == -1);
        assert(interest.input_fd >= 0 && fcntl(interest.input_fd, F_GETFD) >= 0);
        size.ws_col = iteration % 2 ? 38 : 96; assert(ioctl(slave, TIOCSWINSZ, &size) == 0);
        replai_event e = event(); began = milliseconds();
        assert(replai_advance(h, REPLAI_WAKE_RESIZE, 0, &e) == REPLAI_OK); double resize_ms = milliseconds() - began;
        drain(master);
        e = event(); assert(replai_advance(h, REPLAI_WAKE_DEADLINE, UINT64_MAX, &e) == REPLAI_OK && e.kind == REPLAI_EVENT_NONE);
        assert(replai_close(h) == REPLAI_OK);
        assert(replai_output_open(h, slave, slave) == REPLAI_OK);
        struct termios quiet; assert(tcgetattr(slave, &quiet) == 0);
        assert(!(quiet.c_lflag & ECHO) && (quiet.c_lflag & ISIG) == (original.c_lflag & ISIG));
        replai_text feedback = text(&value, 1);
        assert(replai_output_feedback(h, &feedback) == REPLAI_OK);
        feedback = text(NULL, 0); assert(replai_output_feedback(h, &feedback) == REPLAI_OK);
        assert(replai_output_close(h, 2) == REPLAI_INVALID_ARGUMENT);
        assert(replai_output_close(h, 1) == REPLAI_OK);
        struct termios restored; assert(tcgetattr(slave, &restored) == 0);
        assert(original.c_lflag == restored.c_lflag && original.c_iflag == restored.c_iflag && original.c_oflag == restored.c_oflag);
        assert(!memcmp(original.c_cc, restored.c_cc, sizeof original.c_cc)); drain(master);
        printf("PTY iteration=%d menu/stale/driven/feedback/restoration=PASS escape_preview_ms=%.3f resize_notify_ms=%.3f\n", iteration, escape_ms, resize_ms);
    }
    assert(replai_destroy(&h) == REPLAI_OK && !h); close(slave); close(master);
    return 0;
}
