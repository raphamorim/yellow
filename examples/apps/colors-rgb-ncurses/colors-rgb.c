/*
 * colors-rgb - A demonstration of RGB color rendering with raw terminal control
 *
 * This example shows the full range of RGB colors that can be displayed in the terminal.
 * Requires a terminal that supports 24-bit color (true color) and unicode.
 *
 * Features:
 * - RGB color rendering with true color support
 * - FPS calculation and display
 * - Using half-block characters for higher resolution color display
 * - Smooth horizontal scrolling animation
 *
 * Press 'q' or Ctrl+C to quit.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>
#include <locale.h>
#include <unistd.h>
#include <termios.h>
#include <sys/ioctl.h>
#include <sys/select.h>
#include <signal.h>

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

/* RGB color structure */
typedef struct {
    unsigned char r;
    unsigned char g;
    unsigned char b;
} RGB;

/* HSV color structure */
typedef struct {
    float h; /* hue: 0-360 */
    float s; /* saturation: 0-1 */
    float v; /* value: 0-1 */
} HSV;

/* FPS tracking */
typedef struct {
    int frame_count;
    struct timespec last_time;
    float fps;
} FpsWidget;

/* Colors widget */
typedef struct {
    RGB **colors;
    int width;
    int height;
    int frame_count;
} ColorsWidget;

/* Terminal state */
static struct termios orig_termios;
static int terminal_initialized = 0;

/* Cleanup handler */
void cleanup(void) {
    if (terminal_initialized) {
        /* Show cursor */
        printf("\033[?25h");
        /* Reset colors */
        printf("\033[0m");
        /* Clear screen */
        printf("\033[2J\033[H");
        /* Restore terminal */
        tcsetattr(STDIN_FILENO, TCSANOW, &orig_termios);
        fflush(stdout);
    }
}

void sigint_handler(int sig) {
    (void)sig;
    cleanup();
    exit(0);
}

/* Setup raw terminal mode */
void setup_terminal(void) {
    struct termios raw;

    /* Get current terminal settings */
    tcgetattr(STDIN_FILENO, &orig_termios);
    atexit(cleanup);
    signal(SIGINT, sigint_handler);

    /* Setup raw mode */
    raw = orig_termios;
    raw.c_lflag &= ~(ECHO | ICANON);
    raw.c_cc[VMIN] = 0;
    raw.c_cc[VTIME] = 0;
    tcsetattr(STDIN_FILENO, TCSANOW, &raw);

    /* Hide cursor */
    printf("\033[?25l");
    /* Clear screen */
    printf("\033[2J\033[H");
    /* Switch to alternate buffer */
    printf("\033[?1049h");
    fflush(stdout);

    terminal_initialized = 1;
}

/* Get terminal size */
void get_terminal_size(int *rows, int *cols) {
    struct winsize ws;
    ioctl(STDOUT_FILENO, TIOCGWINSZ, &ws);
    *rows = ws.ws_row;
    *cols = ws.ws_col;

    /* Fallback to reasonable defaults if detection fails */
    if (*rows == 0) *rows = 24;
    if (*cols == 0) *cols = 80;
}

/* Convert HSV to RGB */
RGB hsv_to_rgb(HSV hsv) {
    RGB rgb = {0, 0, 0};
    float h = hsv.h;
    float s = hsv.s;
    float v = hsv.v;

    if (s == 0.0f) {
        /* Achromatic (grey) */
        rgb.r = rgb.g = rgb.b = (unsigned char)(v * 255);
        return rgb;
    }

    h /= 60.0f; /* sector 0 to 5 */
    int i = (int)floor(h);
    float f = h - i; /* factorial part of h */
    float p = v * (1.0f - s);
    float q = v * (1.0f - s * f);
    float t = v * (1.0f - s * (1.0f - f));

    float r, g, b;
    switch (i % 6) {
        case 0: r = v; g = t; b = p; break;
        case 1: r = q; g = v; b = p; break;
        case 2: r = p; g = v; b = t; break;
        case 3: r = p; g = q; b = v; break;
        case 4: r = t; g = p; b = v; break;
        case 5: r = v; g = p; b = q; break;
        default: r = v; g = t; b = p; break;
    }

    rgb.r = (unsigned char)(r * 255);
    rgb.g = (unsigned char)(g * 255);
    rgb.b = (unsigned char)(b * 255);

    return rgb;
}

