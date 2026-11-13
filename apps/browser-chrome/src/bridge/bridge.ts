import { createContext, useContext, useSyncExternalStore } from 'react';
import type { UiCommand, UiCommandArgs } from '../generated/protocol';
import type { Snapshot } from './types';

/** The trusted chrome's only path to browser state: typed core commands. */
export interface ChromeBridge {
  execute<C extends UiCommand>(command: C, args: UiCommandArgs[C]): Promise<unknown>;
  /** Current projection; re-fetched by the bridge after every core event. */
  getSnapshot(): Snapshot | null;
  subscribe(listener: () => void): () => void;
  /** Reports the page-host rectangle so the native host can place the WKWebView. */
  reportContentBounds(rect: { x: number; y: number; width: number; height: number; radius: number }): void;
  readonly kind: 'native' | 'fixture';
}

export const BridgeContext = createContext<ChromeBridge | null>(null);

export function useBridge(): ChromeBridge {
  const b = useContext(BridgeContext);
  if (!b) throw new Error('BridgeContext missing');
  return b;
}

export function useSnapshot(): Snapshot | null {
  const b = useBridge();
  return useSyncExternalStore(b.subscribe, b.getSnapshot, b.getSnapshot);
}
