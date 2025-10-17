#ifndef IMAGE_LOADER_H
#define IMAGE_LOADER_H

#ifdef __cplusplus
extern "C" {
#endif

unsigned char* load_image(const char* filename, int* width, int* height, int* channels);
void free_image(unsigned char* data);

#ifdef __cplusplus
}
#endif

#endif
