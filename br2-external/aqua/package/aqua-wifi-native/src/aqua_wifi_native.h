#ifndef AQUA_WIFI_NATIVE_H
#define AQUA_WIFI_NATIVE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define AQUA_WIFI_NATIVE_ABI_VERSION 1U
#define AQUA_WIFI_NATIVE_CONTROL_PATH "/run/wpa_supplicant/wlan0"
#define AQUA_WIFI_NATIVE_MAX_COMMAND_BYTES 192U
#define AQUA_WIFI_NATIVE_MAX_RESPONSE_BYTES 4096U
#define AQUA_WIFI_NATIVE_PSK_BYTES 32U

enum aqua_wifi_native_status {
  AQUA_WIFI_NATIVE_OK = 0,
  AQUA_WIFI_NATIVE_INVALID_ARGUMENT = -1,
  AQUA_WIFI_NATIVE_CONNECT_FAILED = -2,
  AQUA_WIFI_NATIVE_TIMEOUT = -3,
  AQUA_WIFI_NATIVE_API_FAILED = -4,
  AQUA_WIFI_NATIVE_BOUNDS_EXCEEDED = -5,
  AQUA_WIFI_NATIVE_DERIVATION_FAILED = -6,
};

struct aqua_wifi_native;

uint32_t aqua_wifi_native_abi_version(void);
int32_t aqua_wifi_native_derive_wpa2_psk(const uint8_t *ssid,
    size_t ssid_length, const uint8_t *passphrase, size_t passphrase_length,
    uint8_t out_psk[AQUA_WIFI_NATIVE_PSK_BYTES]);
int32_t aqua_wifi_native_open(struct aqua_wifi_native **out_handle);
void aqua_wifi_native_close(struct aqua_wifi_native *handle);
int32_t aqua_wifi_native_request(struct aqua_wifi_native *handle,
    const uint8_t *command, size_t command_length, uint8_t *response,
    size_t *response_length);

#ifdef __cplusplus
}
#endif

#endif
