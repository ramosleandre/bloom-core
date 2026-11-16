import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import St from 'gi://St';
import Clutter from 'gi://Clutter';
import Meta from 'gi://Meta';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Config from 'resource:///org/gnome/shell/misc/config.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

export default class SpikeExtension extends Extension {
    enable() { this._alive = true; this._connect(); }
    disable() {
        this._alive = false;
        if (this._timer) { GLib.source_remove(this._timer); this._timer = null; }
        try { this._conn?.close(null); } catch (_) {}
        this._conn = null;
    }
    _connect() {
        if (!this._alive) return;
        const path = GLib.getenv('SENSE_SPIKE_SOCK');
        const client = new Gio.SocketClient();
        client.connect_async(new Gio.UnixSocketAddress({path}), null, (c, res) => {
            try {
                this._conn = c.connect_finish(res);
                this._out = this._conn.get_output_stream();
                this._in = new Gio.DataInputStream({base_stream: this._conn.get_input_stream()});
                this._send({hello: 'spike', shell: Config.PACKAGE_VERSION});
                this._readLoop();
            } catch (e) { console.error('SPIKE connect failed: ' + e); this._retry(); }
        });
    }
    _send(obj) {
        const bytes = new GLib.Bytes(new TextEncoder().encode(JSON.stringify(obj) + '\n'));
        this._out.write_bytes_async(bytes, GLib.PRIORITY_DEFAULT, null, (s, res) => {
            try { s.write_bytes_finish(res); } catch (e) { this._retry(); }
        });
    }
    _readLoop() {
        this._in.read_line_async(GLib.PRIORITY_DEFAULT, null, (s, res) => {
            let line = null;
            try { [line] = s.read_line_finish_utf8(res); } catch (e) { line = null; }
            if (line === null) { console.error('SPIKE eof, retrying'); this._conn = null; this._retry(); return; }
            try { this._handle(JSON.parse(line)); } catch (e) { this._send({error: String(e)}); }
            this._readLoop();
        });
    }
    _handle(msg) {
        const t0 = GLib.get_monotonic_time();
        if (msg.cmd === 'focus') {
            const w = global.display.focus_window;
            const [x, y] = global.get_pointer();
            this._send({id: msg.id, title: w?.get_title() ?? null, wm_class: w?.get_wm_class() ?? null,
                pid: w?.get_pid() ?? null, gtk_app_id: w?.get_gtk_application_id?.() ?? null,
                sandboxed_app_id: w?.get_sandboxed_app_id?.() ?? null, pointer: [x, y],
                windows: global.get_window_actors().length, us: GLib.get_monotonic_time() - t0});
        } else if (msg.cmd === 'primary' || msg.cmd === 'clipboard') {
            const type = msg.cmd === 'primary' ? St.ClipboardType.PRIMARY : St.ClipboardType.CLIPBOARD;
            St.Clipboard.get_default().get_text(type, (_cb, text) => {
                this._send({id: msg.id, text, us: GLib.get_monotonic_time() - t0});
            });
        } else if (msg.cmd === 'prep') {
            Main.overview.hide();
            const actors = global.get_window_actors();
            const w = actors.length ? actors[actors.length - 1].meta_window : null;
            if (w) Main.activateWindow(w);
            this._send({id: msg.id, activated: w?.get_title() ?? null, overview: Main.overview.visible});
        } else if (msg.cmd === 'paste') {
            St.Clipboard.get_default().set_text(St.ClipboardType.CLIPBOARD, msg.text);
            const seat = Clutter.get_default_backend().get_default_seat();
            this._kbd ??= seat.create_virtual_device(Clutter.InputDeviceType.KEYBOARD_DEVICE);
            const key = (code, down) => this._kbd.notify_key(Clutter.get_current_event_time() * 1000, code,
                down ? Clutter.KeyState.PRESSED : Clutter.KeyState.RELEASED);
            GLib.timeout_add(GLib.PRIORITY_DEFAULT, msg.delay_ms ?? 50, () => {
                key(29, true); key(47, true); key(47, false); key(29, false);     // Ctrl+V (evdev codes)
                GLib.timeout_add(GLib.PRIORITY_DEFAULT, 200, () => {
                    key(28, true); key(28, false);                                 // Enter
                    this._send({id: msg.id, pasted: true, us: GLib.get_monotonic_time() - t0});
                    return GLib.SOURCE_REMOVE;
                });
                return GLib.SOURCE_REMOVE;
            });
        } else if (msg.cmd === 'keys') {
            const seat = Clutter.get_default_backend().get_default_seat();
            this._kbd ??= seat.create_virtual_device(Clutter.InputDeviceType.KEYBOARD_DEVICE);
            for (const [code, down] of msg.keys)
                this._kbd.notify_key(Clutter.get_current_event_time() * 1000, code,
                    down ? Clutter.KeyState.PRESSED : Clutter.KeyState.RELEASED);
            this._send({id: msg.id, sent: msg.keys.length});
        } else if (msg.cmd === 'mimes') {
            const sel = global.display.get_selection();
            this._send({id: msg.id,
                primary: sel.get_mimetypes(Meta.SelectionType.SELECTION_PRIMARY),
                clipboard: sel.get_mimetypes(Meta.SelectionType.SELECTION_CLIPBOARD)});
        } else if (msg.cmd === 'vkbd') {
            const seat = Clutter.get_default_backend().get_default_seat();
            const dev = seat.create_virtual_device(Clutter.InputDeviceType.KEYBOARD_DEVICE);
            this._send({id: msg.id, virtual_keyboard: dev !== null, type: String(dev)});
        } else {
            this._send({id: msg.id, error: 'unknown cmd'});
        }
    }
    _retry() {
        if (!this._alive || this._timer) return;
        this._timer = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 300, () => {
            this._timer = null; this._connect(); return GLib.SOURCE_REMOVE;
        });
    }
}
