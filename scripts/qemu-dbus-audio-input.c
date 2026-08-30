#include <errno.h>
#include <fcntl.h>
#include <gio/gio.h>
#include <gio/gunixfdlist.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

enum {
  EXPECTED_BITS = 16,
  EXPECTED_RATE = 48000,
  EXPECTED_CHANNELS = 2,
  EXPECTED_BYTES_PER_FRAME = 4,
  REQUIRED_FRAMES = 4800,
  SIGNAL_AMPLITUDE = 4096,
  HALF_PERIOD_FRAMES = 24,
};

static const char listener_xml[] =
    "<node>"
    "<interface name='org.qemu.Display1.AudioInListener'>"
    "<method name='Init'><arg type='t' direction='in'/><arg type='y' "
    "direction='in'/><arg type='b' direction='in'/><arg type='b' "
    "direction='in'/><arg type='u' direction='in'/><arg type='y' "
    "direction='in'/><arg type='u' direction='in'/><arg type='u' "
    "direction='in'/><arg type='b' direction='in'/></method>"
    "<method name='Fini'><arg type='t' direction='in'/></method>"
    "<method name='SetEnabled'><arg type='t' direction='in'/><arg type='b' "
    "direction='in'/></method>"
    "<method name='SetVolume'><arg type='t' direction='in'/><arg type='b' "
    "direction='in'/><arg type='ay' direction='in'/></method>"
    "<method name='Read'><arg type='t' direction='in'/><arg type='t' "
    "direction='in'/><arg type='ay' direction='out'/></method>"
    "</interface>"
    "</node>";

struct injector {
  int listener_fd;
  const char *result_path;
  GMutex mutex;
  GCond condition;
  GMainLoop *loop;
  gboolean initialized;
  gboolean format_valid;
  uint64_t bytes_served;
  uint64_t sample_index;
};

static int connect_unix_socket(const char *path) {
  int fd = socket(AF_UNIX, SOCK_STREAM, 0);
  if (fd < 0) {
    return -1;
  }
  struct sockaddr_un address = {0};
  address.sun_family = AF_UNIX;
  if (strlen(path) >= sizeof(address.sun_path)) {
    close(fd);
    errno = ENAMETOOLONG;
    return -1;
  }
  strcpy(address.sun_path, path);
  for (unsigned int attempt = 0; attempt < 100; ++attempt) {
    if (connect(fd, (struct sockaddr *)&address, sizeof(address)) == 0) {
      return fd;
    }
    if (errno != ENOENT && errno != ECONNREFUSED) {
      break;
    }
    usleep(50000);
  }
  close(fd);
  return -1;
}

static int qmp_read_result(int fd) {
  char line[8192];
  size_t used = 0;
  for (;;) {
    char byte;
    ssize_t count = read(fd, &byte, 1);
    if (count <= 0) {
      return -1;
    }
    if (byte == '\n') {
      line[used] = '\0';
      if (strstr(line, "\"error\"") != NULL) {
        fprintf(stderr, "QMP command failed: %s\n", line);
        return -1;
      }
      if (strstr(line, "\"return\"") != NULL ||
          strstr(line, "\"QMP\"") != NULL) {
        return 0;
      }
      used = 0;
      continue;
    }
    if (byte != '\r' && used + 1 < sizeof(line)) {
      line[used++] = byte;
    }
  }
}

static int qmp_command(int fd, const char *json, int passed_fd) {
  struct iovec vector = {.iov_base = (void *)json, .iov_len = strlen(json)};
  char control[CMSG_SPACE(sizeof(int))] = {0};
  struct msghdr message = {.msg_iov = &vector, .msg_iovlen = 1};
  if (passed_fd >= 0) {
    message.msg_control = control;
    message.msg_controllen = sizeof(control);
    struct cmsghdr *header = CMSG_FIRSTHDR(&message);
    header->cmsg_level = SOL_SOCKET;
    header->cmsg_type = SCM_RIGHTS;
    header->cmsg_len = CMSG_LEN(sizeof(int));
    memcpy(CMSG_DATA(header), &passed_fd, sizeof(passed_fd));
  }
  if (sendmsg(fd, &message, 0) != (ssize_t)strlen(json)) {
    return -1;
  }
  return qmp_read_result(fd);
}

