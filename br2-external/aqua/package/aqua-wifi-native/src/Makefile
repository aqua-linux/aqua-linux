PREFIX ?= /usr
LIBDIR ?= $(PREFIX)/lib
INCLUDEDIR ?= $(PREFIX)/include

AQUA_CFLAGS = -std=c11 -fPIC -Wall -Wextra -Werror
AQUA_LDLIBS = -lwpa_client -lcrypto

.PHONY: all clean install install-devel install-runtime

all: libaqua-wifi-native.so

libaqua-wifi-native.so: aqua_wifi_native.o
	$(CC) -shared $(LDFLAGS) -Wl,-soname,libaqua-wifi-native.so.1 \
		-o $@ $^ $(AQUA_LDLIBS) $(LDLIBS)

aqua_wifi_native.o: aqua_wifi_native.c aqua_wifi_native.h
	$(CC) $(CPPFLAGS) $(CFLAGS) $(AQUA_CFLAGS) -c -o $@ $<

install: install-devel

install-runtime: libaqua-wifi-native.so
	install -D -m 0755 libaqua-wifi-native.so \
		$(DESTDIR)$(LIBDIR)/libaqua-wifi-native.so.1

install-devel: install-runtime
	ln -sf libaqua-wifi-native.so.1 \
		$(DESTDIR)$(LIBDIR)/libaqua-wifi-native.so
	install -D -m 0644 aqua_wifi_native.h \
		$(DESTDIR)$(INCLUDEDIR)/aqua_wifi_native.h

clean:
	rm -f aqua_wifi_native.o libaqua-wifi-native.so
