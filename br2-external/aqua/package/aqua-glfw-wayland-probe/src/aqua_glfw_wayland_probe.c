#define _POSIX_C_SOURCE 200112L
#define GLFW_EXPOSE_NATIVE_WAYLAND
#define GLFW_INCLUDE_NONE

#include <GLFW/glfw3.h>
#include <GLFW/glfw3native.h>
#include <errno.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>
#include <wayland-client.h>

enum {
    PROBE_WIDTH = 400,
    PROBE_HEIGHT = 240,
    PROBE_STRIDE = PROBE_WIDTH * 4,
    PROBE_SIZE = PROBE_STRIDE * PROBE_HEIGHT,
    PROBE_POOL_SIZE = PROBE_SIZE * 2,
};

struct probe_state {
    struct wl_shm *shm;
    struct wl_buffer *buffers[2];
    struct wl_surface *surface;
    uint32_t *pixels;
    int backing_fd;
    int repaint_requested;
    int repaint_committed;
};

static void registry_global(void *data, struct wl_registry *registry,
                            uint32_t name, const char *interface,
                            uint32_t version)
{
    struct probe_state *state = data;

    if (strcmp(interface, wl_shm_interface.name) == 0 && state->shm == NULL) {
        uint32_t bind_version = version < 1 ? version : 1;
        state->shm = wl_registry_bind(registry, name, &wl_shm_interface,
                                      bind_version);
    }
}

static void registry_global_remove(void *data, struct wl_registry *registry,
                                   uint32_t name)
{
    (void)data;
    (void)registry;
    (void)name;
}

static const struct wl_registry_listener registry_listener = {
    .global = registry_global,
    .global_remove = registry_global_remove,
};

static int create_backing_file(void)
{
    char name[64];
    int fd;

    if (snprintf(name, sizeof(name), "/aqua-glfw-probe-%ld", (long)getpid()) < 0)
        return -1;
    fd = shm_open(name, O_RDWR | O_CREAT | O_EXCL, 0600);
    if (fd < 0)
        return -1;
    if (shm_unlink(name) < 0 || ftruncate(fd, PROBE_POOL_SIZE) < 0) {
        close(fd);
        return -1;
    }
    return fd;
}

static void paint(struct probe_state *state, int buffer_index,
                  uint32_t background, uint32_t accent)
{
    uint32_t *pixels = state->pixels +
                       (buffer_index * PROBE_WIDTH * PROBE_HEIGHT);
    int x;
    int y;

    for (y = 0; y < PROBE_HEIGHT; y++) {
        for (x = 0; x < PROBE_WIDTH; x++) {
            int inside = x >= 48 && x < PROBE_WIDTH - 48 && y >= 48 &&
                         y < PROBE_HEIGHT - 48;
            pixels[(y * PROBE_WIDTH) + x] = inside ? accent : background;
        }
    }
}

static void commit_buffer(struct probe_state *state, int buffer_index)
{
    wl_surface_attach(state->surface, state->buffers[buffer_index], 0, 0);
    wl_surface_damage(state->surface, 0, 0, PROBE_WIDTH, PROBE_HEIGHT);
    wl_surface_commit(state->surface);
}

static void key_callback(GLFWwindow *window, int key, int scancode, int action,
                         int mods)
{
    struct probe_state *state = glfwGetWindowUserPointer(window);

    (void)scancode;
    (void)mods;
    if (key == GLFW_KEY_G && action == GLFW_PRESS)
        state->repaint_requested = 1;
}

static void report_glfw_error(int code, const char *description)
{
    fprintf(stderr, "GLFW error %d: %s\n", code, description);
}

static double monotonic_seconds(void)
{
    struct timespec now;

    if (clock_gettime(CLOCK_MONOTONIC, &now) != 0)
        return 0.0;
    return (double)now.tv_sec + ((double)now.tv_nsec / 1000000000.0);
}

