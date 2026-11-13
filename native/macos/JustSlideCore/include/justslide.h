// C interface to justslide-core (crates/justslide-ffi). All payloads are
// NUL-terminated UTF-8 JSON; strings passed to callbacks are valid only for
// the duration of the call. Callbacks may run on core threads.
#ifndef JUSTSLIDE_H
#define JUSTSLIDE_H
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct JsCore JsCore;
typedef void (*JsCallback)(void *ctx, const char *json);

typedef struct JsHostCallbacks {
  void *ctx;
  JsCallback engine_command;  // EngineCommand JSON for the WebKit adapter
  JsCallback core_event;      // {"seq":n,"event":"..."} for UI refresh
} JsHostCallbacks;

enum { JS_OPEN_IN_MEMORY = 1, JS_OPEN_NO_IPC = 2 };

JsCore *justslide_core_open(const char *data_dir, const char *capabilities_json, JsHostCallbacks callbacks, uint32_t flags);
void justslide_core_execute(JsCore *core, const char *request_json, JsCallback reply, void *reply_ctx);
int32_t justslide_core_engine_event(JsCore *core, const char *event_json);
void justslide_core_restore_session(JsCore *core);
int32_t justslide_core_load_design_fixture(JsCore *core, const char *variant);  // DEBUG builds only
void justslide_core_close(JsCore *core);

#ifdef __cplusplus
}
#endif
#endif
