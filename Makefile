DESTDIR ?=
PREFIX ?= /usr/local
BINARIES = meow-zip meow-pkg

all:
	cargo build --release

clean:
	cargo clean
	for dir in $(BINARIES); do \
		cd $$dir && cargo clean && cd ..; \
	done

install:
	for bin in $(BINARIES); do \
		install -Dm755 target/release/$$bin $(DESTDIR)$(PREFIX)/bin/$$bin; \
	done
	install -Dm755 meow-craft $(DESTDIR)$(PREFIX)/bin/meow-craft

uninstall:
	for bin in $(BINARIES); do \
		rm -f $(DESTDIR)$(PREFIX)/bin/$$bin; \
	done
	rm -f $(DESTDIR)$(PREFIX)/meow-craft
