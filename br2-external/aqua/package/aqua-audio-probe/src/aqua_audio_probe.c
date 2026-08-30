#include <alsa/asoundlib.h>
#include <aqua_audio_native.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum {
  SAMPLE_RATE = 48000,
  CHANNELS = 2,
  PERIOD_FRAMES = 480,
  PLAYBACK_PERIODS = 100,
  CAPTURE_PERIODS = 10,
};

static struct aqua_audio_native_node *default_output(
    struct aqua_audio_native_snapshot *snapshot) {
  for (uint32_t index = 0; index < snapshot->node_count; ++index) {
    struct aqua_audio_native_node *node = &snapshot->nodes[index];
    if (node->kind == AQUA_AUDIO_NATIVE_OUTPUT &&
        strcmp(node->name, snapshot->default_output) == 0) {
      return node;
    }
  }
  return NULL;
}

static int native_failure(struct aqua_audio_native *handle,
    const char *operation, int32_t status) {
  fprintf(stderr,
          "[AQUA-AUDIO] stage=control-probe status=failed "
          "operation=%s code=%d detail=%s\n",
          operation, status, aqua_audio_native_last_error(handle));
  return 1;
}

static int controls(void) {
  struct aqua_audio_native *handle = NULL;
  int32_t status = aqua_audio_native_open(5000, &handle);
  if (status != AQUA_AUDIO_NATIVE_OK) {
    int result = native_failure(handle, "open", status);
    aqua_audio_native_close(handle);
    return result;
  }

  struct aqua_audio_native_snapshot snapshot;
  status = aqua_audio_native_snapshot(handle, 5000, &snapshot);
  if (status != AQUA_AUDIO_NATIVE_OK) {
    int result = native_failure(handle, "initial-snapshot", status);
    aqua_audio_native_close(handle);
    return result;
  }
  struct aqua_audio_native_node *output = default_output(&snapshot);
  if (!output) {
    int result = native_failure(handle, "default-output",
        AQUA_AUDIO_NATIVE_NODE_NOT_FOUND);
    aqua_audio_native_close(handle);
    return result;
  }
  char output_name[AQUA_AUDIO_NATIVE_NODE_NAME_BYTES];
  memcpy(output_name, output->name, sizeof(output_name));

  status = aqua_audio_native_set_output_volume(handle, output_name, 35, 5000);
  if (status == AQUA_AUDIO_NATIVE_OK) {
    status = aqua_audio_native_set_output_muted(handle, output_name, 1, 5000);
  }
  if (status == AQUA_AUDIO_NATIVE_OK) {
    status = aqua_audio_native_snapshot(handle, 5000, &snapshot);
  }
  output = status == AQUA_AUDIO_NATIVE_OK ? default_output(&snapshot) : NULL;
  if (status != AQUA_AUDIO_NATIVE_OK || !output ||
      output->volume_percent < 34 || output->volume_percent > 36 ||
      output->muted != 1) {
    if (status == AQUA_AUDIO_NATIVE_OK) {
      status = AQUA_AUDIO_NATIVE_API_FAILED;
    }
    int result = native_failure(handle, "muted-snapshot", status);
    aqua_audio_native_close(handle);
    return result;
  }

  status = aqua_audio_native_set_output_muted(handle, output_name, 0, 5000);
  if (status == AQUA_AUDIO_NATIVE_OK) {
    status = aqua_audio_native_snapshot(handle, 5000, &snapshot);
  }
  output = status == AQUA_AUDIO_NATIVE_OK ? default_output(&snapshot) : NULL;
  if (status != AQUA_AUDIO_NATIVE_OK || !output || output->muted != 0) {
    if (status == AQUA_AUDIO_NATIVE_OK) {
      status = AQUA_AUDIO_NATIVE_API_FAILED;
    }
    int result = native_failure(handle, "unmuted-snapshot", status);
    aqua_audio_native_close(handle);
    return result;
  }

  printf("[AQUA-AUDIO] stage=control-probe status=ok "
         "backend=aqua-audio-native default_sink=true volume=35 "
         "mute_cycle=true\n");
  aqua_audio_native_close(handle);
  return 0;
}

static int configure(snd_pcm_t *pcm, snd_pcm_stream_t stream) {
  int status = snd_pcm_set_params(
      pcm, SND_PCM_FORMAT_S16_LE, SND_PCM_ACCESS_RW_INTERLEAVED, CHANNELS,
      SAMPLE_RATE, 1, stream == SND_PCM_STREAM_PLAYBACK ? 100000 : 200000);
  if (status < 0) {
    fprintf(stderr, "[AQUA-AUDIO] stage=media-probe status=failed "
                    "reason=configure detail=%s\n",
            snd_strerror(status));
    return 1;
  }
  return 0;
}

static int recover_io(snd_pcm_t *pcm, int status) {
  status = snd_pcm_recover(pcm, status, 1);
  if (status < 0) {
    fprintf(stderr, "[AQUA-AUDIO] stage=media-probe status=failed "
                    "reason=pcm-io detail=%s\n",
            snd_strerror(status));
    return 1;
  }
  return 0;
}

