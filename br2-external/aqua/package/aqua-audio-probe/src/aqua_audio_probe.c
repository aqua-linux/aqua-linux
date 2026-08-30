#include <alsa/asoundlib.h>
#include <aqua_audio_native.h>
#include <poll.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum {
  SAMPLE_RATE = 48000,
  CHANNELS = 2,
  PERIOD_FRAMES = 480,
  PLAYBACK_PERIODS = 100,
  INTERRUPTED_PLAYBACK_PERIODS = 500,
  CAPTURE_PERIODS = 10,
  INTERRUPTED_CAPTURE_PERIODS = 500,
};

enum capture_requirement {
  CAPTURE_OBSERVE,
  CAPTURE_REQUIRE_SILENCE,
  CAPTURE_REQUIRE_SIGNAL,
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

static int route_failure(struct aqua_audio_native *handle,
    const char *operation, int32_t status) {
  fprintf(stderr,
          "[AQUA-AUDIO] stage=route-probe status=failed "
          "operation=%s code=%d detail=%s\n",
          operation, status, aqua_audio_native_last_error(handle));
  return 1;
}

static int hotplug_failure(struct aqua_audio_native *handle,
    const char *operation, int32_t status) {
  fprintf(stderr,
          "[AQUA-AUDIO] stage=hotplug-probe status=failed "
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

static uint32_t output_count(
    const struct aqua_audio_native_snapshot *snapshot) {
  uint32_t count = 0;
  for (uint32_t index = 0; index < snapshot->node_count; ++index) {
    if (snapshot->nodes[index].kind == AQUA_AUDIO_NATIVE_OUTPUT) {
      ++count;
    }
  }
  return count;
}

static int routes(const char *required_node_fragment) {
  struct aqua_audio_native *handle = NULL;
  int32_t status = aqua_audio_native_open(5000, &handle);
  if (status != AQUA_AUDIO_NATIVE_OK) {
    int result = route_failure(handle, "open", status);
    aqua_audio_native_close(handle);
    return result;
  }

  struct aqua_audio_native_snapshot snapshot;
  status = aqua_audio_native_snapshot(handle, 5000, &snapshot);
  if (status != AQUA_AUDIO_NATIVE_OK) {
    int result = route_failure(handle, "initial-snapshot", status);
    aqua_audio_native_close(handle);
    return result;
  }
  if (!default_output(&snapshot) || output_count(&snapshot) != 2) {
    int result = route_failure(handle, "two-output-topology",
        AQUA_AUDIO_NATIVE_NODE_NOT_FOUND);
    aqua_audio_native_close(handle);
    return result;
  }

  char previous[AQUA_AUDIO_NATIVE_NODE_NAME_BYTES];
  memcpy(previous, snapshot.default_output, sizeof(previous));
  char requested[AQUA_AUDIO_NATIVE_NODE_NAME_BYTES] = {0};
  for (uint32_t index = 0; index < snapshot.node_count; ++index) {
    const struct aqua_audio_native_node *node = &snapshot.nodes[index];
    if (node->kind == AQUA_AUDIO_NATIVE_OUTPUT &&
        strcmp(node->name, previous) != 0 &&
        (!required_node_fragment ||
         strstr(node->name, required_node_fragment) != NULL)) {
      memcpy(requested, node->name, sizeof(requested));
      break;
    }
  }
  if (requested[0] == '\0') {
    int result = route_failure(handle, "alternate-output",
        AQUA_AUDIO_NATIVE_NODE_NOT_FOUND);
    aqua_audio_native_close(handle);
    return result;
  }

  status = aqua_audio_native_set_configured_default_output(
      handle, requested, 5000);
  if (status != AQUA_AUDIO_NATIVE_OK) {
    int result = route_failure(handle, "set-default-output", status);
    aqua_audio_native_close(handle);
    return result;
  }

  int acknowledged = 0;
  for (unsigned int attempt = 0; attempt < 50; ++attempt) {
    status = aqua_audio_native_snapshot(handle, 5000, &snapshot);
    if (status != AQUA_AUDIO_NATIVE_OK) {
      break;
    }
    if (strcmp(snapshot.default_output, requested) == 0) {
      acknowledged = 1;
      break;
    }
    poll(NULL, 0, 100);
  }
  if (status != AQUA_AUDIO_NATIVE_OK || !acknowledged ||
      output_count(&snapshot) != 2 ||
      strcmp(snapshot.default_output, previous) == 0) {
    if (status == AQUA_AUDIO_NATIVE_OK) {
      status = AQUA_AUDIO_NATIVE_API_FAILED;
    }
    int result = route_failure(handle, "acknowledged-default", status);
    aqua_audio_native_close(handle);
    return result;
  }

  printf("[AQUA-AUDIO] stage=route-probe status=ok outputs=2 "
         "previous_default=true requested_node=true default_changed=true%s\n",
         required_node_fragment ? " requested_slot=05.0" : "");
  aqua_audio_native_close(handle);
  return 0;
}

static int fallback(void) {
  struct aqua_audio_native *handle = NULL;
  int32_t status = aqua_audio_native_open(5000, &handle);
  if (status != AQUA_AUDIO_NATIVE_OK) {
    int result = hotplug_failure(handle, "open", status);
    aqua_audio_native_close(handle);
    return result;
  }

  struct aqua_audio_native_snapshot snapshot;
  int acknowledged = 0;
  for (unsigned int attempt = 0; attempt < 50; ++attempt) {
    status = aqua_audio_native_snapshot(handle, 5000, &snapshot);
    if (status != AQUA_AUDIO_NATIVE_OK) {
      break;
    }
    if (output_count(&snapshot) == 1 && default_output(&snapshot)) {
      acknowledged = 1;
      break;
    }
    poll(NULL, 0, 100);
  }
  if (status != AQUA_AUDIO_NATIVE_OK || !acknowledged) {
    if (status == AQUA_AUDIO_NATIVE_OK) {
      status = AQUA_AUDIO_NATIVE_NODE_NOT_FOUND;
    }
    int result = hotplug_failure(handle, "remaining-default", status);
    aqua_audio_native_close(handle);
    return result;
  }

  printf("[AQUA-AUDIO] stage=hotplug-probe status=ok outputs=1 "
         "default_output=true graph_ready=true\n");
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

static int playback(unsigned int required_periods, int expect_interruption) {
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
  for (unsigned int period = 0; period < required_periods; ++period) {
    for (unsigned int frame = 0; frame < PERIOD_FRAMES; ++frame) {
      int16_t sample = phase < 24000 ? 8192 : -8192;
      phase = (phase + 1000) % SAMPLE_RATE;
      samples[frame * CHANNELS] = sample;
      samples[frame * CHANNELS + 1] = sample;
    }
    snd_pcm_sframes_t written = snd_pcm_writei(pcm, samples, PERIOD_FRAMES);
    if (written < 0) {
      if (expect_interruption) {
        fprintf(stderr,
                "[AQUA-AUDIO] stage=media-probe status=interrupted "
                "direction=playback reason=pcm-io frames=%llu detail=%s\n",
                (unsigned long long)frames, snd_strerror((int)written));
        snd_pcm_close(pcm);
        return 3;
      }
      if (recover_io(pcm, (int)written) != 0) {
        snd_pcm_close(pcm);
        return 1;
      }
      --period;
      continue;
    }
    frames += (uint64_t)written;
    if (expect_interruption && period == 0) {
      printf("[AQUA-AUDIO] stage=media-probe status=active "
             "direction=playback frames=%llu\n",
             (unsigned long long)frames);
      fflush(stdout);
      poll(NULL, 0, 2000);
    }
  }
  status = snd_pcm_drain(pcm);
  snd_pcm_close(pcm);
  if (expect_interruption) {
    fprintf(stderr, "[AQUA-AUDIO] stage=media-probe status=failed "
                    "reason=interruption-not-observed frames=%llu\n",
            (unsigned long long)frames);
    return 1;
  }
  if (status < 0 || frames != (uint64_t)PERIOD_FRAMES * required_periods) {
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

static int capture(enum capture_requirement requirement,
                   unsigned int required_periods, int expect_interruption) {
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
  uint64_t nonzero_samples = 0;
  uint64_t positive_samples = 0;
  uint64_t negative_samples = 0;
  for (unsigned int period = 0; period < required_periods; ++period) {
    status = snd_pcm_wait(pcm, 5000);
    if (status <= 0) {
      if (expect_interruption && status < 0) {
        fprintf(stderr,
                "[AQUA-AUDIO] stage=media-probe status=interrupted "
                "direction=capture reason=pcm-io operation=wait frames=%llu "
                "detail=%s\n",
                (unsigned long long)frames, snd_strerror(status));
        snd_pcm_close(pcm);
        return 3;
      }
      fprintf(stderr, "[AQUA-AUDIO] stage=media-probe status=failed "
                      "reason=capture-timeout\n");
      snd_pcm_close(pcm);
      return 1;
    }
    snd_pcm_sframes_t read_frames = snd_pcm_readi(pcm, samples, PERIOD_FRAMES);
    if (read_frames < 0) {
      if (expect_interruption) {
        fprintf(stderr,
                "[AQUA-AUDIO] stage=media-probe status=interrupted "
                "direction=capture reason=pcm-io operation=read frames=%llu "
                "detail=%s\n",
                (unsigned long long)frames, snd_strerror((int)read_frames));
        snd_pcm_close(pcm);
        return 3;
      }
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
        if (sample > 0) {
          ++positive_samples;
          ++nonzero_samples;
        } else if (sample < 0) {
          ++negative_samples;
          ++nonzero_samples;
        }
      }
    }
    frames += (uint64_t)read_frames;
    if (expect_interruption && period == 0) {
      printf("[AQUA-AUDIO] stage=media-probe status=active "
             "direction=capture frames=%llu\n",
             (unsigned long long)frames);
      fflush(stdout);
      poll(NULL, 0, 2000);
    }
  }
  snd_pcm_drop(pcm);
  snd_pcm_close(pcm);
  if (expect_interruption) {
    fprintf(stderr, "[AQUA-AUDIO] stage=media-probe status=failed "
                    "reason=interruption-not-observed frames=%llu\n",
            (unsigned long long)frames);
    return 1;
  }
  if (frames != (uint64_t)PERIOD_FRAMES * required_periods) {
    fprintf(stderr, "[AQUA-AUDIO] stage=media-probe status=failed "
                    "reason=incomplete-capture frames=%llu\n",
            (unsigned long long)frames);
    return 1;
  }
  if (requirement == CAPTURE_REQUIRE_SILENCE && peak_abs != 0) {
    fprintf(stderr, "[AQUA-AUDIO] stage=media-probe status=failed "
                    "reason=unexpected-capture-data peak_abs=%u\n",
            peak_abs);
    return 1;
  }
  if (requirement == CAPTURE_REQUIRE_SIGNAL &&
      (peak_abs < 1024 || peak_abs > 8192 || nonzero_samples < 8000 ||
       positive_samples < 1000 || negative_samples < 1000)) {
    fprintf(stderr, "[AQUA-AUDIO] stage=media-probe status=failed "
                    "reason=invalid-injected-signal peak_abs=%u "
                    "nonzero_samples=%llu positive_samples=%llu "
                    "negative_samples=%llu\n",
            peak_abs, (unsigned long long)nonzero_samples,
            (unsigned long long)positive_samples,
            (unsigned long long)negative_samples);
    return 1;
  }
  printf("[AQUA-AUDIO] stage=media-probe status=ok direction=capture "
         "frames=%llu rate=%d channels=%d format=s16le peak_abs=%u "
         "pattern=%s nonzero_samples=%llu positive_samples=%llu "
         "negative_samples=%llu\n",
         (unsigned long long)frames, SAMPLE_RATE, CHANNELS, peak_abs,
         requirement == CAPTURE_REQUIRE_SILENCE
             ? "silence"
             : (requirement == CAPTURE_REQUIRE_SIGNAL ? "bipolar-injected"
                                                      : "observed"),
         (unsigned long long)nonzero_samples,
         (unsigned long long)positive_samples,
         (unsigned long long)negative_samples);
  return 0;
}

int main(int argc, char **argv) {
  if (argc != 2) {
    fprintf(stderr,
            "usage: aqua-audio-probe playback|playback-expect-interruption|capture|capture-silence|capture-signal|capture-expect-interruption|controls|routes|routes-secondary|fallback\n");
    return 2;
  }
  if (strcmp(argv[1], "playback") == 0) {
    return playback(PLAYBACK_PERIODS, 0);
  }
  if (strcmp(argv[1], "playback-expect-interruption") == 0) {
    return playback(INTERRUPTED_PLAYBACK_PERIODS, 1);
  }
  if (strcmp(argv[1], "capture") == 0) {
    return capture(CAPTURE_OBSERVE, CAPTURE_PERIODS, 0);
  }
  if (strcmp(argv[1], "capture-silence") == 0) {
    return capture(CAPTURE_REQUIRE_SILENCE, CAPTURE_PERIODS, 0);
  }
  if (strcmp(argv[1], "capture-signal") == 0) {
    return capture(CAPTURE_REQUIRE_SIGNAL, CAPTURE_PERIODS, 0);
  }
  if (strcmp(argv[1], "capture-expect-interruption") == 0) {
    return capture(CAPTURE_OBSERVE, INTERRUPTED_CAPTURE_PERIODS, 1);
  }
  if (strcmp(argv[1], "controls") == 0) {
    return controls();
  }
  if (strcmp(argv[1], "routes") == 0) {
    return routes(NULL);
  }
  if (strcmp(argv[1], "routes-secondary") == 0) {
    return routes("pci-0000_00_05.0");
  }
  if (strcmp(argv[1], "fallback") == 0) {
    return fallback();
  }
  fprintf(stderr,
          "usage: aqua-audio-probe playback|playback-expect-interruption|capture|capture-silence|capture-signal|capture-expect-interruption|controls|routes|routes-secondary|fallback\n");
  return 2;
}