static int attach_display_client(const char *qmp_path, int display_fd) {
  int qmp_fd = connect_unix_socket(qmp_path);
  if (qmp_fd < 0) {
    perror("connect QMP socket");
    return -1;
  }
  int status = qmp_read_result(qmp_fd);
  if (status == 0) {
    status = qmp_command(qmp_fd, "{\"execute\":\"qmp_capabilities\"}\r\n", -1);
  }
  if (status == 0) {
    status = qmp_command(
        qmp_fd,
        "{\"execute\":\"getfd\",\"arguments\":{\"fdname\":\"aqua-audio-dbus\"}}\r\n",
        display_fd);
  }
  if (status == 0) {
    status = qmp_command(
        qmp_fd,
        "{\"execute\":\"add_client\",\"arguments\":{\"protocol\":\"@dbus-display\",\"fdname\":\"aqua-audio-dbus\"}}\r\n",
        -1);
  }
  close(qmp_fd);
  return status;
}

static void write_result(struct injector *injector) {
  if (injector->bytes_served <
      (uint64_t)REQUIRED_FRAMES * EXPECTED_BYTES_PER_FRAME) {
    return;
  }
  char result[256];
  int length = snprintf(
      result, sizeof(result),
      "status=ok\nformat=s16le\nrate=%d\nchannels=%d\namplitude=%d\n"
      "pattern=square-1khz\nbytes_served=%llu\n",
      EXPECTED_RATE, EXPECTED_CHANNELS, SIGNAL_AMPLITUDE,
      (unsigned long long)injector->bytes_served);
  if (length <= 0 || (size_t)length >= sizeof(result)) {
    return;
  }
  GError *error = NULL;
  if (!g_file_set_contents(injector->result_path, result, length, &error)) {
    fprintf(stderr, "Unable to write injector result: %s\n", error->message);
    g_error_free(error);
  }
}

static void handle_method_call(GDBusConnection *connection,
                               const gchar *sender,
                               const gchar *object_path,
                               const gchar *interface_name,
                               const gchar *method_name,
                               GVariant *parameters,
                               GDBusMethodInvocation *invocation,
                               gpointer user_data) {
  (void)connection;
  (void)sender;
  (void)object_path;
  (void)interface_name;
  struct injector *injector = user_data;

  if (strcmp(method_name, "Init") == 0) {
    guint64 stream_id;
    guchar bits;
    gboolean is_signed;
    gboolean is_float;
    guint32 rate;
    guchar channels;
    guint32 bytes_per_frame;
    guint32 bytes_per_second;
    gboolean big_endian;
    g_variant_get(parameters, "(tybbuyuub)", &stream_id, &bits, &is_signed,
                  &is_float, &rate, &channels, &bytes_per_frame,
                  &bytes_per_second, &big_endian);
    (void)stream_id;
    g_mutex_lock(&injector->mutex);
    injector->initialized = TRUE;
    injector->format_valid =
        bits == EXPECTED_BITS && is_signed && !is_float &&
        rate == EXPECTED_RATE && channels == EXPECTED_CHANNELS &&
        bytes_per_frame == EXPECTED_BYTES_PER_FRAME &&
        bytes_per_second == EXPECTED_RATE * EXPECTED_BYTES_PER_FRAME &&
        !big_endian;
    g_cond_signal(&injector->condition);
    g_mutex_unlock(&injector->mutex);
    g_dbus_method_invocation_return_value(invocation, NULL);
    return;
  }

  if (strcmp(method_name, "Read") == 0) {
    guint64 stream_id;
    guint64 requested;
    g_variant_get(parameters, "(tt)", &stream_id, &requested);
    (void)stream_id;
    if (requested > 1024 * 1024) {
      g_dbus_method_invocation_return_dbus_error(
          invocation, "org.qemu.Display1.Error.Invalid",
          "Audio read request exceeds the bounded injector limit");
      return;
    }
    gsize size = (gsize)requested;
    guint8 *data = g_malloc(size);
    for (gsize offset = 0; offset + 1 < size; offset += 2) {
      uint64_t frame = injector->sample_index / EXPECTED_CHANNELS;
      int16_t sample = ((frame / HALF_PERIOD_FRAMES) % 2 == 0)
                           ? SIGNAL_AMPLITUDE
                           : -SIGNAL_AMPLITUDE;
      data[offset] = (guint8)((uint16_t)sample & 0xff);
      data[offset + 1] = (guint8)(((uint16_t)sample >> 8) & 0xff);
      ++injector->sample_index;
    }
    if ((size & 1) != 0) {
      data[size - 1] = 0;
    }
    injector->bytes_served += size;
    write_result(injector);
    GVariant *bytes = g_variant_new_fixed_array(G_VARIANT_TYPE_BYTE, data,
                                                size, sizeof(guint8));
    g_dbus_method_invocation_return_value(invocation,
                                          g_variant_new("(@ay)", bytes));
    g_free(data);
    return;
  }

  g_dbus_method_invocation_return_value(invocation, NULL);
}

