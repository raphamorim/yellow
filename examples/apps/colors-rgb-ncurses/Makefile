# Makefile for colors-rgb example

CC = gcc
CFLAGS = -Wall -Wextra -O3 -march=native -std=c99
LDFLAGS = -lm

TARGET = colors-rgb
SRC = colors-rgb.c

.PHONY: all clean run

all: $(TARGET)

$(TARGET): $(SRC)
	$(CC) $(CFLAGS) -o $(TARGET) $(SRC) $(LDFLAGS)

run: $(TARGET)
	./$(TARGET)

clean:
	rm -f $(TARGET)
