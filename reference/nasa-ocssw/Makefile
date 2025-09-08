# Makefile for NASA OCSSW QAA Reference Implementation

CC = gcc
CFLAGS = -Wall -O2 -std=c99
LIBS = -lm
TARGET = nasa_qaa_compare
SOURCE = nasa_qaa_compare.c

# Default target
all: $(TARGET)

# Build the executable
$(TARGET): $(SOURCE)
	$(CC) $(CFLAGS) -o $(TARGET) $(SOURCE) $(LIBS)

# Clean build artifacts
clean:
	rm -f $(TARGET)

# Run the program
run: $(TARGET)
	./$(TARGET)

# Install (copy to a bin directory if needed)
install: $(TARGET)
	cp $(TARGET) /usr/local/bin/

# Show help
help:
	@echo "Available targets:"
	@echo "  all     - Build the executable (default)"
	@echo "  clean   - Remove build artifacts"
	@echo "  run     - Build and run the program"
	@echo "  install - Install to /usr/local/bin"
	@echo "  help    - Show this help message"

.PHONY: all clean run install help