int main(void)
{
    struct probe_state state = {.backing_fd = -1};
    struct wl_display *display;
    struct wl_registry *registry = NULL;
    struct wl_shm_pool *pool = NULL;
    GLFWwindow *window = NULL;
    double deadline;
    int exit_code = EXIT_FAILURE;

    glfwSetErrorCallback(report_glfw_error);
    glfwInitHint(GLFW_PLATFORM, GLFW_PLATFORM_WAYLAND);
    if (!glfwInit())
        goto cleanup;

    glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
    glfwWindowHint(GLFW_DECORATED, GLFW_FALSE);
    glfwWindowHint(GLFW_RESIZABLE, GLFW_FALSE);
    glfwWindowHintString(GLFW_WAYLAND_APP_ID, "aqua.glfw-wayland-probe");
    window = glfwCreateWindow(PROBE_WIDTH, PROBE_HEIGHT,
                              "Aqua GLFW Wayland Probe", NULL, NULL);
    if (window == NULL)
        goto cleanup;

    glfwSetWindowUserPointer(window, &state);
    glfwSetKeyCallback(window, key_callback);
    display = glfwGetWaylandDisplay();
    state.surface = glfwGetWaylandWindow(window);
    if (display == NULL || state.surface == NULL)
        goto cleanup;

    registry = wl_display_get_registry(display);
    if (registry == NULL ||
        wl_registry_add_listener(registry, &registry_listener, &state) < 0 ||
        wl_display_roundtrip(display) < 0 || state.shm == NULL)
        goto cleanup;

    state.backing_fd = create_backing_file();
    if (state.backing_fd < 0)
        goto cleanup;
    state.pixels = mmap(NULL, PROBE_POOL_SIZE, PROT_READ | PROT_WRITE, MAP_SHARED,
                        state.backing_fd, 0);
    if (state.pixels == MAP_FAILED) {
        state.pixels = NULL;
        goto cleanup;
    }
    pool = wl_shm_create_pool(state.shm, state.backing_fd, PROBE_POOL_SIZE);
    if (pool == NULL)
        goto cleanup;
    state.buffers[0] = wl_shm_pool_create_buffer(
        pool, 0, PROBE_WIDTH, PROBE_HEIGHT, PROBE_STRIDE,
        WL_SHM_FORMAT_ARGB8888);
    state.buffers[1] = wl_shm_pool_create_buffer(
        pool, PROBE_SIZE, PROBE_WIDTH, PROBE_HEIGHT, PROBE_STRIDE,
        WL_SHM_FORMAT_ARGB8888);
    if (state.buffers[0] == NULL || state.buffers[1] == NULL)
        goto cleanup;

    paint(&state, 0, 0xff14213dU, 0xff2f80edU);
    commit_buffer(&state, 0);
    deadline = monotonic_seconds() + 20.0;
    while (!glfwWindowShouldClose(window) && monotonic_seconds() < deadline) {
        glfwWaitEventsTimeout(0.1);
        if (state.repaint_requested) {
            paint(&state, 1, 0xff102a1fU, 0xff35c46aU);
            commit_buffer(&state, 1);
            state.repaint_requested = 0;
            state.repaint_committed = 1;
        }
    }
    if (glfwWindowShouldClose(window) && state.repaint_committed)
        exit_code = EXIT_SUCCESS;

cleanup:
    if (state.buffers[0] != NULL)
        wl_buffer_destroy(state.buffers[0]);
    if (state.buffers[1] != NULL)
        wl_buffer_destroy(state.buffers[1]);
    if (pool != NULL)
        wl_shm_pool_destroy(pool);
    if (state.pixels != NULL)
        munmap(state.pixels, PROBE_POOL_SIZE);
    if (state.backing_fd >= 0)
        close(state.backing_fd);
    if (registry != NULL)
        wl_registry_destroy(registry);
    if (state.shm != NULL)
        wl_shm_destroy(state.shm);
    if (window != NULL)
        glfwDestroyWindow(window);
    glfwTerminate();
    return exit_code;
}
