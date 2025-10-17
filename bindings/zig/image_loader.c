#define STB_IMAGE_IMPLEMENTATION
#include "stb_image.h"

// Simple wrapper functions for Zig to call
unsigned char* load_image(const char* filename, int* width, int* height, int* channels) {
    return stbi_load(filename, width, height, channels, 3);
}

void free_image(unsigned char* data) {
    stbi_image_free(data);
}
