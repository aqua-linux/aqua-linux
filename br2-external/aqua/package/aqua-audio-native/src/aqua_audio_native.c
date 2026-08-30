#include "aqua_audio_native.h"

#include <math.h>
#include <pipewire/keys.h>
#include <stdbool.h>
#include <stddef.h>
#include <spa/utils/defs.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <wp/wp.h>

_Static_assert(sizeof(struct aqua_audio_native_node) == 166,
    "native node ABI layout changed");
_Static_assert(offsetof(struct aqua_audio_native_snapshot, nodes) == 154,
    "native snapshot ABI layout changed");

struct aqua_audio_native {
  GMainLoop *loop;
  WpCore *core;
  WpObjectManager *object_manager;
  WpPlugin *default_nodes;
  WpPlugin *mixer;
  GCancellable *cancellable;
  gint references;
  uint32_t phase;
  uint64_t generation;
  struct aqua_audio_native_snapshot last_snapshot;
  uint32_t pending_plugins;
  guint timeout_source;
  bool operation_complete;
  bool initialization_complete;
  bool has_snapshot;
  bool closing;
  int32_t operation_status;
  char error[AQUA_AUDIO_NATIVE_ERROR_BYTES];
};

static int compare_nodes_by_name(const void *left, const void *right) {
  const struct aqua_audio_native_node *left_node = left;
  const struct aqua_audio_native_node *right_node = right;
  return strcmp(left_node->name, right_node->name);
}

static bool snapshot_payload_equal(
    const struct aqua_audio_native_snapshot *left,
    const struct aqua_audio_native_snapshot *right) {
  return left->phase == right->phase &&
      left->node_count == right->node_count &&
      strcmp(left->default_output, right->default_output) == 0 &&
      strcmp(left->default_input, right->default_input) == 0 &&
      memcmp(left->nodes, right->nodes,
          left->node_count * sizeof(left->nodes[0])) == 0;
}

static struct aqua_audio_native *handle_ref(
    struct aqua_audio_native *handle) {
  g_atomic_int_inc(&handle->references);
  return handle;
}

static void handle_unref(struct aqua_audio_native *handle) {
  if (!g_atomic_int_dec_and_test(&handle->references))
    return;
  g_clear_object(&handle->cancellable);
  g_clear_object(&handle->mixer);
  g_clear_object(&handle->default_nodes);
  g_clear_object(&handle->object_manager);
  g_clear_object(&handle->core);
  g_clear_pointer(&handle->loop, g_main_loop_unref);
  free(handle);
}

static void set_error(struct aqua_audio_native *handle, int32_t status,
    const char *message) {
  handle->operation_status = status;
  g_strlcpy(handle->error, message, sizeof(handle->error));
}

static gboolean operation_timeout(gpointer data) {
  struct aqua_audio_native *handle = data;
  handle->timeout_source = 0;
  handle->phase = AQUA_AUDIO_NATIVE_DEGRADED;
  set_error(handle, AQUA_AUDIO_NATIVE_TIMEOUT,
      "WirePlumber operation timed out");
  if (handle->cancellable)
    g_cancellable_cancel(handle->cancellable);
  g_main_loop_quit(handle->loop);
  return G_SOURCE_REMOVE;
}

static void begin_wait(struct aqua_audio_native *handle, uint32_t timeout_ms) {
  handle->operation_complete = false;
  handle->operation_status = AQUA_AUDIO_NATIVE_OK;
  handle->error[0] = '\0';
  g_clear_object(&handle->cancellable);
  handle->cancellable = g_cancellable_new();
  handle->timeout_source = g_timeout_add(MAX(timeout_ms, 1U),
      operation_timeout, handle);
}

static int32_t end_wait(struct aqua_audio_native *handle) {
  if (handle->timeout_source != 0) {
    g_source_remove(handle->timeout_source);
    handle->timeout_source = 0;
  }
  return handle->operation_status;
}