/* Initialize FPS widget */
void fps_init(FpsWidget *fps) {
    fps->frame_count = 0;
    clock_gettime(CLOCK_MONOTONIC, &fps->last_time);
    fps->fps = 0.0f;
}

/* Calculate FPS */
void fps_calculate(FpsWidget *fps) {
    fps->frame_count++;
    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);

    double elapsed = (now.tv_sec - fps->last_time.tv_sec) +
                     (now.tv_nsec - fps->last_time.tv_nsec) / 1000000000.0;

    if (elapsed >= 1.0 && fps->frame_count > 2) {
        fps->fps = (float)fps->frame_count / (float)elapsed;
        fps->frame_count = 0;
        fps->last_time = now;
    }
}

/* Initialize colors widget */
void colors_init(ColorsWidget *colors) {
    colors->colors = NULL;
    colors->width = 0;
    colors->height = 0;
    colors->frame_count = 0;
}

/* Free colors widget */
void colors_free(ColorsWidget *colors) {
    if (colors->colors) {
        for (int i = 0; i < colors->height; i++) {
            free(colors->colors[i]);
        }
        free(colors->colors);
        colors->colors = NULL;
    }
}

/* Setup colors */
void colors_setup(ColorsWidget *colors, int width, int height) {
    /* Double the height because each screen row has two rows of half block pixels */
    int pixel_height = height * 2;

    /* Only update if size changed */
    if (colors->colors && colors->width == width && colors->height == pixel_height) {
        return;
    }

    /* Free old colors */
    colors_free(colors);

    /* Allocate new colors */
    colors->width = width;
    colors->height = pixel_height;
    colors->colors = malloc(pixel_height * sizeof(RGB *));

    for (int y = 0; y < pixel_height; y++) {
        colors->colors[y] = malloc(width * sizeof(RGB));

        for (int x = 0; x < width; x++) {
            float hue = (float)x * 360.0f / (float)width;
            float value = (float)(pixel_height - y) / (float)pixel_height;
            float saturation = 1.0f; /* max saturation */

            HSV hsv = {hue, saturation, value};
            colors->colors[y][x] = hsv_to_rgb(hsv);
        }
    }
}

/* Fast integer to string conversion - returns pointer to end */
static inline char *fast_itoa(char *ptr, int val) {
    if (val == 0) {
        *ptr++ = '0';
        return ptr;
    }

    char buf[12];
    char *p = buf + 11;

    while (val > 0) {
        *--p = '0' + (val % 10);
        val /= 10;
    }

    while (p < buf + 11) {
        *ptr++ = *p++;
    }

    return ptr;
}

/* Move cursor to position (row, col) - 1-indexed */
void move_cursor(int row, int col) {
    printf("\033[%d;%dH", row, col);
}

/* Render colors widget - optimized with buffering and fast formatting */
void colors_render(ColorsWidget *colors, int start_row, int width) {
    int height = colors->height / 2; /* screen rows (each contains 2 pixel rows) */

    /* Pre-allocate buffer for entire frame (rough estimate) */
    static char *buffer = NULL;
    static size_t buffer_size = 0;
    size_t needed_size = height * width * 50; /* ~50 bytes per cell for escape codes */

    if (buffer_size < needed_size) {
        buffer = realloc(buffer, needed_size);
        buffer_size = needed_size;
    }

    char *ptr = buffer;

    for (int y = 0; y < height; y++) {
        /* Move cursor - manual formatting */
        *ptr++ = '\033';
        *ptr++ = '[';
        ptr = fast_itoa(ptr, start_row + y);
        *ptr++ = ';';
        *ptr++ = '1';
        *ptr++ = 'H';

        for (int x = 0; x < width; x++) {
            /* Animate by shifting x index */
            int xi = (x + colors->frame_count) % width;

            /* Get foreground (top pixel) and background (bottom pixel) colors */
            RGB fg = colors->colors[y * 2][xi];
            RGB bg = colors->colors[y * 2 + 1][xi];

            /* Build escape sequence manually: ESC[38;2;R;G;B;48;2;R;G;Bm */
            *ptr++ = '\033';
            *ptr++ = '[';
            *ptr++ = '3';
            *ptr++ = '8';
            *ptr++ = ';';
            *ptr++ = '2';
            *ptr++ = ';';
            ptr = fast_itoa(ptr, fg.r);
            *ptr++ = ';';
            ptr = fast_itoa(ptr, fg.g);
            *ptr++ = ';';
            ptr = fast_itoa(ptr, fg.b);
            *ptr++ = ';';
            *ptr++ = '4';
            *ptr++ = '8';
            *ptr++ = ';';
            *ptr++ = '2';
            *ptr++ = ';';
            ptr = fast_itoa(ptr, bg.r);
            *ptr++ = ';';
            ptr = fast_itoa(ptr, bg.g);
            *ptr++ = ';';
            ptr = fast_itoa(ptr, bg.b);
            *ptr++ = 'm';

            /* Add half block character (UTF-8: E2 96 80) */
            *ptr++ = '\xE2';
            *ptr++ = '\x96';
            *ptr++ = '\x80';
        }
    }

    /* Write entire frame in one call */
    write(STDOUT_FILENO, buffer, ptr - buffer);

    colors->frame_count++;
}

