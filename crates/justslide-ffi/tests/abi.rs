//! Exercises the C ABI exactly as the Swift host does (C callbacks, JSON).
use justslide_ffi::*;
use std::ffi::{CStr, CString, c_char, c_void};
use std::sync::Mutex;

static ENGINE: Mutex<Vec<String>> = Mutex::new(Vec::new());
static REPLIES: Mutex<Vec<String>> = Mutex::new(Vec::new());
extern "C" fn on_engine(_: *mut c_void, j: *const c_char) { ENGINE.lock().unwrap().push(unsafe { CStr::from_ptr(j) }.to_string_lossy().into()); }
extern "C" fn on_event(_: *mut c_void, _: *const c_char) {}
extern "C" fn on_reply(_: *mut c_void, j: *const c_char) { REPLIES.lock().unwrap().push(unsafe { CStr::from_ptr(j) }.to_string_lossy().into()); }

fn wait(n: usize) { for _ in 0..200 { if REPLIES.lock().unwrap().len() >= n { return; } std::thread::sleep(std::time::Duration::from_millis(10)); } panic!("no reply"); }

#[test]
fn chrome_commands_round_trip_through_the_c_abi() {
    let caps = CString::new(r#"{"engine":"webkit","engineVersion":"test","screenshots":{"support":"supported","note":""},"backgroundCapture":{"support":"partial","note":""},"pageSnapshot":{"support":"supported","note":""},"profileIsolation":{"support":"supported","note":""},"privateProfiles":{"support":"supported","note":""},"downloads":{"support":"unknown","note":""},"webInspector":{"support":"supported","note":""},"extensions":{"support":"unknown","note":""},"tabSuspension":{"support":"unknown","note":""},"pageAutomation":{"support":"unsupported","note":""}}"#).unwrap();
    let cb = JsHostCallbacks { ctx: std::ptr::null_mut(), engine_command: on_engine, core_event: on_event };
    let core = unsafe { justslide_core_open(std::ptr::null(), caps.as_ptr(), cb, JS_OPEN_IN_MEMORY | JS_OPEN_NO_IPC) };
    assert!(!core.is_null());
    let req = CString::new(r#"{"id":"ui-1","command":"tabs.open","args":{"input":"figma.com"}}"#).unwrap();
    unsafe { justslide_core_execute(core, req.as_ptr(), on_reply, std::ptr::null_mut()) };
    wait(1);
    let r: serde_json::Value = serde_json::from_str(&REPLIES.lock().unwrap()[0]).unwrap();
    assert_eq!(r["ok"], true, "{r}");
    let tab = r["result"]["tab"]["id"].as_str().unwrap().to_string();
    let cmds = ENGINE.lock().unwrap().clone();
    assert!(cmds.iter().any(|c| c.contains("\"createView\"") && c.contains("https://figma.com/")), "{cmds:?}");
    assert!(cmds.iter().any(|c| c.contains("\"show\"") && c.contains(&tab) && c.contains("\"focus\":true")));
    let nav = cmds.iter().find(|c| c.contains("createView")).and_then(|c| serde_json::from_str::<serde_json::Value>(c).ok()).unwrap()["navigationId"].as_str().unwrap().to_string();
    for ev in [
        format!(r#"{{"type":"navigationCommitted","tabId":"{tab}","navigationId":"{nav}","url":"https://www.figma.com/","secure":true,"canGoBack":false,"canGoForward":false}}"#),
        format!(r#"{{"type":"navigationFinished","tabId":"{tab}","navigationId":"{nav}","title":"Figma"}}"#),
    ] {
        let e = CString::new(ev).unwrap();
        assert_eq!(unsafe { justslide_core_engine_event(core, e.as_ptr()) }, 0);
    }
    let bad = CString::new("{\"type\":\"nope\"}").unwrap();
    assert_eq!(unsafe { justslide_core_engine_event(core, bad.as_ptr()) }, -2);
    let snap = CString::new(r#"{"id":"ui-2","command":"state.snapshot","args":{}}"#).unwrap();
    unsafe { justslide_core_execute(core, snap.as_ptr(), on_reply, std::ptr::null_mut()) };
    wait(2);
    let s: serde_json::Value = serde_json::from_str(&REPLIES.lock().unwrap()[1]).unwrap();
    let t = s["result"]["tabs"].as_array().unwrap().iter().find(|t| t["id"] == tab.as_str()).unwrap().clone();
    assert_eq!(t["title"], "Figma");
    assert_eq!(t["loadState"], "loaded");
    assert_eq!(t["secure"], true);
    // Release builds cannot load the design fixture.
    let v = CString::new("desktop").unwrap();
    let rc = unsafe { justslide_core_load_design_fixture(core, v.as_ptr()) };
    assert_eq!(rc, if cfg!(feature = "design-fixture") { 0 } else { -1 });
    unsafe { justslide_core_close(core) };
}
