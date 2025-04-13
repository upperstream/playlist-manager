.PHONY: help install uninstall test build clean

PREFIX=/usr/local
BINDIR=$(PREFIX)/bin
PLMBINDIR=$(PREFIX)/libexec/playlist-manager
MANDIR=$(PREFIX)/share/man

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

build:
	cargo build --release
	-mkdir -p libexec/playlist-manager
	-if [ -f target/release/plm-put-playlist.exe ]; then \
		cp target/release/plm-put-playlist.exe libexec/playlist-manager/; \
	else \
		cp target/release/plm-put-playlist libexec/playlist-manager/; \
	fi
	-if [ -f target/release/plm-delete-playlist.exe ]; then \
		cp target/release/plm-delete-playlist.exe libexec/playlist-manager/; \
	else \
		cp target/release/plm-delete-playlist libexec/playlist-manager/; \
	fi

uninstall:
	rm -rf $(BINDIR)/plm $(PLMBINDIR) $(MANDIR)/man1/plm.1 $(MANDIR)/man1/plm-*.1 $(MANDIR)/cat1/plm.1 $(MANDIR)/cat1/plm-*.1

test:
	cargo test

clean:
	cargo clean
	-rm -f libexec/playlist-manager/plm-put-playlist libexec/playlist-manager/plm-put-playlist.exe
	-rm -f libexec/playlist-manager/plm-delete-playlist libexec/playlist-manager/plm-delete-playlist.exe
