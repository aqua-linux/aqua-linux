PREFIX ?= /usr
LIBDIR ?= $(PREFIX)/lib
INCLUDEDIR ?= $(PREFIX)/include
PKG_CONFIG ?= pkg-config

AQUA_CPPFLAGS = $(shell $(PKG_CONFIG) --cflags wireplumber-0.5)
AQUA_CFLAGS = -std=c11 -fPIC -Wall -Wextra -Werror
AQUA_LDLIBS = $(shell $(PKG_CONFIG) --libs wireplumber-0.5) -lm

.PHONY: all clean install install-devel install-runtime

all: libaqua-audio-native.so

libaqua-audio-native.so: aqua_audio_native.o
	$(CC) -shared $(LDFLAGS) -Wl,-soname,libaqua-audio-native.so.1 \
		-o $@ $^ $(AQUA_LDLIBS) $(LDLIBS)

aqua_audio_native.o: aqua_audio_native.c aqua_audio_native.h
	$(CC) $(CPPFLAGS) $(AQUA_CPPFLAGS) $(CFLAGS) $(AQUA_CFLAGS) -c -o $@ $<

install: install-devel

install-runtime: libaqua-audio-native.so
	install -D -m 0755 libaqua-audio-native.so \
		$(DESTDIR)$(LIBDIR)/libaqua-audio-native.so.1

install-devel: install-runtime
	ln -sf libaqua-audio-native.so.1 \
		$(DESTDIR)$(LIBDIR)/libaqua-audio-native.so
	install -D -m 0644 aqua_audio_native.h \
		$(DESTDIR)$(INCLUDEDIR)/aqua_audio_native.h

clean:
	rm -f aqua_audio_native.o libaqua-audio-native.so
