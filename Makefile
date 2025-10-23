prefix ?= /usr
exec_prefix ?= $(prefix)
bindir ?= $(exec_prefix)/bin

DESTDIR ?=
CARGO ?= cargo

.PHONY: all build test clean install uninstall help

all: build

build:
	$(CARGO) build --release

test:
	$(CARGO) test

install:
	@mkdir -p "$(DESTDIR)$(bindir)"
	install -m 755 target/release/meow "$(DESTDIR)$(bindir)/"

uninstall:
	rm -f "$(DESTDIR)$(bindir)/meow"

clean:
	$(CARGO) clean

help:
	@echo "Targets: all build release test install uninstall clean"
	@echo "Variables you can override:"
	@echo "  make prefix=/usr DESTDIR=/tmp/stage install"
