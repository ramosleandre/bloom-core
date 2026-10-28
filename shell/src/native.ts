import { useEffect, useRef, useState } from 'react';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export const shortcutLabel = navigator.platform.includes('Mac') ? '⌘ ⇧ space' : 'Ctrl + Space';
type ShortcutStatus = { registered: boolean; message: string };

export function useNativeShortcut(
  start: (prepared?: boolean) => void,
  enabled: boolean,
  mainWindow: boolean,
) {
  const trigger = useRef(start);
  trigger.current = start;
  const [status, setStatus] = useState<ShortcutStatus>({
    registered: false,
    message: 'Quick launch works while Bloom is focused.',
  });
  useEffect(() => {
    if (!isTauri() || !mainWindow) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    // Listen before registration so a shortcut cannot arrive before its receiver is ready.
    listen('bloom-trigger', () => trigger.current(true))
      .then(async (remove) => {
        if (disposed) {
          remove();
          return;
        }
        unlisten = remove;
        const next = await invoke<ShortcutStatus>('configure_shortcut', { enabled });
        if (!disposed) setStatus(next);
      })
      .catch((error: unknown) => {
        if (!disposed)
          setStatus({
            registered: false,
            message: `Global shortcut unavailable: ${String(error)}. Quick launch works while Bloom is focused.`,
          });
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [enabled, mainWindow]);
  return status;
}
