// Test only: a headless GNOME Shell starts in the overview with nothing focused.
import GLib from 'gi://GLib';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

export default class Helper extends Extension {
    enable() {
        this._id = global.display.connect('window-created', (_d, window) => {
            GLib.timeout_add(GLib.PRIORITY_DEFAULT, 300, () => {
                Main.overview.hide();
                Main.activateWindow(window);
                return GLib.SOURCE_REMOVE;
            });
        });
    }
    disable() {
        if (this._id) global.display.disconnect(this._id);
        this._id = null;
    }
}