static const GDBusInterfaceVTable listener_vtable = {
    .method_call = handle_method_call,
};

static void connection_closed(GDBusConnection *connection,
                              gboolean remote_peer_vanished,
                              GError *error,
                              gpointer user_data) {
  (void)connection;
  (void)remote_peer_vanished;
  (void)error;
  struct injector *injector = user_data;
  if (injector->loop != NULL) {
    g_main_loop_quit(injector->loop);
  }
}

static gpointer run_listener(gpointer user_data) {
  struct injector *injector = user_data;
  GMainContext *context = g_main_context_new();
  g_main_context_push_thread_default(context);
  GError *error = NULL;
  GSocket *socket = g_socket_new_from_fd(injector->listener_fd, &error);
  if (socket == NULL) {
    fprintf(stderr, "Unable to adopt listener socket: %s\n", error->message);
    g_error_free(error);
    g_main_context_pop_thread_default(context);
    g_main_context_unref(context);
    return NULL;
  }
  GSocketConnection *stream = g_socket_connection_factory_create_connection(socket);
  g_object_unref(socket);
  GDBusConnection *connection = g_dbus_connection_new_sync(
      G_IO_STREAM(stream), NULL, G_DBUS_CONNECTION_FLAGS_AUTHENTICATION_CLIENT,
      NULL, NULL, &error);
  g_object_unref(stream);
  if (connection == NULL) {
    fprintf(stderr, "Unable to create audio listener connection: %s\n",
            error->message);
    g_error_free(error);
    g_main_context_pop_thread_default(context);
    g_main_context_unref(context);
    return NULL;
  }
  GDBusNodeInfo *node = g_dbus_node_info_new_for_xml(listener_xml, &error);
  if (node == NULL ||
      g_dbus_connection_register_object(
          connection, "/org/qemu/Display1/AudioInListener",
          node->interfaces[0], &listener_vtable, injector, NULL, &error) == 0) {
    fprintf(stderr, "Unable to register audio listener: %s\n", error->message);
    g_clear_error(&error);
    if (node != NULL) {
      g_dbus_node_info_unref(node);
    }
    g_object_unref(connection);
    g_main_context_pop_thread_default(context);
    g_main_context_unref(context);
    return NULL;
  }
  g_dbus_node_info_unref(node);
  injector->loop = g_main_loop_new(context, FALSE);
  g_signal_connect(connection, "closed", G_CALLBACK(connection_closed), injector);
  g_main_loop_run(injector->loop);
  g_main_loop_unref(injector->loop);
  injector->loop = NULL;
  g_object_unref(connection);
  g_main_context_pop_thread_default(context);
  g_main_context_unref(context);
  return NULL;
}

static GDBusConnection *open_display_connection(int fd, GError **error) {
  GSocket *socket = g_socket_new_from_fd(fd, error);
  if (socket == NULL) {
    return NULL;
  }
  GSocketConnection *stream = g_socket_connection_factory_create_connection(socket);
  g_object_unref(socket);
  GDBusConnection *connection = g_dbus_connection_new_sync(
      G_IO_STREAM(stream), NULL, G_DBUS_CONNECTION_FLAGS_AUTHENTICATION_CLIENT,
      NULL, NULL, error);
  g_object_unref(stream);
  return connection;
}

