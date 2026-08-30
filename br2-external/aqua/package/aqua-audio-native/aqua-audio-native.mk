################################################################################
#
# aqua-audio-native
#
################################################################################

AQUA_AUDIO_NATIVE_VERSION = 1
AQUA_AUDIO_NATIVE_SITE = $(BR2_EXTERNAL_AQUA_PATH)/package/aqua-audio-native/src
AQUA_AUDIO_NATIVE_SITE_METHOD = local
AQUA_AUDIO_NATIVE_LICENSE = MIT
AQUA_AUDIO_NATIVE_LICENSE_FILES = LICENSE
AQUA_AUDIO_NATIVE_DEPENDENCIES = host-pkgconf wireplumber
AQUA_AUDIO_NATIVE_INSTALL_STAGING = YES

# Buildroot 2025.02.17 installs WirePlumber only into the target tree. Stage its
# public development interface before compiling this dependent native library.
define AQUA_AUDIO_NATIVE_INSTALL_WIREPLUMBER_STAGING
	$(TARGET_MAKE_ENV) DESTDIR=$(STAGING_DIR) \
		$(NINJA) $(NINJA_OPTS) -C $(WIREPLUMBER_DIR)/buildroot-build install
endef
AQUA_AUDIO_NATIVE_PRE_BUILD_HOOKS += \
	AQUA_AUDIO_NATIVE_INSTALL_WIREPLUMBER_STAGING

define AQUA_AUDIO_NATIVE_BUILD_CMDS
	$(TARGET_MAKE_ENV) $(MAKE) $(TARGET_CONFIGURE_OPTS) -C $(@D)
endef

define AQUA_AUDIO_NATIVE_INSTALL_STAGING_CMDS
	$(TARGET_MAKE_ENV) $(MAKE) -C $(@D) DESTDIR=$(STAGING_DIR) install-devel
endef

define AQUA_AUDIO_NATIVE_INSTALL_TARGET_CMDS
	$(RM) $(TARGET_DIR)/usr/include/aqua_audio_native.h \
		$(TARGET_DIR)/usr/lib/libaqua-audio-native.so
	$(TARGET_MAKE_ENV) $(MAKE) -C $(@D) DESTDIR=$(TARGET_DIR) install-runtime
endef

$(eval $(generic-package))