static int playback(void) {
  snd_pcm_t *pcm = NULL;
  int status = snd_pcm_open(&pcm, "default", SND_PCM_STREAM_PLAYBACK, 0);
  if (status < 0) {
    fprintf(stderr, "[AQUA-AUDIO] stage=media-probe status=failed "
                    "reason=open-playback detail=%s\n",
            snd_strerror(status));
    return 1;
  }
  if (configure(pcm, SND_PCM_STREAM_PLAYBACK) != 0) {
    snd_pcm_close(pcm);
    return 1;
  }

  int16_t samples[PERIOD_FRAMES * CHANNELS];
  uint32_t phase = 0;
  uint64_t frames = 0;
  for (unsigned int period = 0; period < PLAYBACK_PERIODS; ++period) {
    for (unsigned int frame = 0; frame < PERIOD_FRAMES; ++frame) {
      int16_t sample = phase < 24000 ? 8192 : -8192;
      phase = (phase + 1000) % SAMPLE_RATE;
      samples[frame * CHANNELS] = sample;
      samples[frame * CHANNELS + 1] = sample;
    }
    snd_pcm_sframes_t written = snd_pcm_writei(pcm, samples, PERIOD_FRAMES);
    if (written < 0) {
      if (recover_io(pcm, (int)written) != 0) {
        snd_pcm_close(pcm);
        return 1;
      }
      --period;
      continue;
    }
    frames += (uint64_t)written;
  }
  status = snd_pcm_drain(pcm);
  snd_pcm_close(pcm);
  if (status < 0 || frames != (uint64_t)PERIOD_FRAMES * PLAYBACK_PERIODS) {
    fprintf(stderr, "[AQUA-AUDIO] stage=media-probe status=failed "
                    "reason=incomplete-playback frames=%llu\n",
            (unsigned long long)frames);
    return 1;
  }
  printf("[AQUA-AUDIO] stage=media-probe status=ok direction=playback "
         "frames=%llu rate=%d channels=%d format=s16le\n",
         (unsigned long long)frames, SAMPLE_RATE, CHANNELS);
  return 0;
}

static int capture(int require_silence) {
  snd_pcm_t *pcm = NULL;
  int status = snd_pcm_open(&pcm, "default", SND_PCM_STREAM_CAPTURE, 0);
  if (status < 0) {
    fprintf(stderr, "[AQUA-AUDIO] stage=media-probe status=failed "
                    "reason=open-capture detail=%s\n",
            snd_strerror(status));
    return 1;
  }
  if (configure(pcm, SND_PCM_STREAM_CAPTURE) != 0) {
    snd_pcm_close(pcm);
    return 1;
  }
  status = snd_pcm_start(pcm);
  if (status < 0) {
    fprintf(stderr, "[AQUA-AUDIO] stage=media-probe status=failed "
                    "reason=start-capture detail=%s\n",
            snd_strerror(status));
    snd_pcm_close(pcm);
    return 1;
  }

  int16_t samples[PERIOD_FRAMES * CHANNELS];
  uint64_t frames = 0;
  uint32_t peak_abs = 0;
  for (unsigned int period = 0; period < CAPTURE_PERIODS; ++period) {
    status = snd_pcm_wait(pcm, 5000);
    if (status <= 0) {
      fprintf(stderr, "[AQUA-AUDIO] stage=media-probe status=failed "
                      "reason=capture-timeout\n");
      snd_pcm_close(pcm);
      return 1;
    }
    snd_pcm_sframes_t read_frames = snd_pcm_readi(pcm, samples, PERIOD_FRAMES);
    if (read_frames < 0) {
      if (recover_io(pcm, (int)read_frames) != 0) {
        snd_pcm_close(pcm);
        return 1;
      }
      --period;
      continue;
    }
    for (snd_pcm_sframes_t frame = 0; frame < read_frames; ++frame) {
      for (unsigned int channel = 0; channel < CHANNELS; ++channel) {
        int32_t sample = samples[frame * CHANNELS + channel];
        uint32_t magnitude = (uint32_t)(sample < 0 ? -sample : sample);
        if (magnitude > peak_abs) {
          peak_abs = magnitude;
        }
      }
    }
    frames += (uint64_t)read_frames;
  }
  snd_pcm_drop(pcm);
  snd_pcm_close(pcm);
  if (frames != (uint64_t)PERIOD_FRAMES * CAPTURE_PERIODS) {
    fprintf(stderr, "[AQUA-AUDIO] stage=media-probe status=failed "
                    "reason=incomplete-capture frames=%llu\n",
            (unsigned long long)frames);
    return 1;
  }
  if (require_silence && peak_abs != 0) {
    fprintf(stderr, "[AQUA-AUDIO] stage=media-probe status=failed "
                    "reason=unexpected-capture-data peak_abs=%u\n",
            peak_abs);
    return 1;
  }
  printf("[AQUA-AUDIO] stage=media-probe status=ok direction=capture "
         "frames=%llu rate=%d channels=%d format=s16le peak_abs=%u "
         "pattern=%s\n",
         (unsigned long long)frames, SAMPLE_RATE, CHANNELS, peak_abs,
         require_silence ? "silence" : "observed");
  return 0;
}

int main(int argc, char **argv) {
  if (argc != 2) {
    fprintf(stderr,
            "usage: aqua-audio-probe playback|capture|capture-silence|controls\n");
    return 2;
  }
  if (strcmp(argv[1], "playback") == 0) {
    return playback();
  }
  if (strcmp(argv[1], "capture") == 0) {
    return capture(0);
  }
  if (strcmp(argv[1], "capture-silence") == 0) {
    return capture(1);
  }
  if (strcmp(argv[1], "controls") == 0) {
    return controls();
  }
  fprintf(stderr,
          "usage: aqua-audio-probe playback|capture|capture-silence|controls\n");
  return 2;
}
