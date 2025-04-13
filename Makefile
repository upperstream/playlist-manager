.PHONY: help install uninstall test build clean

# Uncomment the following line to enable Windows compatibility
# EXE=.exe

WORKDIR=work
PREFIX=/usr/local
BINDIR=$(PREFIX)/bin
PLMBINDIR=$(PREFIX)/libexec/playlist-manager
MANDIR=$(PREFIX)/share/man
BUILDDIR=libexec/playlist-manager
EXECUTABLES=$(BUILDDIR)/plm-put-playlist$(EXE) $(BUILDDIR)/plm-delete-playlist$(EXE)
SRCFILES=src/bin/*.rs
BUILD_MARKER=$(WORKDIR)/.build_successful

help:
	@echo "make [-f Makefile] [-n] [PREFIX=...] [<target>]"
	@echo ""
	@echo "-f Makefile"
	@echo "   : Explicitly specify this makefile for 'make' to process"
	@echo "-n : Merely print what will be executed and quit"
	@echo "PREFIX=..."
	@echo "   : Specify destination directory tree to install this tool into"
	@echo ""
	@echo "Valid targets are:"
	@echo "help : Default target; print this help message and quit"
	@echo "build"
	@echo "     : Build the Rust binaries"
	@echo "install"
	@echo "     : Perform installation of this tool"
	@echo "uninstall"
	@echo "     : Perform uninstallation of this tool"
	@echo "test"
	@echo "     : Run integration tests"
	@echo "clean"
	@echo "     : Remove build artifacts"
	@echo ""
	@echo "'awk', 'make' and 'install' are required."

install: build
	mkdir -p $(BINDIR) $(PLMBINDIR) $(MANDIR)/man1
	awk -f embed_version.awk Cargo.toml bin/plm > $(BINDIR)/plm
	chmod +x $(BINDIR)/plm
	install libexec/playlist-manager/* $(PLMBINDIR)
	install man/man1/* $(MANDIR)/man1

build: $(EXECUTABLES)

$(BUILDDIR)/plm-put-playlist$(EXE): src/bin/plm-put-playlist.rs $(BUILD_MARKER)
$(BUILDDIR)/plm-delete-playlist$(EXE): src/bin/plm-delete-playlist.rs $(BUILD_MARKER)

$(BUILD_MARKER): $(SRCFILES)
	cargo build --release
	-mkdir -p libexec/playlist-manager && \
	cp target/release/plm-put-playlist$(EXE) target/release/plm-delete-playlist$(EXE) libexec/playlist-manager/ && \
	touch $(BUILD_MARKER)

uninstall:
	rm -rf $(BINDIR)/plm $(PLMBINDIR) $(MANDIR)/man1/plm.1 $(MANDIR)/man1/plm-*.1 $(MANDIR)/cat1/plm.1 $(MANDIR)/cat1/plm-*.1

test:
	cargo test

clean:
	cargo clean && \
	rm -f $(EXECUTABLES) $(BUILD_MARKER)
