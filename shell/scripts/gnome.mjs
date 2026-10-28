import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { setTimeout } from 'node:timers/promises';
import { requireAdapter, uuid } from './gnome-check.mjs';

const source = fileURLToPath(new URL(`../gnome/${uuid}/`, import.meta.url));
const output = fileURLToPath(new URL('../../.bloom/gnome/', import.meta.url));
const archive = `${output}${uuid}.shell-extension.zip`;
const action = process.argv[2] ?? 'status';
function run(command, args) {
  return execFileSync(command, args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
}
function pack() {
  mkdirSync(output, { recursive: true });
  run('gnome-extensions', [
    'pack',
    '--force',
    `--out-dir=${output}`,
    '--extra-source=dbus.js',
    '--extra-source=service.js',
    source,
  ]);
  console.log(`Extension package: ${archive}`);
}
try {
  if (action === 'pack') pack();
  else if (action === 'install') {
    const version = run('gnome-shell', ['--version']).match(/\b(\d+)\.\d+/)?.[1];
    const metadata = JSON.parse(readFileSync(`${source}metadata.json`, 'utf8'));
    if (!metadata['shell-version'].includes(version))
      throw new Error(
        `This adapter targets GNOME ${metadata['shell-version'].join(', ')}; detected ${version ?? 'an unknown version'}.`,
      );
    pack();
    run('gnome-extensions', ['install', '--force', archive]);
    console.log('Installed Bloom Desktop Adapter for this user.');
    try {
      run('gnome-extensions', ['enable', uuid]);
    } catch (error) {
      if (run('gnome-extensions', ['list']).split('\n').includes(uuid)) throw error;
      console.log(
        'GNOME has not discovered the new extension yet. Log out and back in once, then run:',
      );
      console.log(`  gnome-extensions enable ${uuid}`);
      console.log('  make ui-gnome');
      process.exit(0);
    }
    // Shell owns its bus name asynchronously after enable().
    await setTimeout(300);
    requireAdapter(run);
    console.log('The adapter is running. Restart Bloom with make ui-gnome.');
  } else if (action === 'remove') {
    run('gnome-extensions', ['uninstall', uuid]);
    console.log('Removed Bloom Desktop Adapter.');
  } else if (action === 'status' || action === 'check') {
    requireAdapter(run);
    if (action === 'status') console.log(run('gnome-extensions', ['info', uuid]));
    console.log('Bloom GNOME adapter is running (protocol 1).');
  } else {
    throw new Error('Usage: node scripts/gnome.mjs pack|install|remove|status|check');
  }
} catch (error) {
  console.error(String(error.stderr ?? error.message));
  process.exitCode = 1;
}
