.PHONY: help install

PREFIX=/usr/local
BINDIR=$(PREFIX)/bin
PLMBINDIR=$(PREFIX)/libexec/playlist-manager
MANDIR=$(PREFIX)/share/man

help:
	@echo "make [-f Makefile] [-n] [PREFIX=...] [help | install]"
	@echo ""
	@echo "-f Makefile"
	@echo "   : explicitly specify this makefile for 'make' to process"
	@echo "-n : merely prints what will be executed and quits"
	@echo "PREFIX=..."
	@echo "   : specify destination directory tree to install this tool into"
	@echo "help"
	@echo "   : prints this help message and quits"
	@echo "install"
	@echo "   : performs installation of this tool"
	@echo ""
	@echo "make and install are required."

install:
	mkdir -p $(BINDIR) $(PLMBINDIR) $(MANDIR)/man1
	install bin/plm $(BINDIR)
	install libexec/playlist-manager/* $(PLMBINDIR)
	install man/man1/* $(MANDIR)/man1
