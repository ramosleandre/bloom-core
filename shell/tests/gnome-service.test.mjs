import { test } from 'node:test';
import assert from 'node:assert/strict';
import { DesktopService } from '../gnome/bloom-desktop@bloom.local/service.js';
import { requireAdapter } from '../scripts/gnome-check.mjs';

function fixture() {
  const events = [];
  const state = { pointer: [500, 300, 1920, 1080], grabs: 0, releases: 0, unwatches: 0 };
  let activate;
  let vanished;
  const service = new DesktopService({
    capture: () => state.pointer,
    grab: (callback) => {
      state.grabs++;
      activate = callback;
      return () => state.releases++;
    },
    watch: (_owner, callback) => {
      vanished = callback;
      return () => state.unwatches++;
    },
    emit: (pointer) => events.push(pointer),
  });
  return { service, state, events, activate: () => activate(), vanished: () => vanished() };
}

test('shortcut captures the current desktop point at activation, outside the main window', () => {
  const f = fixture();
  assert.equal(f.service.setEnabled(':1.20', true), true);
  f.state.pointer = [1750, 950, 1920, 1080];
  f.activate();
  assert.deepEqual(f.events, [[1750, 950, 1920, 1080]]);
  assert.equal(f.service.setEnabled(':1.20', true), true);
  assert.equal(f.state.grabs, 1);
});

test('turning quick launch off releases the keybinding and connection watch', () => {
  const f = fixture();
  f.service.setEnabled(':1.20', true);
  assert.equal(f.service.setEnabled(':1.20', false), false);
  f.service.release();
  assert.equal(f.state.releases, 1);
  assert.equal(f.state.unwatches, 1);
  assert.equal(f.service.owner, null);
});

test('a disconnected app releases the shortcut so a new instance can acquire it', () => {
  const f = fixture();
  f.service.setEnabled(':1.20', true);
  f.vanished();
  assert.equal(f.state.releases, 1);
  assert.equal(f.service.setEnabled(':1.21', true), true);
});

test('another client cannot disable or replace the active shortcut owner', () => {
  const f = fixture();
  f.service.setEnabled(':1.20', true);
  assert.throws(() => f.service.setEnabled(':1.21', false), /already owns/);
  assert.throws(() => f.service.setEnabled(':1.21', true), /already owns/);
  assert.equal(f.service.owner, ':1.20');
  assert.equal(f.state.releases, 0);
});

test('registration failure does not leave a phantom shortcut owner', () => {
  const f = fixture();
  f.service.grab = () => {
    throw new Error('Shortcut busy');
  };
  assert.throws(() => f.service.setEnabled(':1.20', true), /busy/);
  assert.equal(f.service.owner, null);
});

test('launch check accepts a live adapter, rather than relying on extension enable status', () => {
  requireAdapter((command) => {
    assert.equal(command, 'gdbus');
    return '(uint32 1,)\n';
  });
});

test('launch check explains the master switch when an installed extension is disabled globally', () => {
  assert.throws(
    () =>
      requireAdapter((command) => {
        if (command === 'gdbus') throw new Error('The name is not activatable');
        if (command === 'gsettings') return 'true\n';
        return 'Enabled: No\nState: INITIALIZED\n';
      }),
    /gsettings set org.gnome.shell disable-user-extensions false/,
  );
});

test('launch check refuses an enabled extension whose service failed to load', () => {
  assert.throws(
    () =>
      requireAdapter((command) => {
        if (command === 'gdbus') throw new Error('The name is not activatable');
        if (command === 'gsettings') return 'false\n';
        return 'Enabled: Yes\nState: ERROR\n';
      }),
    /not running[\s\S]*State: ERROR/,
  );
});

test('launch check rejects an incompatible adapter protocol', () => {
  assert.throws(
    () =>
      requireAdapter((command) => {
        if (command === 'gdbus') return '(uint32 2,)\n';
        if (command === 'gsettings') return 'false\n';
        return 'Enabled: Yes\n';
      }),
    /Unsupported GNOME adapter protocol/,
  );
});
