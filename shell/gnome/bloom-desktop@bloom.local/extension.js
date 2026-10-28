import Meta from 'gi://Meta';
import Shell from 'gi://Shell';
import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import { exportDesktop } from './dbus.js';

export default class BloomDesktop extends Extension {
  enable() {
    this._dispose = exportDesktop({
      capture: () => {
        const [x, y] = global.get_pointer();
        return [x, y, Math.round(global.stage.width), Math.round(global.stage.height)];
      },
      grab: (trigger) => {
        const action = global.display.grab_accelerator('<Control>space', Meta.KeyBindingFlags.NONE);
        if (action === Meta.KeyBindingAction.NONE)
          throw new Error('Ctrl+Space is already used by another desktop shortcut.');
        const binding = Meta.external_binding_name_for_action(action);
        Main.wm.allowKeybinding(binding, Shell.ActionMode.NORMAL);
        const signal = global.display.connect('accelerator-activated', (_display, activated) => {
          if (activated === action) trigger();
        });
        return () => {
          global.display.disconnect(signal);
          global.display.ungrab_accelerator(action);
          Main.wm.allowKeybinding(binding, Shell.ActionMode.NONE);
        };
      },
    });
  }

  disable() {
    this._dispose?.();
    this._dispose = null;
  }
}
