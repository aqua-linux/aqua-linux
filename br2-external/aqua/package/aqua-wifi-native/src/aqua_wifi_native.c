#include "aqua_wifi_native.h"

#include <openssl/evp.h>
#include <stdlib.h>
#include <string.h>
#include <wpa_ctrl.h>

struct aqua_wifi_native {
  struct wpa_ctrl *control;
};

static int bounded_printable_passphrase(const uint8_t *passphrase,
    size_t length) {
  size_t index;

  if (passphrase == NULL || length < 8U || length > 63U) {
    return 0;
  }
  for (index = 0U; index < length; index++) {
    if (passphrase[index] < 0x20U || passphrase[index] > 0x7eU) {
      return 0;
    }
  }
  return 1;
}

static int bounded_command(const uint8_t *command, size_t length) {
  size_t index;

  if (command == NULL || length == 0U ||
      length > AQUA_WIFI_NATIVE_MAX_COMMAND_BYTES) {
    return 0;
  }
  for (index = 0U; index < length; index++) {
    if (command[index] == 0U || command[index] == '\r' ||
        command[index] == '\n') {
      return 0;
    }
  }
  return 1;
}

uint32_t aqua_wifi_native_abi_version(void) {
  return AQUA_WIFI_NATIVE_ABI_VERSION;
}

int32_t aqua_wifi_native_derive_wpa2_psk(const uint8_t *ssid,
    size_t ssid_length, const uint8_t *passphrase, size_t passphrase_length,
    uint8_t out_psk[AQUA_WIFI_NATIVE_PSK_BYTES]) {
  if (ssid == NULL || ssid_length == 0U || ssid_length > 32U ||
      !bounded_printable_passphrase(passphrase, passphrase_length) ||
      out_psk == NULL) {
    return AQUA_WIFI_NATIVE_INVALID_ARGUMENT;
  }
  if (PKCS5_PBKDF2_HMAC_SHA1((const char *)passphrase,
          (int)passphrase_length, ssid, (int)ssid_length, 4096,
          AQUA_WIFI_NATIVE_PSK_BYTES, out_psk) != 1) {
    return AQUA_WIFI_NATIVE_DERIVATION_FAILED;
  }
  return AQUA_WIFI_NATIVE_OK;
}

int32_t aqua_wifi_native_open(struct aqua_wifi_native **out_handle) {
  struct aqua_wifi_native *handle;

  if (out_handle == NULL) {
    return AQUA_WIFI_NATIVE_INVALID_ARGUMENT;
  }
  *out_handle = NULL;
  handle = calloc(1U, sizeof(*handle));
  if (handle == NULL) {
    return AQUA_WIFI_NATIVE_API_FAILED;
  }
  handle->control = wpa_ctrl_open(AQUA_WIFI_NATIVE_CONTROL_PATH);
  if (handle->control == NULL) {
    free(handle);
    return AQUA_WIFI_NATIVE_CONNECT_FAILED;
  }
  *out_handle = handle;
  return AQUA_WIFI_NATIVE_OK;
}

void aqua_wifi_native_close(struct aqua_wifi_native *handle) {
  if (handle == NULL) {
    return;
  }
  if (handle->control != NULL) {
    wpa_ctrl_close(handle->control);
  }
  free(handle);
}

int32_t aqua_wifi_native_request(struct aqua_wifi_native *handle,
    const uint8_t *command, size_t command_length, uint8_t *response,
    size_t *response_length) {
  int result;

  if (handle == NULL || handle->control == NULL ||
      !bounded_command(command, command_length) || response == NULL ||
      response_length == NULL || *response_length == 0U) {
    return AQUA_WIFI_NATIVE_INVALID_ARGUMENT;
  }
  if (*response_length > AQUA_WIFI_NATIVE_MAX_RESPONSE_BYTES) {
    return AQUA_WIFI_NATIVE_BOUNDS_EXCEEDED;
  }
  result = wpa_ctrl_request(handle->control, (const char *)command,
      command_length, (char *)response, response_length, NULL);
  if (result == -2) {
    return AQUA_WIFI_NATIVE_TIMEOUT;
  }
  if (result != 0) {
    return AQUA_WIFI_NATIVE_API_FAILED;
  }
  if (*response_length > AQUA_WIFI_NATIVE_MAX_RESPONSE_BYTES) {
    return AQUA_WIFI_NATIVE_BOUNDS_EXCEEDED;
  }
  return AQUA_WIFI_NATIVE_OK;
}
