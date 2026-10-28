import { useCallback, useEffect, useRef, useState } from 'react';
import { isTauri } from '@tauri-apps/api/core';
import type {
  ApprovalCommand,
  BridgeState,
  ChoiceCommand,
  ErrorEvent,
  StartCommand,
  StateEvent,
} from './contracts.generated';

export type Connection = 'connecting' | 'connected' | 'disconnected' | 'missing';

export function useBridge() {
  const [state, setState] = useState<BridgeState>({
    phase: 'idle',
    history: [],
    plan: null,
    approval: null,
    history_warning: null,
  });
  const [connection, setConnection] = useState<Connection>('connecting');
  const [error, setError] = useState<string | null>(null);
  const socket = useRef<WebSocket | null>(null);

  useEffect(() => {
    if (!isTauri() || !navigator.locks) return;
    const controller = new AbortController();
    let release: (() => void) | undefined;
    const view = new URLSearchParams(location.search).get('view') ?? 'main';
    // A pending Web Lock keeps the desktop bridge eligible for background work. Without
    // it, WebKit may suspend a minimized view, including its shortcut/state handlers.
    navigator.locks
      .request(
        `bloom-bridge-${view}`,
        { signal: controller.signal },
        () =>
          new Promise<void>((resolve) => {
            release = resolve;
          }),
      )
      .catch(() => {});
    return () => {
      release?.();
      controller.abort();
    };
  }, []);

  useEffect(() => {
    const params = new URLSearchParams(location.hash.slice(1));
    const token = params.get('token') ?? sessionStorage.getItem('bloom.launch-token');
    if (!token) {
      setConnection('missing');
      return;
    }
    sessionStorage.setItem('bloom.launch-token', token);
    history.replaceState(null, '', location.pathname + location.search);
    let stopped = false;
    let timer: ReturnType<typeof setTimeout> | undefined;
    function connect() {
      setConnection('connecting');
      const host = location.port === '1420' ? '127.0.0.1:8765' : location.host;
      const ws = new WebSocket(`ws://${host}/api/ws`);
      socket.current = ws;
      ws.onopen = () => ws.send(JSON.stringify({ type: 'authenticate', token }));
      ws.onmessage = (message) => {
        const event = JSON.parse(message.data) as StateEvent | ErrorEvent;
        if (event.type === 'state') {
          setConnection('connected');
          setState(event.state);
        } else {
          setError(event.message);
        }
      };
      ws.onclose = (event) => {
        if (stopped) return;
        setConnection('disconnected');
        setState((previous) => ({ ...previous, phase: 'idle', plan: null, approval: null }));
        if (event.code === 1008) {
          setError('This launch link has expired. Open the new link printed by make ui.');
        } else {
          timer = setTimeout(connect, 1500);
        }
      };
      ws.onerror = () => ws.close();
    }
    connect();
    return () => {
      stopped = true;
      clearTimeout(timer);
      socket.current?.close();
    };
  }, []);

  const send = useCallback((message: StartCommand | ChoiceCommand | ApprovalCommand) => {
    if (socket.current?.readyState !== WebSocket.OPEN) {
      setError('Bloom is disconnected. Start the Python core and reconnect.');
      return false;
    }
    setError(null);
    socket.current.send(JSON.stringify(message));
    return true;
  }, []);

  return { state, connection, error, dismissError: () => setError(null), send };
}
