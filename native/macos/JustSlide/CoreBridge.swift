import Foundation

/// Swift wrapper over the justslide-core C ABI (crates/justslide-ffi).
/// The core is the single authority for browser state; this class only moves
/// JSON across the boundary and hops callbacks onto the main queue.
final class CoreBridge {
    private var handle: OpaquePointer?
    /// EngineCommand JSON objects for the WebKit adapter (main queue).
    var onEngineCommand: (([String: Any]) -> Void)?
    /// Core event notifications (main queue); the chrome refreshes its snapshot.
    var onCoreEvent: ((Int, String) -> Void)?

    init?(dataDirectory: URL?, capabilities: [String: Any], inMemory: Bool = false, enableAgentIPC: Bool = true) {
        let caps = (try? JSONSerialization.data(withJSONObject: capabilities)).flatMap { String(data: $0, encoding: .utf8) } ?? "{}"
        let ctx = Unmanaged.passUnretained(self).toOpaque()
        let callbacks = JsHostCallbacks(ctx: ctx, engine_command: { ctx, json in
            guard let ctx, let json else { return }
            let text = String(cString: json)
            let me = Unmanaged<CoreBridge>.fromOpaque(ctx).takeUnretainedValue()
            DispatchQueue.main.async {
                if let data = text.data(using: .utf8), let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any] {
                    me.onEngineCommand?(obj)
                }
            }
        }, core_event: { ctx, json in
            guard let ctx, let json else { return }
            let text = String(cString: json)
            let me = Unmanaged<CoreBridge>.fromOpaque(ctx).takeUnretainedValue()
            DispatchQueue.main.async {
                guard let data = text.data(using: .utf8), let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else { return }
                me.onCoreEvent?(obj["seq"] as? Int ?? 0, obj["event"] as? String ?? "")
            }
        })
        var flags: UInt32 = 0
        if inMemory { flags |= UInt32(JS_OPEN_IN_MEMORY) }
        if !enableAgentIPC { flags |= UInt32(JS_OPEN_NO_IPC) }
        let opened: OpaquePointer? = dataDirectory?.path.withCString { justslide_core_open($0, caps, callbacks, flags) } ?? justslide_core_open(nil, caps, callbacks, flags)
        guard let opened else { return nil }
        handle = opened
    }

    deinit { if let handle { justslide_core_close(handle) } }

    private final class ReplyBox { let fn: ([String: Any]) -> Void; init(_ fn: @escaping ([String: Any]) -> Void) { self.fn = fn } }

    /// Runs a human (trusted chrome) command; `completion` runs on the main queue.
    func execute(id: String, command: String, args: Any, completion: @escaping ([String: Any]) -> Void) {
        guard let handle else { return }
        let payload: [String: Any] = ["id": id, "command": command, "args": args]
        guard let data = try? JSONSerialization.data(withJSONObject: payload), let text = String(data: data, encoding: .utf8) else {
            completion(["ok": false, "error": ["code": "invalid_argument", "message": "unserializable request", "retryable": false]])
            return
        }
        let box = Unmanaged.passRetained(ReplyBox(completion)).toOpaque()
        text.withCString { justslide_core_execute(handle, $0, { ctx, json in
            guard let ctx else { return }
            let box = Unmanaged<ReplyBox>.fromOpaque(ctx).takeRetainedValue()
            let text = json.map { String(cString: $0) } ?? "{}"
            DispatchQueue.main.async {
                let obj = (text.data(using: .utf8).flatMap { try? JSONSerialization.jsonObject(with: $0) } as? [String: Any]) ?? [:]
                box.fn(obj)
            }
        }, box) }
    }

    /// Forwards a real WebKit callback to the core.
    func send(event: [String: Any]) {
        guard let handle, let data = try? JSONSerialization.data(withJSONObject: event), let text = String(data: data, encoding: .utf8) else { return }
        let rc = text.withCString { justslide_core_engine_event(handle, $0) }
        if rc != 0 { NSLog("JustSlide: core rejected engine event %@ (%d)", event["type"] as? String ?? "?", rc) }
    }

    func restoreSession() { if let handle { justslide_core_restore_session(handle) } }

    /// DEBUG builds only: loads the Figma design fixture (returns false in release).
    @discardableResult
    func loadDesignFixture(variant: String) -> Bool {
        guard let handle else { return false }
        return variant.withCString { justslide_core_load_design_fixture(handle, $0) } == 0
    }
}
