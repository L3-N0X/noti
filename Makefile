PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
DATADIR = $(PREFIX)/share
APPDIR = $(DATADIR)/applications
ICONDIR = $(DATADIR)/icons/hicolor/scalable/apps
METAINFODIR = $(DATADIR)/metainfo

APP_ID = io.github.L3-N0X.noti
BINARY = target/release/noti

.PHONY: all build install uninstall clean

all: build

build:
	cargo build --release --locked

install:
	# Build if not already built
	@if [ ! -f $(BINARY) ]; then \
		cargo build --release --locked; \
	fi
	
	# Create directories
	install -d "$(DESTDIR)$(BINDIR)"
	install -d "$(DESTDIR)$(APPDIR)"
	install -d "$(DESTDIR)$(ICONDIR)"
	install -d "$(DESTDIR)$(METAINFODIR)"
	
	# Install binary
	install -m 755 "$(BINARY)" "$(DESTDIR)$(BINDIR)/noti"
	
	# Install assets
	install -m 644 "resources/$(APP_ID).desktop" "$(DESTDIR)$(APPDIR)/$(APP_ID).desktop"
	sed -i 's|^Exec=noti|Exec=$(BINDIR)/noti|' "$(DESTDIR)$(APPDIR)/$(APP_ID).desktop"
	install -m 644 "resources/$(APP_ID).svg" "$(DESTDIR)$(ICONDIR)/$(APP_ID).svg"
	install -m 644 "resources/$(APP_ID).metainfo.xml" "$(DESTDIR)$(METAINFODIR)/$(APP_ID).metainfo.xml"
	
	# Update caches if possible
	@if [ -z "$(DESTDIR)" ]; then \
		echo "Updating desktop and icon caches..."; \
		update-desktop-database -q "$(APPDIR)" || true; \
		gtk-update-icon-cache -q -t -f "$(DATADIR)/icons/hicolor" || true; \
	fi

uninstall:
	rm -f "$(DESTDIR)$(BINDIR)/noti"
	rm -f "$(DESTDIR)$(APPDIR)/$(APP_ID).desktop"
	rm -f "$(DESTDIR)$(ICONDIR)/$(APP_ID).svg"
	rm -f "$(DESTDIR)$(METAINFODIR)/$(APP_ID).metainfo.xml"
	
	@if [ -z "$(DESTDIR)" ]; then \
		echo "Updating desktop and icon caches..."; \
		update-desktop-database -q "$(APPDIR)" || true; \
		gtk-update-icon-cache -q -t -f "$(DATADIR)/icons/hicolor" || true; \
	fi

clean:
	cargo clean
