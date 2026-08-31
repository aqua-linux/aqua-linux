PKG_CONFIG ?= pkg-config

AQUA_CPPFLAGS = $(shell $(PKG_CONFIG) --cflags glfw3 wayland-client)
AQUA_CFLAGS = -std=c11 -Wall -Wextra -Werror
AQUA_LDLIBS = $(shell $(PKG_CONFIG) --libs glfw3 wayland-client)

.PHONY: all clean

all: aqua-glfw-wayland-probe

aqua-glfw-wayland-probe: aqua_glfw_wayland_probe.c
	$(CC) $(CPPFLAGS) $(AQUA_CPPFLAGS) $(CFLAGS) $(AQUA_CFLAGS) \
		$(LDFLAGS) -o $@ $< $(AQUA_LDLIBS) $(LDLIBS)

clean:
	rm -f aqua-glfw-wayland-probe
