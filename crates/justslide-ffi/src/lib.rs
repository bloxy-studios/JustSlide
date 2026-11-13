//! C ABI that lets the macOS AppKit/WebKit host embed `justslide-core`.
//!
//! Everything crosses the boundary as NUL-terminated UTF-8 JSON. The unsafe
//! surface is limited to: reading caller-owned C strings, calling host
//! callbacks with Rust-owned strings that are valid only for the duration of
//! the call, and converting the opaque `JsCore` handle. Header:
//! native/macos/JustSlideCore/include/justslide.h

use justslide_core::engine::{BrowserEngine, EngineCapabilities, EngineCommand, EngineEvent};
use justslide_core::{Actor, Core, Request, ipc};
use serde_json::{Value, json};
use std::ffi::{CStr, CString, c_char, c_void};
use std::sync::Arc;

/// Host callback. `json` is valid only during the call; the host must copy it
/// and must not block (dispatch to the main queue instead).
pub type JsCallback = extern "C" fn(ctx: *mut c_void, json: *const c_char);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsHostCallbacks {
    pub ctx: *mut c_void,
    /// Receives `EngineCommand` JSON for the WebKit adapter.
    pub engine_command: JsCallback,
    /// Receives `{"seq":n,"event":"..."}` for every core event (UI refresh).
    pub core_event: JsCallback,
}

/// The host guarantees `ctx` outlives the core and that both callbacks are
/// thread-safe (they copy and hop to the main queue).
struct HostCallbacks(JsHostCallbacks);
unsafe impl Send for HostCallbacks {}
unsafe impl Sync for HostCallbacks {}

impl HostCallbacks {
    fn send(&self, f: JsCallback, v: &Value) {
        if let Ok(s) = CString::new(v.to_string()) {
            f(self.0.ctx, s.as_ptr());
        }
    }
}

struct FfiEngine {
    cb: Arc<HostCallbacks>,
    caps: EngineCapabilities,
}

impl BrowserEngine for FfiEngine {
    fn capabilities(&self) -> EngineCapabilities { self.caps.clone() }
    fn send(&self, cmd: EngineCommand) {
        let v = serde_json::to_value(&cmd).unwrap_or(Value::Null);
        self.cb.send(self.cb.0.engine_command, &v);
    }
}

pub struct JsCore {
    rt: tokio::runtime::Runtime,
    core: Arc<Core>,
    _ipc: Option<ipc::IpcHandle>,
}

/// Flags for `justslide_core_open`.
pub const JS_OPEN_IN_MEMORY: u32 = 1;
pub const JS_OPEN_NO_IPC: u32 = 2;

unsafe fn cstr<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    // SAFETY: caller passes a valid NUL-terminated string for the call's duration.
    unsafe { CStr::from_ptr(p) }.to_str().ok()
}

/// Opens the core. Returns NULL on failure (details on stderr).
///
/// # Safety
/// `data_dir` and `capabilities_json` must be valid C strings or NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn justslide_core_open(data_dir: *const c_char, capabilities_json: *const c_char, callbacks: JsHostCallbacks, flags: u32) -> *mut JsCore {
    let dir = unsafe { cstr(data_dir) }.map(std::path::PathBuf::from);
    let caps: EngineCapabilities = unsafe { cstr(capabilities_json) }
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| EngineCapabilities::unknown("webkit"));
    let rt = match tokio::runtime::Builder::new_multi_thread().worker_threads(2).thread_name("justslide-core").enable_all().build() {
        Ok(rt) => rt,
        Err(e) => { eprintln!("justslide: runtime: {e}"); return std::ptr::null_mut(); }
    };
    let in_memory = flags & JS_OPEN_IN_MEMORY != 0;
    let core = match Core::open(if in_memory { None } else { dir.as_deref() }) {
        Ok(c) => c,
        Err(e) => { eprintln!("justslide: core open failed: {e:#}"); return std::ptr::null_mut(); }
    };
    let cb = Arc::new(HostCallbacks(callbacks));
    core.set_engine(Arc::new(FfiEngine { cb: cb.clone(), caps }));
    let mut events = core.subscribe();
    let ev_cb = cb.clone();
    rt.spawn(async move {
        while let Ok(ev) = events.recv().await {
            ev_cb.send(ev_cb.0.core_event, &json!({"seq": ev.seq, "event": ev.event}));
        }
    });
    let ipc = match (flags & JS_OPEN_NO_IPC == 0, dir.as_ref(), in_memory) {
        (true, Some(d), false) => match rt.block_on(ipc::start(core.clone(), d)) {
            Ok(h) => Some(h),
            Err(e) => { eprintln!("justslide: agent IPC unavailable: {e:#}"); None }
        },
        _ => None,
    };
    Box::into_raw(Box::new(JsCore { rt, core, _ipc: ipc }))
}

