/* Zaz Terminal Library - C Bindings
 * Auto-maintained header for FFI
 */

#ifndef ZAZ_H
#define ZAZ_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque screen handle */
typedef struct ZazScreen ZazScreen;

/* Key codes - C representation of Rust enum */
typedef enum {
    ZazKey_Char = 0,
    ZazKey_ArrowUp,
    ZazKey_ArrowDown,
    ZazKey_ArrowLeft,
    ZazKey_ArrowRight,
    ZazKey_Enter,
    ZazKey_Backspace,
    ZazKey_Delete,
    ZazKey_Home,
    ZazKey_End,
    ZazKey_PageUp,
    ZazKey_PageDown,
    ZazKey_Tab,
    ZazKey_Escape,
    ZazKey_F1,
    ZazKey_F2,
    ZazKey_F3,
    ZazKey_F4,
    ZazKey_F5,
    ZazKey_F6,
    ZazKey_F7,
    ZazKey_F8,
    ZazKey_F9,
    ZazKey_F10,
    ZazKey_F11,
    ZazKey_F12,
    ZazKey_Unknown,
} ZazKeyTag;

/* Tagged union for key events */
typedef struct {
    ZazKeyTag tag;
    union {
        uint32_t char_value; /* For ZazKey_Char */
    } value;
} ZazKey;

/* Attribute constants */
#define ZAZ_ATTR_BOLD           1
#define ZAZ_ATTR_DIM            2
#define ZAZ_ATTR_ITALIC         4
#define ZAZ_ATTR_UNDERLINE      8
#define ZAZ_ATTR_BLINK          16
#define ZAZ_ATTR_REVERSE        32
#define ZAZ_ATTR_HIDDEN         64
#define ZAZ_ATTR_STRIKETHROUGH  128

/* Screen management */

/**
 * Initialize a new screen
 * Returns NULL on error
 */
ZazScreen* zaz_init(void);

/**
 * Clean up and restore terminal
 * Returns 0 on success, -1 on error
 */
int32_t zaz_endwin(ZazScreen* screen);

/**
 * Clear the screen
 * Returns 0 on success, -1 on error
 */
int32_t zaz_clear(ZazScreen* screen);

/**
 * Refresh the screen (flush output)
 * Returns 0 on success, -1 on error
 */
int32_t zaz_refresh(ZazScreen* screen);

/**
 * Move cursor to position (y, x)
 * Returns 0 on success, -1 on error
 */
int32_t zaz_move_cursor(ZazScreen* screen, uint16_t y, uint16_t x);

/**
 * Print string at current cursor position
 * Returns 0 on success, -1 on error
 */
int32_t zaz_print(ZazScreen* screen, const char* text);

/**
 * Print string at position (y, x)
 * Returns 0 on success, -1 on error
 */
int32_t zaz_mvprint(ZazScreen* screen, uint16_t y, uint16_t x, const char* text);

/**
 * Get a key from input
 * Returns 0 on success, -1 on error
 * The key is written to key_out
 */
int32_t zaz_getch(ZazScreen* screen, ZazKey* key_out);

/**
 * Set foreground color (RGB)
 * Returns 0 on success, -1 on error
 */
int32_t zaz_set_fg_color(ZazScreen* screen, uint8_t r, uint8_t g, uint8_t b);

/**
 * Set background color (RGB)
 * Returns 0 on success, -1 on error
 */
int32_t zaz_set_bg_color(ZazScreen* screen, uint8_t r, uint8_t g, uint8_t b);

/**
 * Turn on attribute
 * Use ZAZ_ATTR_* constants (can be OR'd together)
 * Returns 0 on success, -1 on error
 */
int32_t zaz_attron(ZazScreen* screen, uint32_t attr);

/**
 * Turn off attribute
 * Use ZAZ_ATTR_* constants (can be OR'd together)
 * Returns 0 on success, -1 on error
 */
int32_t zaz_attroff(ZazScreen* screen, uint32_t attr);

/**
 * Get terminal size
 * Returns (height << 16) | width, or 0 on error
 */
uint32_t zaz_get_size(ZazScreen* screen);

/**
 * Render mosaic (Unicode block art) from RGB image data
 *
 * @param data - Raw RGB pixel data (3 bytes per pixel)
 * @param data_len - Length of data array (should be width * height * 3)
 * @param width - Image width in pixels
 * @param height - Image height in pixels
 * @param output_width - Output width in terminal cells (0 = auto)
 * @param threshold - Luminance threshold 0-255 (default: 128)
 * @return Malloc'd string with Unicode art (must be freed with zaz_free_string)
 */
char* zaz_render_mosaic(
    const uint8_t* data,
    size_t data_len,
    size_t width,
    size_t height,
    size_t output_width,
    uint8_t threshold
);

/**
 * Free a string returned by zaz_render_mosaic
 */
void zaz_free_string(char* s);

#ifdef __cplusplus
}
#endif

#endif /* ZAZ_H */
