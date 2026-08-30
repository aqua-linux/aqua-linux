CPPFLAGS ?=
CFLAGS ?=
LDFLAGS ?=
LDLIBS ?=

AQUA_CFLAGS = -std=c11 -Wall -Wextra -Werror

.PHONY: all clean

all: aqua-audio-probe

aqua-audio-probe: aqua_audio_probe.o
	$(CC) $(LDFLAGS) -o $@ $^ -lasound -laqua-audio-native $(LDLIBS)

aqua_audio_probe.o: aqua_audio_probe.c
	$(CC) $(CPPFLAGS) $(CFLAGS) $(AQUA_CFLAGS) -c -o $@ $<

clean:
	rm -f aqua_audio_probe.o aqua-audio-probe
