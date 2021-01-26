OS = $(shell uname -s)
KRUNVM_RELEASE = target/release/krunvm
KRUNVM_DEBUG = target/debug/krunvm
INIT_BINARY = init/init

ifeq ($(PREFIX),)
    PREFIX := /usr/local
endif

.PHONY: install clean

all: $(KRUNVM_RELEASE)

debug: $(KRUNVM_DEBUG)

$(KRUNVM_RELEASE):
	cargo build --release
ifeq ($(OS),Darwin)
	codesign --entitlements krunvm.entitlements --force -s - $@
endif

$(KRUNVM_DEBUG):
	cargo build --debug

install: $(KRUNVM_RELEASE)
	install -d $(DESTDIR)$(PREFIX)/bin
	install -m 755 $(KRUNVM_RELEASE) $(DESTDIR)$(PREFIX)/bin

clean:
	cargo clean