int main(int argc, char **argv) {
  if (argc != 4) {
    fprintf(stderr, "usage: qemu-dbus-audio-input QMP_SOCKET READY_FILE RESULT_FILE\n");
    return 2;
  }
  unlink(argv[2]);
  unlink(argv[3]);

  int display_pair[2];
  if (socketpair(AF_UNIX, SOCK_STREAM, 0, display_pair) != 0) {
    perror("display socketpair");
    return 1;
  }
  if (attach_display_client(argv[1], display_pair[1]) != 0) {
    close(display_pair[0]);
    close(display_pair[1]);
    return 1;
  }
  close(display_pair[1]);
  GError *error = NULL;
  GDBusConnection *display = open_display_connection(display_pair[0], &error);
  if (display == NULL) {
    fprintf(stderr, "Unable to create QEMU display connection: %s\n",
            error->message);
    g_error_free(error);
    return 1;
  }

  int listener_pair[2];
  if (socketpair(AF_UNIX, SOCK_STREAM, 0, listener_pair) != 0) {
    perror("listener socketpair");
    g_object_unref(display);
    return 1;
  }
  struct injector injector = {
      .listener_fd = listener_pair[0],
      .result_path = argv[3],
  };
  g_mutex_init(&injector.mutex);
  g_cond_init(&injector.condition);
  GThread *listener_thread = g_thread_new("audio-input", run_listener, &injector);

  GUnixFDList *fd_list = g_unix_fd_list_new();
  int handle = g_unix_fd_list_append(fd_list, listener_pair[1], &error);
  close(listener_pair[1]);
  GVariant *reply = NULL;
  if (handle >= 0) {
    reply = g_dbus_connection_call_with_unix_fd_list_sync(
        display, NULL, "/org/qemu/Display1/Audio",
        "org.qemu.Display1.Audio", "RegisterInListener",
        g_variant_new("(h)", handle), G_VARIANT_TYPE("()"),
        G_DBUS_CALL_FLAGS_NONE, 10000, fd_list, NULL, NULL, &error);
  }
  g_object_unref(fd_list);
  if (reply == NULL) {
    fprintf(stderr, "Unable to register QEMU audio input listener: %s\n",
            error != NULL ? error->message : "FD registration failed");
    g_clear_error(&error);
    g_dbus_connection_close_sync(display, NULL, NULL);
    g_object_unref(display);
    g_thread_join(listener_thread);
    g_cond_clear(&injector.condition);
    g_mutex_clear(&injector.mutex);
    return 1;
  }
  g_variant_unref(reply);

  gint64 deadline = g_get_monotonic_time() + 10 * G_TIME_SPAN_SECOND;
  g_mutex_lock(&injector.mutex);
  while (!injector.initialized &&
         g_cond_wait_until(&injector.condition, &injector.mutex, deadline)) {
  }
  gboolean initialized = injector.initialized;
  gboolean format_valid = injector.format_valid;
  g_mutex_unlock(&injector.mutex);
  if (!initialized || !format_valid) {
    fprintf(stderr, "QEMU audio input stream format was not accepted\n");
    g_dbus_connection_close_sync(display, NULL, NULL);
    g_object_unref(display);
    g_thread_join(listener_thread);
    g_cond_clear(&injector.condition);
    g_mutex_clear(&injector.mutex);
    return 1;
  }
  if (!g_file_set_contents(argv[2], "status=ready\n", -1, &error)) {
    fprintf(stderr, "Unable to write injector readiness: %s\n", error->message);
    g_error_free(error);
    g_dbus_connection_close_sync(display, NULL, NULL);
    g_object_unref(display);
    g_thread_join(listener_thread);
    g_cond_clear(&injector.condition);
    g_mutex_clear(&injector.mutex);
    return 1;
  }

  g_thread_join(listener_thread);
  g_object_unref(display);
  g_cond_clear(&injector.condition);
  g_mutex_clear(&injector.mutex);
  return 0;
}
