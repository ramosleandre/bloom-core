export const uuid = 'bloom-desktop@bloom.local';

// A successful `gnome-extensions enable` does not prove that Shell loaded the service:
// the master extension switch can still be off. Check the actual production endpoint.
export function requireAdapter(run) {
  let failure;
  try {
    const version = run('gdbus', [
      'call',
      '--session',
      '--timeout=2',
      '--dest',
      'com.bloom.Desktop',
      '--object-path',
      '/com/bloom/Desktop',
      '--method',
      'com.bloom.Desktop.GetVersion',
    ]);
    if (/^\(uint32 1,\)\s*$/.test(version)) return;
    throw new Error(`Unsupported GNOME adapter protocol: ${version.trim()}`);
  } catch (error) {
    failure = error;
  }
  let disabled = false;
  let info;
  try {
    disabled =
      run('gsettings', ['get', 'org.gnome.shell', 'disable-user-extensions']).trim() === 'true';
    info = run('gnome-extensions', ['info', uuid]);
  } catch {
    // Missing/unavailable Shell tools are reported with the service error below.
  }
  if (disabled)
    throw new Error(
      'GNOME has disabled user extensions, so Bloom cannot receive desktop shortcuts or cursor coordinates.\n' +
        'Enable extensions in the Extensions app, or run:\n' +
        '  gsettings set org.gnome.shell disable-user-extensions false\n' +
        `  gnome-extensions enable ${uuid}\n` +
        'Then run make ui-gnome again.',
    );
  throw new Error(
    'Bloom Desktop Adapter is not running. Desktop-wide placement and shortcuts require it.\n' +
      (info
        ? `${info}\nRun: gnome-extensions enable ${uuid}\nIf its state is ERROR, inspect journalctl --user -b for Bloom errors.`
        : 'Run make gnome-setup. If GNOME has not discovered the extension, log out and back in once, then enable it.') +
      `\nService check: ${String(failure.stderr ?? failure.message).trim()}`,
  );
}
