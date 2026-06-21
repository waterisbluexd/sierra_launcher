PREFIX ?= /usr/local
BINARY = target/release/sierra_launcher

build:
	cargo build --release

install: build
	install -Dm755 $(BINARY) $(DESTDIR)$(PREFIX)/bin/sierra_launcher
	install -Dm644 ui/main_card.slint $(DESTDIR)$(PREFIX)/share/sierra_launcher/ui/main_card.slint
	install -Dm644 ui/theme.slint $(DESTDIR)$(PREFIX)/share/sierra_launcher/ui/theme.slint

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/sierra_launcher
	rm -rf $(DESTDIR)$(PREFIX)/share/sierra_launcher
