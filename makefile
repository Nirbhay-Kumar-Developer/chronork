# Compiler and Flags
CXX      = g++
CXXFLAGS = -std=c++20 -Wall -O3 -Iinclude
LIBS     = 

# Target Binary Name
TARGET   = build/chronork

# Source and Header files
SRC      = src/*.cpp
HEADERS  = include/*.hpp

# Default target: compile the binary
all: $(TARGET)

# Link the binary
# We include HEADERS as a dependency so that if you change 
# a logic file, the binary is rebuilt.
$(TARGET): $(SRC) $(HEADERS)
	$(CXX) $(CXXFLAGS) $(SRC) -o $(TARGET) $(LIBS)

# Install the binary to Termux's local bin
install: $(TARGET)
	@echo "Installing $(TARGET) to $(PREFIX)/bin..."
	@cp $(TARGET) $(PREFIX)/bin/
	@chmod +x $(PREFIX)/bin/$(TARGET)
	@echo "Done. You can now run 'worklog' from anywhere."

# Remove the local binary
clean:
	@echo "Cleaning up..."
	@rm -f $(TARGET)

# Uninstall from Termux bin
uninstall:
	@echo "Removing $(TARGET) from $(PREFIX)/bin..."
	@rm -f $(PREFIX)/bin/$(TARGET)

.PHONY: all install