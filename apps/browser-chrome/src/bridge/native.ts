// Bridge to the macOS host. The chrome runs in a privileged, local-only
// WKWebView whose script message handler (`justslide`) exists ONLY in the
// chrome's content world; external pages run in separate WKWebViews with no
// handler at all (native/macos/JustSlide/ChromeBridge.swift).
import type { ChromeBridge } from './bridge';
import type { Snapshot } from './types';

interface WebKitHandlers { justslide?: { postMessage(m: unknown): Promise<unknown> } }
declare global {
  interface Window {
    webkit?: { messageHandlers: WebKitHandlers };
    __justslideEvent?: (seq: number) => void;
  }
}

export function createNativeBridge(): ChromeBridge {
  const handler = window.webkit?.messageHandlers.justslide;
  if (!handler) throw new Error('JustSlide native bridge unavailable');
  let snapshot: Snapshot | null = null;
  const listeners = new Set<() => void>();
  let refreshing: Promise<void> | null = null;
  let dirty = false;
  let nextId = 1;

  const call = async (command: string, args: unknown) => {
    const reply = (await handler.postMessage({ id: `ui-${nextId++}`, command, args })) as { ok: boolean; result?: unknown; error?: unknown };
    if (!reply?.ok) throw reply?.error ?? new Error('bridge failure');
    return reply.result;
  };

  // Coalesce bursts of core events into one snapshot fetch (bounded chatter).
  const refresh = async (): Promise<void> => {
    if (refreshing) { dirty = true; return refreshing; }
    refreshing = (async () => {
      do {
        dirty = false;
        snapshot = (await call('state.snapshot', {})) as Snapshot;
        listeners.forEach((l) => l());
      } while (dirty);
    })().finally(() => { refreshing = null; });
    return refreshing;
  };
  window.__justslideEvent = () => { void refresh(); };
  void refresh();

  return {
    kind: 'native',
    execute: (command, args) => call(command, args),
    getSnapshot: () => snapshot,
    subscribe: (l) => { listeners.add(l); return () => listeners.delete(l); },
    // Layout message for the native host (not a core command).
    reportContentBounds: (rect) => { void handler.postMessage({ kind: 'contentBounds', rect }).catch(() => {}); },
  };
}
