// Sense source for GNOME Shell: id `linux.gnome`, channel `compositor`.
// Specification: sense/spec/platform-linux.md (family `gnome`) and sense/spec/02-sources.md.
//
// Rules this file keeps:
// - it connects OUT to the user's Sense socket and exports nothing (no D-Bus name, no shortcut);
// - it translates Shell objects into observations and contains no other logic;
// - ambient observations carry identity only; titles are sent only at the session-awareness
//   level or in answer to a read, which the service makes for a user-initiated capture;
// - clipboard content is never read here, only the kinds of data present.

import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Meta from 'gi://Meta';
import Shell from 'gi://Shell';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';

const RETRY_MS = [300, 1000, 3000, 10000];

function socketPath() {
    return GLib.getenv('SENSE_SOCKET') ??
        GLib.build_filenamev([GLib.get_user_runtime_dir(), 'sense', 'sense.sock']);
}

function clipKinds(mimes) {
    const kinds = new Set();
    for (const m of mimes) {
        if (m.startsWith('text/html')) kinds.add('html');
        else if (m.startsWith('text/uri-list') || m.includes('gnome-copied-files')) kinds.add('files');
        else if (m.startsWith('text/') || m === 'UTF8_STRING' || m === 'STRING') kinds.add('text');
        else if (m.startsWith('image/')) kinds.add('image');
        else kinds.add('other');
    }
    return [...kinds];
}

export default class SenseExtension extends Extension {
    enable() {
        this._alive = true;
        this._attempt = 0;
        this._level = 'on_request';
        this._focusId = global.display.connect('notify::focus-window', () => this._observeFocus());
        this._connect();
    }

    disable() {
        this._alive = false;
        if (this._focusId) global.display.disconnect(this._focusId);
        this._focusId = null;
        if (this._timer) GLib.source_remove(this._timer);
        this._timer = null;
        this._close();
    }

    _close() {
        try { this._conn?.close(null); } catch (_) { /* already closed */ }
        this._conn = null;
        this._out = null;
        this._ready = false;
    }

    _retry() {
        this._close();
        if (!this._alive || this._timer) return;
        const delay = RETRY_MS[Math.min(this._attempt++, RETRY_MS.length - 1)];
        this._timer = GLib.timeout_add(GLib.PRIORITY_DEFAULT, delay, () => {
            this._timer = null;
            this._connect();
            return GLib.SOURCE_REMOVE;
        });
    }

    _connect() {
        if (!this._alive) return;
        const address = new Gio.UnixSocketAddress({path: socketPath()});
        new Gio.SocketClient().connect_async(address, null, (client, res) => {
            try {
                this._conn = client.connect_finish(res);
            } catch (_) {
                this._retry();          // the service is not running: stay quiet, try later
                return;
            }
            this._out = this._conn.get_output_stream();
            this._in = new Gio.DataInputStream({base_stream: this._conn.get_input_stream()});
            this._send({jsonrpc: '2.0', id: 0, method: 'source.hello', params: {
                id: 'linux.gnome', version: String(this.metadata.version), channel: 'compositor',
                provides: ['apps', 'windows', 'focus', 'pointer', 'clipboard'],
            }});
            this._readLoop();
        });
    }

    _send(message) {
        if (!this._out) return;
        const bytes = new GLib.Bytes(new TextEncoder().encode(`${JSON.stringify(message)}\n`));
        this._out.write_bytes_async(bytes, GLib.PRIORITY_DEFAULT, null, (stream, res) => {
            try { stream.write_bytes_finish(res); } catch (_) { this._retry(); }
        });
    }

    _readLoop() {
        const input = this._in;
        input.read_line_async(GLib.PRIORITY_DEFAULT, null, (stream, res) => {
            let line = null;
            try { [line] = stream.read_line_finish_utf8(res); } catch (_) { line = null; }
            if (line === null) {
                if (input === this._in) this._retry();
                return;
            }
            try { this._handle(JSON.parse(line)); } catch (_) { /* ignore a malformed message */ }
            if (input === this._in) this._readLoop();
        });
    }

    _handle(message) {
        if (message.id === 0 && message.result) {          // welcome
            this._attempt = 0;
            this._ready = true;
            this._level = message.result.config?.level ?? 'on_request';
            this._observeFocus();
        } else if (message.method === 'source.read') {
            const {request_id, signals} = message.params;
            this._send({jsonrpc: '2.0', method: 'source.result',
                params: {request_id, patches: this._current(signals ?? [])}});
        }
    }

    // ── Translating Shell objects ────────────────────────────────────────────────────────

    _describe(window, withTitle) {
        const app = Shell.WindowTracker.get_default().get_window_app(window);
        const rawId = (app?.get_id() ?? '').replace(/\.desktop$/, '') ||
            window.get_sandboxed_app_id?.() || window.get_gtk_application_id?.() ||
            window.get_wm_class() || '';
        const pid = window.get_pid();
        const key = String(window.get_id());
        return {key, patches: [
            {type: 'app_seen', pid, raw_id: rawId, name: app?.get_name() ?? rawId},
            {type: 'window_seen', key, pid, title: withTitle ? window.get_title() : null,
                fullscreen: window.is_fullscreen()},
        ]};
    }

    _observeFocus() {
        if (!this._ready) return;
        const window = global.display.focus_window;
        if (!window) {
            this._send({jsonrpc: '2.0', method: 'source.observe',
                params: {patches: [{type: 'focus_changed', key: null}]}});
            return;
        }
        const {key, patches} = this._describe(window, this._level === 'session_awareness');
        patches.push({type: 'focus_changed', key});
        this._send({jsonrpc: '2.0', method: 'source.observe', params: {patches}});
    }

    // Answer to a read: made by the service for a capture the user started.
    _current(signals) {
        const patches = [];
        const window = global.display.focus_window;
        if (window && signals.includes('windows'))
            patches.push(...this._describe(window, true).patches);
        if (signals.includes('pointer')) {
            const [x, y] = global.get_pointer();
            patches.push({type: 'pointer', x, y});
        }
        if (signals.includes('clipboard')) {
            const mimes = global.display.get_selection().get_mimetypes(Meta.SelectionType.SELECTION_CLIPBOARD);
            if (mimes.length)
                patches.push({type: 'clipboard', kinds: clipKinds(mimes), size: 0});
        }
        return patches;
    }
}