/* Check for keyboard input with timeout */
int check_input(void) {
    fd_set fds;
    struct timeval tv;

    FD_ZERO(&fds);
    FD_SET(STDIN_FILENO, &fds);

    tv.tv_sec = 0;
    tv.tv_usec = 16000; /* 16ms for ~60 FPS */

    int ret = select(STDIN_FILENO + 1, &fds, NULL, NULL, &tv);

    if (ret > 0) {
        char c;
        if (read(STDIN_FILENO, &c, 1) > 0) {
            return c;
        }
    }

    return 0;
}

int main(void) {
    /* Set locale for unicode support */
    setlocale(LC_ALL, "");

    /* Setup terminal */
    setup_terminal();

    /* Initialize widgets */
    FpsWidget fps;
    ColorsWidget colors;
    fps_init(&fps);
    colors_init(&colors);

    int running = 1;

    while (running) {
        int rows, cols;
        get_terminal_size(&rows, &cols);

        /* Build title bar in buffer */
        char title_bar[512];
        char *ptr = title_bar;

        /* Move to 1,1 and reset colors */
        *ptr++ = '\033';
        *ptr++ = '[';
        *ptr++ = '1';
        *ptr++ = ';';
        *ptr++ = '1';
        *ptr++ = 'H';
        *ptr++ = '\033';
        *ptr++ = '[';
        *ptr++ = '0';
        *ptr++ = 'm';

        /* Clear line */
        *ptr++ = '\033';
        *ptr++ = '[';
        *ptr++ = '2';
        *ptr++ = 'K';

        /* Render title (centered in left portion, leaving 8 chars for FPS) */
        const char *title = "colors_rgb example. Press q to quit";
        int title_len = 36; /* strlen(title) */
        int title_area_width = cols - 8;
        if (title_area_width < 0) title_area_width = 0;
        int title_x = (title_area_width / 2) - (title_len / 2);
        if (title_x < 1) title_x = 1;

        /* Add spacing before title */
        for (int i = 1; i < title_x; i++) *ptr++ = ' ';

        /* Copy title */
        const char *t = title;
        while (*t) *ptr++ = *t++;

        /* Render FPS on the right side */
        fps_calculate(&fps);
        if (fps.fps > 0.0f) {
            /* Format fps manually */
            int fps_int = (int)fps.fps;
            int fps_frac = (int)((fps.fps - fps_int) * 10);

            /* Calculate FPS text length: "XX.X fps" */
            int fps_text_len = 6; /* " fps" = 4, "X.X" = at least 3, total at least 7 */
            if (fps_int >= 100) fps_text_len = 8;
            else if (fps_int >= 10) fps_text_len = 7;

            int fps_x = cols - fps_text_len;
            if (fps_x < 1) fps_x = 1;

            /* Add spaces */
            int spaces_needed = fps_x - title_x - title_len;
            for (int i = 0; i < spaces_needed && i < 50; i++) *ptr++ = ' ';

            /* Write FPS */
            ptr = fast_itoa(ptr, fps_int);
            *ptr++ = '.';
            *ptr++ = '0' + fps_frac;
            *ptr++ = ' ';
            *ptr++ = 'f';
            *ptr++ = 'p';
            *ptr++ = 's';
        }

        /* Write title bar */
        write(STDOUT_FILENO, title_bar, ptr - title_bar);

        /* Setup and render colors widget */
        int colors_height = rows - 1;
        if (colors_height > 0) {
            colors_setup(&colors, cols, colors_height);
            colors_render(&colors, 2, cols);
        }

        /* Check for input */
        int ch = check_input();
        if (ch == 'q' || ch == 'Q' || ch == 3) { /* q or Ctrl+C */
            running = 0;
        }
    }

    /* Cleanup */
    colors_free(&colors);

    return 0;
}
