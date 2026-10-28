import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import { DesktopService } from './service.js';

export const BUS_NAME = 'com.bloom.Desktop';
export const OBJECT_PATH = '/com/bloom/Desktop';
const XML = `<node><interface name="com.bloom.Desktop">
  <method name="GetVersion"><arg type="u" direction="out"/></method>
  <method name="GetPointer">
    <arg type="i" direction="out"/><arg type="i" direction="out"/>
    <arg type="u" direction="out"/><arg type="u" direction="out"/>
  </method>
  <method name="SetShortcutEnabled">
    <arg type="b" direction="in"/><arg type="b" direction="out"/>
  </method>
  <signal name="Trigger"><arg type="i"/><arg type="i"/><arg type="u"/><arg type="u"/></signal>
</interface></node>`;

export function exportDesktop({ capture, grab }) {
  let object;
  const service = new DesktopService({
    capture,
    grab,
    emit: (pointer) => object.emit_signal('Trigger', new GLib.Variant('(iiuu)', pointer)),
    watch: (sender, vanished) => {
      const id = Gio.bus_watch_name_on_connection(
        Gio.DBus.session,
        sender,
        Gio.BusNameWatcherFlags.NONE,
        null,
        vanished,
      );
      return () => Gio.bus_unwatch_name(id);
    },
  });
  object = Gio.DBusExportedObject.wrapJSObject(XML, {
    GetVersion: () => 1,
    GetPointer: () => service.pointer(),
    SetShortcutEnabledAsync: ([enabled], invocation) => {
      try {
        const registered = service.setEnabled(invocation.get_sender(), enabled);
        invocation.return_value(new GLib.Variant('(b)', [registered]));
      } catch (error) {
        invocation.return_dbus_error('com.bloom.Desktop.Error', String(error));
      }
    },
  });
  object.export(Gio.DBus.session, OBJECT_PATH);
  const name = Gio.bus_own_name_on_connection(
    Gio.DBus.session,
    BUS_NAME,
    Gio.BusNameOwnerFlags.NONE,
    null,
    null,
  );
  return () => {
    service.release();
    object.unexport();
    Gio.bus_unown_name(name);
  };
}
