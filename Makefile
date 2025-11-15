BINARY = deskforge
PREFIX ?= $(HOME)/.local
BINDIR := $(PREFIX)/bin

.PHONY: all build install uninstall clean

all: build

build:
	cargo build --release

install: build
	@if [ ! -d "$(BINDIR)" ]; then \
		echo "[WARNING] $(BINDIR) does not exist, creating..."; \
		mkdir -p "$(BINDIR)"; \
	fi
	install -m 755 target/release/$(BINARY) $(BINDIR)/$(BINARY)

uninstall:
	rm -f $(BINDIR)/$(BINARY)