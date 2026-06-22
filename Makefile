PREFIX ?= /usr/local
BINARY = target/release/sierra_launcher

build:
	cargo build --release

install: build
	sudo install -Dm755 $(BINARY) $(DESTDIR)$(PREFIX)/bin/sierra_launcher
	sudo install -Dm644 ui/main_card.slint $(DESTDIR)$(PREFIX)/share/sierra_launcher/ui/main_card.slint
	sudo install -Dm644 ui/theme.slint $(DESTDIR)$(PREFIX)/share/sierra_launcher/ui/theme.slint

uninstall:
	sudo rm -f $(DESTDIR)$(PREFIX)/bin/sierra_launcher
	sudo rm -rf $(DESTDIR)$(PREFIX)/share/sierra_launcher