static void on_disconnected(WpCore *core, gpointer data) {
  struct aqua_audio_native *handle = data;
  (void)core;
  handle->phase = AQUA_AUDIO_NATIVE_DISCONNECTED;
  set_error(handle, AQUA_AUDIO_NATIVE_CONNECT_FAILED,
      "PipeWire disconnected");
  if (g_main_loop_is_running(handle->loop))
    g_main_loop_quit(handle->loop);
}

static void on_object_manager_installed(WpObjectManager *object_manager,
    gpointer data) {
  struct aqua_audio_native *handle = data;
  (void)object_manager;
  handle->initialization_complete = true;
  handle->operation_complete = true;
  handle->phase = AQUA_AUDIO_NATIVE_READY;
  g_main_loop_quit(handle->loop);
}

static void on_plugin_activated(WpObject *object, GAsyncResult *result,
    gpointer data) {
  struct aqua_audio_native *handle = data;
  g_autoptr(GError) error = NULL;
  if (handle->closing)
    goto finished;
  if (!wp_object_activate_finish(object, result, &error)) {
    if (handle->operation_status == AQUA_AUDIO_NATIVE_TIMEOUT)
      goto finished;
    handle->phase = AQUA_AUDIO_NATIVE_DEGRADED;
    set_error(handle, AQUA_AUDIO_NATIVE_API_FAILED, error->message);
    g_main_loop_quit(handle->loop);
    goto finished;
  }
  if (--handle->pending_plugins == 0)
    wp_core_install_object_manager(handle->core, handle->object_manager);

finished:
  handle_unref(handle);
}

static void on_plugin_loaded(WpCore *core, GAsyncResult *result,
    gpointer data) {
  struct aqua_audio_native *handle = data;
  g_autoptr(GError) error = NULL;
  if (handle->closing)
    goto finished;
  if (!wp_core_load_component_finish(core, result, &error)) {
    if (handle->operation_status == AQUA_AUDIO_NATIVE_TIMEOUT)
      goto finished;
    handle->phase = AQUA_AUDIO_NATIVE_DEGRADED;
    set_error(handle, AQUA_AUDIO_NATIVE_API_FAILED, error->message);
    g_main_loop_quit(handle->loop);
    goto finished;
  }
  if (--handle->pending_plugins == 0) {
    handle->default_nodes = wp_plugin_find(core, "default-nodes-api");
    handle->mixer = wp_plugin_find(core, "mixer-api");
    if (!handle->default_nodes || !handle->mixer) {
      handle->phase = AQUA_AUDIO_NATIVE_DEGRADED;
      set_error(handle, AQUA_AUDIO_NATIVE_API_FAILED,
          "Required WirePlumber APIs are unavailable");
      g_main_loop_quit(handle->loop);
      goto finished;
    }
    g_object_set(handle->mixer, "scale", 1, NULL);
    handle->pending_plugins = 2;
    wp_object_activate(WP_OBJECT(handle->default_nodes),
        WP_PLUGIN_FEATURE_ENABLED, handle->cancellable,
        (GAsyncReadyCallback)on_plugin_activated, handle_ref(handle));
    wp_object_activate(WP_OBJECT(handle->mixer), WP_PLUGIN_FEATURE_ENABLED,
        handle->cancellable, (GAsyncReadyCallback)on_plugin_activated,
        handle_ref(handle));
  }

finished:
  handle_unref(handle);
}

static void on_sync_complete(WpCore *core, GAsyncResult *result,
    gpointer data) {
  struct aqua_audio_native *handle = data;
  g_autoptr(GError) error = NULL;
  if (handle->closing)
    goto finished;
  if (!wp_core_sync_finish(core, result, &error)) {
    if (handle->operation_status == AQUA_AUDIO_NATIVE_TIMEOUT)
      goto finished;
    handle->phase = AQUA_AUDIO_NATIVE_DEGRADED;
    set_error(handle, AQUA_AUDIO_NATIVE_API_FAILED, error->message);
  } else {
    handle->operation_complete = true;
    handle->phase = AQUA_AUDIO_NATIVE_READY;
  }
  g_main_loop_quit(handle->loop);

finished:
  handle_unref(handle);
}

