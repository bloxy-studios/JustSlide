# ADR 0001 — Desktop host: AppKit-owned host (proposed), Tauri v2 evaluated

Status: **Proposed — needs owner approval** (spec §4.1 requires approval for a host change). Date 2026-11-13.

Context: the spec's baseline is Tauri v2 with a Swift/AppKit `WKWebView` plugin, gated by a native feasibility spike. Requirements that decide the host: independent page `WKWebView`s per tab with chosen `WKWebsiteDataStore(forIdentifier:)`, strict first-responder control, `takeSnapshot`, `WKWebExtensionController`, native sheets/menus above pages, and a buildable `.app` through the only Mac path available (SimBridge → `xcodebuild`, no shell).

Findings (documentation/source level; the native spike has **not run**):
- Tauri v2 multi-webview (`Window::add_child`) is behind the `unstable` feature; page webviews would be wry-managed, so data-store selection, snapshots, first-responder and extension controllers require reaching into raw `WKWebView` handles from Rust via Objective-C runtime calls (new `unsafe` code the spec asks us to avoid without necessity).
- Tauri desktop apps are built by `cargo tauri build`, not `xcodebuild`; SimBridge cannot build them.
- Every browser-critical behaviour is a first-class AppKit/WebKit API in Swift.

Decision (proposed): AppKit-owned native host that embeds the same React/ADS chrome in a trusted local `WKWebView` (served by a custom URL scheme) and links the same Rust core through a JSON C ABI. Still system WebKit — no Chromium. Tauri remains a documented alternative; app logic and UI are host-agnostic (`BrowserEngine` trait, bridge interface).

Consequences: Xcode project in `native/macos`; Rust built by an Xcode run-script phase; chrome↔page overlap is solved by keeping chrome overlays inside the sidebar and using native sheets/menus over pages.

Verification still required: build and run `JustSlideTests` (focus, profile isolation, bridge isolation, background snapshot) on the owner's Mac.
