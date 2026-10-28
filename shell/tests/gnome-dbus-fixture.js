// Runs only under dbus-run-session. No Shell, desktop keybindings or pointer access.
import GLib from 'gi://GLib';
import Gio from 'gi://Gio';
import { exportDesktop } from '../gnome/bloom-desktop@bloom.local/dbus.js';

let pointer = [1500, 700, 1920, 1080];
let fire;
const dispose = exportDesktop({
  capture: () => pointer,
  grab: (activate) => {
    fire = activate;
    let live = true;
    GLib.timeout_add(GLib.PRIORITY_DEFAULT, 50, () => {
      if (live) {
        pointer = [1740, 900, 1920, 1080];
        activate();
      }
      return GLib.SOURCE_REMOVE;
    });
    return () => {
      live = false;
    };
  },
});
// Test-only input for the native smoke harness; this file is not in the extension package.
const control = Gio.DBusExportedObject.wrapJSObject(
  '<node><interface name="com.bloom.Desktop.Test"><method name="Fire"/></interface></node>',
  { Fire: () => fire?.() },
);
control.export(Gio.DBus.session, '/com/bloom/Desktop/Test');
try {
  new GLib.MainLoop(null, false).run();
} finally {
  control.unexport();
  dispose();
}
