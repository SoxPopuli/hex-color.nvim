#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct BufferColor BufferColor;

typedef struct BufferColors BufferColors;

typedef struct RgbString {
  char s[6];
} RgbString;

uintptr_t buffer_color_row_index(const struct BufferColor *b);

uintptr_t buffer_color_col_start(const struct BufferColor *b);

uintptr_t buffer_color_col_end(const struct BufferColor *b);

struct RgbString buffer_color_hex_string(const struct BufferColor *b);

struct RgbString buffer_color_foreground_hex(const struct BufferColor *b);

struct BufferColors *buffer_colors_init(void);

bool buffer_colors_has_value(const struct BufferColors *b);

const char *buffer_colors_error_msg(const struct BufferColors *b);

struct BufferColor *buffer_colors_value(struct BufferColors *b, uintptr_t idx);

uintptr_t buffer_colors_value_len(const struct BufferColors *b);

void buffer_colors_add_line(struct BufferColors *b,
                            uintptr_t row_index,
                            const char *line,
                            uintptr_t line_len);

void buffer_colors_free(struct BufferColors *b);