static int32_t synchronize(struct aqua_audio_native *handle,
    uint32_t timeout_ms) {
  if (handle->phase == AQUA_AUDIO_NATIVE_DISCONNECTED)
    return AQUA_AUDIO_NATIVE_NOT_READY;
  handle->phase = AQUA_AUDIO_NATIVE_SYNCHRONIZING;
  begin_wait(handle, timeout_ms);
  wp_core_sync(handle->core, handle->cancellable,
      (GAsyncReadyCallback)on_sync_complete, handle_ref(handle));
  g_main_loop_run(handle->loop);
  return end_wait(handle);
}

uint32_t aqua_audio_native_abi_version(void) {
  return AQUA_AUDIO_NATIVE_ABI_VERSION;
}

int32_t aqua_audio_native_open(uint32_t timeout_ms,
    struct aqua_audio_native **out_handle) {
  if (!out_handle)
    return AQUA_AUDIO_NATIVE_INVALID_ARGUMENT;
  *out_handle = NULL;

  struct aqua_audio_native *handle = calloc(1, sizeof(*handle));
  if (!handle)
    return AQUA_AUDIO_NATIVE_API_FAILED;
  handle->references = 1;

  wp_init(WP_INIT_ALL);
  handle->phase = AQUA_AUDIO_NATIVE_CONNECTING;
  handle->loop = g_main_loop_new(NULL, FALSE);
  handle->core = wp_core_new(NULL, NULL, NULL);
  handle->object_manager = wp_object_manager_new();
  if (!handle->loop || !handle->core || !handle->object_manager) {
    set_error(handle, AQUA_AUDIO_NATIVE_API_FAILED,
        "Unable to allocate WirePlumber client state");
    *out_handle = handle;
    return AQUA_AUDIO_NATIVE_API_FAILED;
  }

  wp_object_manager_add_interest(handle->object_manager, WP_TYPE_NODE, NULL);
  wp_object_manager_request_object_features(handle->object_manager,
      WP_TYPE_NODE, WP_PIPEWIRE_OBJECT_FEATURES_MINIMAL);
  wp_object_manager_add_interest(handle->object_manager, WP_TYPE_METADATA,
      NULL);
  wp_object_manager_request_object_features(handle->object_manager,
      WP_TYPE_METADATA, WP_OBJECT_FEATURES_ALL);

  g_signal_connect(handle->core, "disconnected",
      G_CALLBACK(on_disconnected), handle);
  g_signal_connect(handle->object_manager, "installed",
      G_CALLBACK(on_object_manager_installed), handle);

  if (!wp_core_connect(handle->core)) {
    set_error(handle, AQUA_AUDIO_NATIVE_CONNECT_FAILED,
        "Unable to connect to PipeWire");
    handle->phase = AQUA_AUDIO_NATIVE_DISCONNECTED;
    *out_handle = handle;
    return AQUA_AUDIO_NATIVE_CONNECT_FAILED;
  }

  begin_wait(handle, timeout_ms);
  handle->pending_plugins = 2;
  wp_core_load_component(handle->core,
      "libwireplumber-module-default-nodes-api", "module", NULL,
      "default-nodes-api", handle->cancellable,
      (GAsyncReadyCallback)on_plugin_loaded, handle_ref(handle));
  wp_core_load_component(handle->core, "libwireplumber-module-mixer-api",
      "module", NULL, "mixer-api", handle->cancellable,
      (GAsyncReadyCallback)on_plugin_loaded, handle_ref(handle));
  g_main_loop_run(handle->loop);
  int32_t status = end_wait(handle);
  if (status == AQUA_AUDIO_NATIVE_OK && !handle->initialization_complete) {
    set_error(handle, AQUA_AUDIO_NATIVE_API_FAILED,
        "WirePlumber object manager was not installed");
    status = AQUA_AUDIO_NATIVE_API_FAILED;
  }
  *out_handle = handle;
  return status;
}

