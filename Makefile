BINARY = deskforge
PREFIX ?= $(HOME)/.local
BINDIR := $(PREFIX)/bin

.PHONY: all build install uninstall clean

all: build

check-cargo:
	@command -v cargo >/dev/null 2>&1 || { \
		echo "[ERROR]: Cargo is not installed or not in PATH."; \
		echo "Please install Rust and Cargo from https://www.rust-lang.org/tools/install"; \
		exit 1; \
	}

build: check-cargo
	cargo build --release

install: build
	@if [ ! -d "$(BINDIR)" ]; then \
		echo "[WARNING] $(BINDIR) does not exist, creating..."; \
		mkdir -p "$(BINDIR)"; \
	fi
	install -m 755 target/release/$(BINARY) $(BINDIR)/$(BINARY)

uninstall:
	rm -f $(BINDIR)/$(BINARY)

clean:
	cargo clean
