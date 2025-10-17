/* Yellow Terminal Library - C Bindings
 * Auto-maintained header for FFI
 */

#ifndef YELLOW_H
#define YELLOW_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque screen handle */
typedef struct YellowScreen YellowScreen;

/* Key codes - C representation of Rust enum */
typedef enum {
    YellowKey_Char = 0,
    YellowKey_ArrowUp,
    YellowKey_ArrowDown,
    YellowKey_ArrowLeft,
    YellowKey_ArrowRight,
    YellowKey_Enter,
    YellowKey_Backspace,
    YellowKey_Delete,
    YellowKey_Home,
    YellowKey_End,
    YellowKey_PageUp,
    YellowKey_PageDown,
    YellowKey_Tab,
    YellowKey_Escape,
    YellowKey_F1,
    YellowKey_F2,
    YellowKey_F3,
    YellowKey_F4,
    YellowKey_F5,
    YellowKey_F6,
    YellowKey_F7,
    YellowKey_F8,
    YellowKey_F9,
    YellowKey_F10,
    YellowKey_F11,
    YellowKey_F12,
    YellowKey_Unknown,
} YellowKeyTag;

/* Tagged union for key events */
typedef struct {
    YellowKeyTag tag;
    union {
        uint32_t char_value; /* For YellowKey_Char */
    } value;
} YellowKey;

/* Attribute constants */
#define YELLOW_ATTR_BOLD           1
#define YELLOW_ATTR_DIM            2
#define YELLOW_ATTR_ITALIC         4
#define YELLOW_ATTR_UNDERLINE      8
#define YELLOW_ATTR_BLINK          16
#define YELLOW_ATTR_REVERSE        32
#define YELLOW_ATTR_HIDDEN         64
#define YELLOW_ATTR_STRIKETHROUGH  128

/* Screen management */

/**
 * Initialize a new screen
 * Returns NULL on error
 */
YellowScreen* yellow_init(void);

/**
 * Clean up and restore terminal
 * Returns 0 on success, -1 on error
 */
int32_t yellow_endwin(YellowScreen* screen);

/**
 * Clear the screen
 * Returns 0 on success, -1 on error
 */
int32_t yellow_clear(YellowScreen* screen);

/**
 * Refresh the screen (flush output)
 * Returns 0 on success, -1 on error
 */
int32_t yellow_refresh(YellowScreen* screen);

/**
 * Move cursor to position (y, x)
 * Returns 0 on success, -1 on error
 */
int32_t yellow_move_cursor(YellowScreen* screen, uint16_t y, uint16_t x);

/**
 * Print string at current cursor position
 * Returns 0 on success, -1 on error
 */
int32_t yellow_print(YellowScreen* screen, const char* text);

/**
 * Print string at position (y, x)
 * Returns 0 on success, -1 on error
 */
int32_t yellow_mvprint(YellowScreen* screen, uint16_t y, uint16_t x, const char* text);

/**
 * Get a key from input
 * Returns 0 on success, -1 on error
 * The key is written to key_out
 */
int32_t yellow_getch(YellowScreen* screen, YellowKey* key_out);

/**
 * Set foreground color (RGB)
 * Returns 0 on success, -1 on error
 */
int32_t yellow_set_fg_color(YellowScreen* screen, uint8_t r, uint8_t g, uint8_t b);

/**
 * Set background color (RGB)
 * Returns 0 on success, -1 on error
 */
int32_t yellow_set_bg_color(YellowScreen* screen, uint8_t r, uint8_t g, uint8_t b);

/**
 * Turn on attribute
 * Use YELLOW_ATTR_* constants (can be OR'd together)
 * Returns 0 on success, -1 on error
 */
int32_t yellow_attron(YellowScreen* screen, uint32_t attr);

/**
 * Turn off attribute
 * Use YELLOW_ATTR_* constants (can be OR'd together)
 * Returns 0 on success, -1 on error
 */
int32_t yellow_attroff(YellowScreen* screen, uint32_t attr);

/**
 * Get terminal size
 * Returns (height << 16) | width, or 0 on error
 */
uint32_t yellow_get_size(YellowScreen* screen);

/**
 * Render mosaic (Unicode block art) from RGB image data
 *
 * @param data - Raw RGB pixel data (3 bytes per pixel)
 * @param data_len - Length of data array (should be width * height * 3)
 * @param width - Image width in pixels
 * @param height - Image height in pixels
 * @param output_width - Output width in terminal cells (0 = auto)
 * @param threshold - Luminance threshold 0-255 (default: 128)
 * @return Malloc'd string with Unicode art (must be freed with yellow_free_string)
 */
char* yellow_render_mosaic(
    const uint8_t* data,
    size_t data_len,
    size_t width,
    size_t height,
    size_t output_width,
    uint8_t threshold
);

/**
 * Free a string returned by yellow_render_mosaic
 */
void yellow_free_string(char* s);

#ifdef __cplusplus
}
#endif

#endif /* YELLOW_H */