void aqua_audio_native_close(struct aqua_audio_native *handle) {
  if (!handle)
    return;
  handle->closing = true;
  if (handle->timeout_source != 0)
    g_source_remove(handle->timeout_source);
  if (handle->cancellable)
    g_cancellable_cancel(handle->cancellable);
  if (handle->core)
    g_signal_handlers_disconnect_by_data(handle->core, handle);
  if (handle->object_manager)
    g_signal_handlers_disconnect_by_data(handle->object_manager, handle);
  handle_unref(handle);
}

const char *aqua_audio_native_last_error(struct aqua_audio_native *handle) {
  return handle ? handle->error : "invalid native audio handle";
}

static WpPipewireObject *lookup_node(struct aqua_audio_native *handle,
    const char *node_name) {
  return WP_PIPEWIRE_OBJECT(wp_object_manager_lookup(handle->object_manager,
      WP_TYPE_NODE, WP_CONSTRAINT_TYPE_PW_PROPERTY, PW_KEY_NODE_NAME, "=s",
      node_name, NULL));
}

static bool media_class_kind(const char *media_class, uint8_t *kind) {
  if (g_strcmp0(media_class, "Audio/Sink") == 0) {
    *kind = AQUA_AUDIO_NATIVE_OUTPUT;
    return true;
  }
  if (g_strcmp0(media_class, "Audio/Source") == 0) {
    *kind = AQUA_AUDIO_NATIVE_INPUT;
    return true;
  }
  return false;
}

static int32_t copy_bounded(char *destination, size_t capacity,
    const char *source, struct aqua_audio_native *handle) {
  if (!source || strlen(source) >= capacity) {
    set_error(handle, AQUA_AUDIO_NATIVE_BOUNDS_EXCEEDED,
        "WirePlumber node text exceeds the Aqua ABI bound");
    return AQUA_AUDIO_NATIVE_BOUNDS_EXCEEDED;
  }
  g_strlcpy(destination, source, capacity);
  return AQUA_AUDIO_NATIVE_OK;
}