/// Executes a chrome (human) command `{"id","command","args"}`. `reply` is
/// called exactly once, on a core thread, with the wire response.
///
/// # Safety
/// `core` must come from `justslide_core_open`; `request_json` a C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn justslide_core_execute(core: *mut JsCore, request_json: *const c_char, reply: JsCallback, reply_ctx: *mut c_void) {
    let Some(c) = (unsafe { core.as_ref() }) else { return };
    let req: Value = unsafe { cstr(request_json) }.and_then(|s| serde_json::from_str(s).ok()).unwrap_or(Value::Null);
    let request = Request {
        id: req["id"].as_str().unwrap_or("ui").chars().take(64).collect(),
        command: req["command"].as_str().unwrap_or_default().into(),
        args: req["args"].clone(),
        precondition: None,
    };
    let target = HostCallbacks(JsHostCallbacks { ctx: reply_ctx, engine_command: reply, core_event: reply });
    let core = c.core.clone();
    c.rt.spawn(async move {
        let resp = core.execute(&Actor::Human, request).await;
        target.send(reply, &resp.to_wire());
    });
}

/// Delivers a real WebKit callback (`EngineEvent` JSON) to the core.
///
/// # Safety
/// `core` must come from `justslide_core_open`; `event_json` a C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn justslide_core_engine_event(core: *mut JsCore, event_json: *const c_char) -> i32 {
    let Some(c) = (unsafe { core.as_ref() }) else { return -1 };
    match unsafe { cstr(event_json) }.map(serde_json::from_str::<EngineEvent>) {
        Some(Ok(ev)) => { c.core.on_engine_event(ev); 0 }
        _ => -2,
    }
}

/// Re-attaches the persisted session to a fresh adapter (active tab only).
///
/// # Safety
/// `core` must come from `justslide_core_open`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn justslide_core_restore_session(core: *mut JsCore) {
    if let Some(c) = unsafe { core.as_ref() } { c.core.restore_session(); }
}

/// DEVELOPMENT ONLY (feature `design-fixture`): loads the Figma reference
/// fixture. Returns -1 in builds without the feature.
///
/// # Safety
/// `core` must come from `justslide_core_open`; `variant` a C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn justslide_core_load_design_fixture(core: *mut JsCore, variant: *const c_char) -> i32 {
    #[cfg(feature = "design-fixture")]
    {
        let Some(c) = (unsafe { core.as_ref() }) else { return -1 };
        let v = unsafe { cstr(variant) }.unwrap_or("desktop");
        return if c.core.load_fixture(&json!({"variant": v})).is_ok() { 0 } else { -2 };
    }
    #[cfg(not(feature = "design-fixture"))]
    {
        let _ = (core, variant);
        -1
    }
}

/// Shuts down IPC and the runtime and frees the handle.
///
/// # Safety
/// `core` must come from `justslide_core_open` and not be used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn justslide_core_close(core: *mut JsCore) {
    if !core.is_null() {
        let b = unsafe { Box::from_raw(core) };
        let JsCore { rt, core: _, _ipc } = *b;
        drop(_ipc);
        rt.shutdown_timeout(std::time::Duration::from_secs(1));
    }
}
