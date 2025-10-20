.PHONY: run build clean

# Build flags
LDFLAGS := -s -w
FLAGS := -trimpath

# Binary name
BINARY := colors-rgb-ultraviolet

run: build
	./$(BINARY)

build:
	go build -ldflags="$(LDFLAGS)" $(FLAGS) -o $(BINARY) .

clean:
	rm -f $(BINARY)