int32_t aqua_audio_native_snapshot(struct aqua_audio_native *handle,
    uint32_t timeout_ms, struct aqua_audio_native_snapshot *out_snapshot) {
  if (!handle || !out_snapshot)
    return AQUA_AUDIO_NATIVE_INVALID_ARGUMENT;
  memset(out_snapshot, 0, sizeof(*out_snapshot));
  out_snapshot->abi_version = AQUA_AUDIO_NATIVE_ABI_VERSION;
  out_snapshot->phase = handle->phase;

  int32_t status = synchronize(handle, timeout_ms);
  out_snapshot->phase = handle->phase;
  if (status != AQUA_AUDIO_NATIVE_OK)
    return status;

  if (!handle->default_nodes || !handle->mixer) {
    handle->phase = AQUA_AUDIO_NATIVE_DEGRADED;
    out_snapshot->phase = handle->phase;
    set_error(handle, AQUA_AUDIO_NATIVE_API_FAILED,
        "Required WirePlumber APIs are unavailable");
    return AQUA_AUDIO_NATIVE_API_FAILED;
  }

  guint32 default_output_id = SPA_ID_INVALID;
  guint32 default_input_id = SPA_ID_INVALID;
  g_signal_emit_by_name(handle->default_nodes, "get-default-node", "Audio/Sink",
      &default_output_id);
  g_signal_emit_by_name(handle->default_nodes, "get-default-node", "Audio/Source",
      &default_input_id);

  g_autoptr(WpIterator) iterator = wp_object_manager_new_filtered_iterator(
      handle->object_manager, WP_TYPE_NODE, NULL);
  GValue item = G_VALUE_INIT;
  while (wp_iterator_next(iterator, &item)) {
    WpPipewireObject *node = g_value_get_object(&item);
    const char *media_class = wp_pipewire_object_get_property(node,
        PW_KEY_MEDIA_CLASS);
    uint8_t kind = 0;
    if (media_class_kind(media_class, &kind)) {
      if (out_snapshot->node_count >= AQUA_AUDIO_NATIVE_MAX_NODES) {
        g_value_unset(&item);
        set_error(handle, AQUA_AUDIO_NATIVE_BOUNDS_EXCEEDED,
            "WirePlumber graph exceeds the Aqua node bound");
        return AQUA_AUDIO_NATIVE_BOUNDS_EXCEEDED;
      }
      const char *name = wp_pipewire_object_get_property(node,
          PW_KEY_NODE_NAME);
      const char *description = wp_pipewire_object_get_property(node,
          PW_KEY_NODE_DESCRIPTION);
      if (!description)
        description = name;
      struct aqua_audio_native_node *output =
          &out_snapshot->nodes[out_snapshot->node_count];
      status = copy_bounded(output->name, sizeof(output->name), name, handle);
      if (status == AQUA_AUDIO_NATIVE_OK)
        status = copy_bounded(output->description, sizeof(output->description),
            description, handle);
      if (status != AQUA_AUDIO_NATIVE_OK) {
        g_value_unset(&item);
        return status;
      }
      output->kind = kind;
      output->muted = 1;

      guint32 id = wp_proxy_get_bound_id(WP_PROXY(node));
      GVariant *properties = NULL;
      g_signal_emit_by_name(handle->mixer, "get-volume", id, &properties);
      if (properties) {
        gboolean muted = TRUE;
        gdouble volume = 0.0;
        if (g_variant_lookup(properties, "mute", "b", &muted))
          output->muted = muted ? 1 : 0;
        if (g_variant_lookup(properties, "volume", "d", &volume))
          output->volume_percent = (uint8_t)CLAMP(lround(volume * 100.0),
              0, 100);
        g_variant_unref(properties);
      }

      if (id == default_output_id && kind == 0)
        status = copy_bounded(out_snapshot->default_output,
            sizeof(out_snapshot->default_output), name, handle);
      else if (id == default_input_id && kind == 1)
        status = copy_bounded(out_snapshot->default_input,
            sizeof(out_snapshot->default_input), name, handle);
      if (status != AQUA_AUDIO_NATIVE_OK) {
        g_value_unset(&item);
        return status;
      }
      out_snapshot->node_count++;
    }
    g_value_unset(&item);
  }

  qsort(out_snapshot->nodes, out_snapshot->node_count,
      sizeof(out_snapshot->nodes[0]), compare_nodes_by_name);
  out_snapshot->phase = AQUA_AUDIO_NATIVE_READY;
  if (!handle->has_snapshot ||
      !snapshot_payload_equal(out_snapshot, &handle->last_snapshot)) {
    handle->generation++;
  }
  out_snapshot->generation = handle->generation;
  handle->last_snapshot = *out_snapshot;
  handle->has_snapshot = true;
  return AQUA_AUDIO_NATIVE_OK;
}

static int32_t set_volume_properties(struct aqua_audio_native *handle,
    const char *node_name, GVariant *properties, uint32_t timeout_ms) {
  if (!handle || !node_name || node_name[0] == '\0')
    return AQUA_AUDIO_NATIVE_INVALID_ARGUMENT;
  g_autoptr(WpPipewireObject) node = lookup_node(handle, node_name);
  if (!node) {
    set_error(handle, AQUA_AUDIO_NATIVE_NODE_NOT_FOUND,
        "WirePlumber output node was not found");
    return AQUA_AUDIO_NATIVE_NODE_NOT_FOUND;
  }
  const char *media_class = wp_pipewire_object_get_property(node,
      PW_KEY_MEDIA_CLASS);
  if (g_strcmp0(media_class, "Audio/Sink") != 0) {
    set_error(handle, AQUA_AUDIO_NATIVE_INVALID_ARGUMENT,
        "Requested node is not an audio output");
    return AQUA_AUDIO_NATIVE_INVALID_ARGUMENT;
  }
  if (!handle->mixer) {
    set_error(handle, AQUA_AUDIO_NATIVE_API_FAILED,
        "WirePlumber mixer API is unavailable");
    return AQUA_AUDIO_NATIVE_API_FAILED;
  }
  gboolean applied = FALSE;
  guint32 id = wp_proxy_get_bound_id(WP_PROXY(node));
  g_signal_emit_by_name(handle->mixer, "set-volume", id, properties, &applied);
  if (!applied) {
    set_error(handle, AQUA_AUDIO_NATIVE_API_FAILED,
        "WirePlumber rejected the mixer request");
    return AQUA_AUDIO_NATIVE_API_FAILED;
  }
  return synchronize(handle, timeout_ms);
}

