BINARY := $(shell grep -m 1 'name = ' Cargo.toml | cut -d '"' -f 2)
CARGO_HOME := $(or $(CARGO_HOME),$(HOME)/.cargo)
INSTALL_DIR := $(CARGO_HOME)/bin

all: build

test:
	cargo test

build: test
	cargo build --release

install: build
	install -m 0755 target/release/$(BINARY) $(INSTALL_DIR)/$(BINARY)

uninstall: 
	rm -f $(INSTALL_DIR)/$(BINARY)

clean: 
	cargo clean

.PHONY: all build install uninstall clean
