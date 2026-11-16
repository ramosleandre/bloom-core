// Test only: a headless GNOME Shell starts in the overview with nothing focused, and its seat has
// no keyboard, so applications never learn that they have keyboard focus.
import Clutter from 'gi://Clutter';
import GLib from 'gi://GLib';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

export default class Helper extends Extension {
    enable() {
        const seat = Clutter.get_default_backend().get_default_seat();
        this._keyboard = seat.create_virtual_device(Clutter.InputDeviceType.KEYBOARD_DEVICE);
        this._id = global.display.connect('window-created', (_d, window) => {
            GLib.timeout_add(GLib.PRIORITY_DEFAULT, 300, () => {
                Main.overview.hide();
                Main.activateWindow(window);
                // A seat only announces its keyboard once a key has been seen: tap Shift, so the
                // application receives keyboard focus as it would with a real keyboard.
                GLib.timeout_add(GLib.PRIORITY_DEFAULT, 200, () => {
                    const t = Clutter.get_current_event_time() * 1000;
                    this._keyboard.notify_key(t, 42, Clutter.KeyState.PRESSED);
                    this._keyboard.notify_key(t, 42, Clutter.KeyState.RELEASED);
                    return GLib.SOURCE_REMOVE;
                });
                return GLib.SOURCE_REMOVE;
            });
        });
    }
    disable() {
        if (this._id) global.display.disconnect(this._id);
        this._id = null;
        this._keyboard = null;
    }
}