int32_t aqua_audio_native_set_output_volume(struct aqua_audio_native *handle,
    const char *node_name, uint8_t volume_percent, uint32_t timeout_ms) {
  if (volume_percent > 100)
    return AQUA_AUDIO_NATIVE_INVALID_ARGUMENT;
  GVariantBuilder builder;
  g_variant_builder_init(&builder, G_VARIANT_TYPE_VARDICT);
  g_variant_builder_add(&builder, "{sv}", "volume",
      g_variant_new_double((double)volume_percent / 100.0));
  g_autoptr(GVariant) properties = g_variant_ref_sink(
      g_variant_builder_end(&builder));
  return set_volume_properties(handle, node_name, properties, timeout_ms);
}

int32_t aqua_audio_native_set_output_muted(struct aqua_audio_native *handle,
    const char *node_name, uint8_t muted, uint32_t timeout_ms) {
  if (muted > 1)
    return AQUA_AUDIO_NATIVE_INVALID_ARGUMENT;
  GVariantBuilder builder;
  g_variant_builder_init(&builder, G_VARIANT_TYPE_VARDICT);
  g_variant_builder_add(&builder, "{sv}", "mute",
      g_variant_new_boolean(muted != 0));
  g_autoptr(GVariant) properties = g_variant_ref_sink(
      g_variant_builder_end(&builder));
  return set_volume_properties(handle, node_name, properties, timeout_ms);
}

int32_t aqua_audio_native_set_configured_default_output(
    struct aqua_audio_native *handle, const char *node_name,
    uint32_t timeout_ms) {
  if (!handle || !node_name || node_name[0] == '\0')
    return AQUA_AUDIO_NATIVE_INVALID_ARGUMENT;
  g_autoptr(WpPipewireObject) node = lookup_node(handle, node_name);
  if (!node) {
    set_error(handle, AQUA_AUDIO_NATIVE_NODE_NOT_FOUND,
        "WirePlumber output node was not found");
    return AQUA_AUDIO_NATIVE_NODE_NOT_FOUND;
  }
  const char *media_class = wp_pipewire_object_get_property(node,
      PW_KEY_MEDIA_CLASS);
  if (g_strcmp0(media_class, "Audio/Sink") != 0) {
    set_error(handle, AQUA_AUDIO_NATIVE_INVALID_ARGUMENT,
        "Requested default node is not an audio output");
    return AQUA_AUDIO_NATIVE_INVALID_ARGUMENT;
  }
  if (!handle->default_nodes) {
    set_error(handle, AQUA_AUDIO_NATIVE_API_FAILED,
        "WirePlumber default-nodes API is unavailable");
    return AQUA_AUDIO_NATIVE_API_FAILED;
  }
  gboolean applied = FALSE;
  g_signal_emit_by_name(handle->default_nodes,
      "set-default-configured-node-name",
      "Audio/Sink", node_name, &applied);
  if (!applied) {
    set_error(handle, AQUA_AUDIO_NATIVE_API_FAILED,
        "WirePlumber rejected the default output request");
    return AQUA_AUDIO_NATIVE_API_FAILED;
  }
  return synchronize(handle, timeout_ms);
}
