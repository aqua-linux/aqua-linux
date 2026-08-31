################################################################################
#
# aqua-wifi-native
#
################################################################################

AQUA_WIFI_NATIVE_VERSION = 1
AQUA_WIFI_NATIVE_SITE = $(BR2_EXTERNAL_AQUA_PATH)/package/aqua-wifi-native/src
AQUA_WIFI_NATIVE_SITE_METHOD = local
AQUA_WIFI_NATIVE_LICENSE = MIT
AQUA_WIFI_NATIVE_LICENSE_FILES = LICENSE
AQUA_WIFI_NATIVE_DEPENDENCIES = wpa_supplicant openssl
AQUA_WIFI_NATIVE_INSTALL_STAGING = YES

define AQUA_WIFI_NATIVE_BUILD_CMDS
	$(TARGET_MAKE_ENV) $(MAKE) $(TARGET_CONFIGURE_OPTS) -C $(@D)
endef

define AQUA_WIFI_NATIVE_INSTALL_STAGING_CMDS
	$(TARGET_MAKE_ENV) $(MAKE) -C $(@D) DESTDIR=$(STAGING_DIR) install-devel
endef

define AQUA_WIFI_NATIVE_INSTALL_TARGET_CMDS
	$(RM) $(TARGET_DIR)/usr/include/aqua_wifi_native.h \
		$(TARGET_DIR)/usr/lib/libaqua-wifi-native.so
	$(TARGET_MAKE_ENV) $(MAKE) -C $(@D) DESTDIR=$(TARGET_DIR) install-runtime
endef

$(eval $(generic-package))
