################################################################################
#
# aqua-audio-probe
#
################################################################################

AQUA_AUDIO_PROBE_VERSION = 10
AQUA_AUDIO_PROBE_SITE = $(BR2_EXTERNAL_AQUA_PATH)/package/aqua-audio-probe/src
AQUA_AUDIO_PROBE_SITE_METHOD = local
AQUA_AUDIO_PROBE_LICENSE = MIT
AQUA_AUDIO_PROBE_LICENSE_FILES = LICENSE
AQUA_AUDIO_PROBE_DEPENDENCIES = alsa-lib aqua-audio-native

define AQUA_AUDIO_PROBE_BUILD_CMDS
	$(TARGET_MAKE_ENV) $(MAKE) $(TARGET_CONFIGURE_OPTS) -C $(@D)
endef

define AQUA_AUDIO_PROBE_INSTALL_TARGET_CMDS
	$(INSTALL) -D -m 0755 $(@D)/aqua-audio-probe \
		$(TARGET_DIR)/usr/bin/aqua-audio-probe
endef

$(eval $(generic-package))
