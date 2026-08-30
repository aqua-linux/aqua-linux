#ifndef AQUA_AUDIO_NATIVE_H
#define AQUA_AUDIO_NATIVE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define AQUA_AUDIO_NATIVE_ABI_VERSION 1U
#define AQUA_AUDIO_NATIVE_MAX_NODES 32U
#define AQUA_AUDIO_NATIVE_NODE_NAME_BYTES 65U
#define AQUA_AUDIO_NATIVE_NODE_DESCRIPTION_BYTES 97U
#define AQUA_AUDIO_NATIVE_ERROR_BYTES 192U

enum aqua_audio_native_status {
  AQUA_AUDIO_NATIVE_OK = 0,
  AQUA_AUDIO_NATIVE_INVALID_ARGUMENT = -1,
  AQUA_AUDIO_NATIVE_CONNECT_FAILED = -2,
  AQUA_AUDIO_NATIVE_TIMEOUT = -3,
  AQUA_AUDIO_NATIVE_API_FAILED = -4,
  AQUA_AUDIO_NATIVE_BOUNDS_EXCEEDED = -5,
  AQUA_AUDIO_NATIVE_NOT_READY = -6,
  AQUA_AUDIO_NATIVE_NODE_NOT_FOUND = -7,
};

enum aqua_audio_native_phase {
  AQUA_AUDIO_NATIVE_DISCONNECTED = 0,
  AQUA_AUDIO_NATIVE_CONNECTING = 1,
  AQUA_AUDIO_NATIVE_SYNCHRONIZING = 2,
  AQUA_AUDIO_NATIVE_READY = 3,
  AQUA_AUDIO_NATIVE_DEGRADED = 4,
};

enum aqua_audio_native_node_kind {
  AQUA_AUDIO_NATIVE_OUTPUT = 0,
  AQUA_AUDIO_NATIVE_INPUT = 1,
};

struct aqua_audio_native_node {
  char name[AQUA_AUDIO_NATIVE_NODE_NAME_BYTES];
  char description[AQUA_AUDIO_NATIVE_NODE_DESCRIPTION_BYTES];
  uint8_t kind;
  uint8_t volume_percent;
  uint8_t muted;
  uint8_t reserved;
};

struct aqua_audio_native_snapshot {
  uint32_t abi_version;
  uint32_t phase;
  uint64_t generation;
  uint32_t node_count;
  uint32_t reserved;
  char default_output[AQUA_AUDIO_NATIVE_NODE_NAME_BYTES];
  char default_input[AQUA_AUDIO_NATIVE_NODE_NAME_BYTES];
  struct aqua_audio_native_node nodes[AQUA_AUDIO_NATIVE_MAX_NODES];
};

struct aqua_audio_native;

uint32_t aqua_audio_native_abi_version(void);
int32_t aqua_audio_native_open(uint32_t timeout_ms,
    struct aqua_audio_native **out_handle);
void aqua_audio_native_close(struct aqua_audio_native *handle);
const char *aqua_audio_native_last_error(struct aqua_audio_native *handle);
int32_t aqua_audio_native_snapshot(struct aqua_audio_native *handle,
    uint32_t timeout_ms, struct aqua_audio_native_snapshot *out_snapshot);
int32_t aqua_audio_native_set_output_volume(struct aqua_audio_native *handle,
    const char *node_name, uint8_t volume_percent, uint32_t timeout_ms);
int32_t aqua_audio_native_set_output_muted(struct aqua_audio_native *handle,
    const char *node_name, uint8_t muted, uint32_t timeout_ms);
int32_t aqua_audio_native_set_configured_default_output(
    struct aqua_audio_native *handle, const char *node_name,
    uint32_t timeout_ms);

#ifdef __cplusplus
}
#endif

#endif
