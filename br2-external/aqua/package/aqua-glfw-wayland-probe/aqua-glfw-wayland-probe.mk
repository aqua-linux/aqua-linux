################################################################################
#
# aqua-glfw-wayland-probe
#
################################################################################

AQUA_GLFW_WAYLAND_PROBE_VERSION = 4
AQUA_GLFW_WAYLAND_PROBE_SITE = $(BR2_EXTERNAL_AQUA_PATH)/package/aqua-glfw-wayland-probe/src
AQUA_GLFW_WAYLAND_PROBE_SITE_METHOD = local
AQUA_GLFW_WAYLAND_PROBE_LICENSE = MIT
AQUA_GLFW_WAYLAND_PROBE_LICENSE_FILES = LICENSE
AQUA_GLFW_WAYLAND_PROBE_DEPENDENCIES = host-pkgconf libglfw wayland

define AQUA_GLFW_WAYLAND_PROBE_BUILD_CMDS
	$(TARGET_MAKE_ENV) $(MAKE) $(TARGET_CONFIGURE_OPTS) -C $(@D)
endef

define AQUA_GLFW_WAYLAND_PROBE_INSTALL_TARGET_CMDS
	$(INSTALL) -D -m 0755 $(@D)/aqua-glfw-wayland-probe \
		$(TARGET_DIR)/usr/libexec/aqua-tests/aqua-glfw-wayland-probe
endef

$(eval $(generic-package))
