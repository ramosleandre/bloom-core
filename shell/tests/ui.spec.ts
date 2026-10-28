import { test as base, expect } from '@playwright/test';
import { spawn } from 'node:child_process';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

type Session = { url: string; audit: string; stop: () => void };
const test = base.extend<{ session: Session }>({
  session: async ({}, use) => {
    const temporary = await mkdtemp(join(tmpdir(), 'bloom-ui-test-'));
    const audit = join(temporary, 'audit.jsonl');
    const server = spawn(
      'uv',
      ['run', 'python', '-u', '-m', 'bloom', '--ui', '--no-open', '--port', '9876'],
      {
        cwd: fileURLToPath(new URL('../../', import.meta.url)),
        env: { ...process.env, BLOOM_AUDIT_PATH: audit, BLOOM_DECISION_BACKENDS: 'rules' },
        stdio: ['ignore', 'pipe', 'pipe'],
      },
    );
    let output = '';
    let stderr = '';
    server.stderr.on('data', (chunk: Buffer) => {
      stderr += chunk.toString();
    });
    try {
      const url = await new Promise<string>((resolve, reject) => {
        const timer = setTimeout(
          () => reject(new Error(`Python shell did not start: ${stderr}`)),
          10_000,
        );
        server.stdout.on('data', (chunk: Buffer) => {
          output += chunk.toString();
          const match = output.match(/Open: (http:\/\/[^\s]+)/);
          if (match) {
            clearTimeout(timer);
            resolve(match[1]);
          }
        });
        server.once('error', (error) => {
          clearTimeout(timer);
          reject(error);
        });
        server.once('exit', (code) => {
          clearTimeout(timer);
          reject(new Error(`Python exited (${code}): ${stderr}`));
        });
      });
      await use({
        url,
        audit,
        stop: () => {
          server.kill('SIGTERM');
        },
      });
    } finally {
      if (server.exitCode === null && server.signalCode === null) {
        server.kill('SIGTERM');
        await new Promise<void>((resolve) => {
          server.once('exit', () => resolve());
        });
      }
      await rm(temporary, { recursive: true, force: true });
    }
  },
});

test('suggestions from Python can be selected and the outcome is audited', async ({
  page,
  session,
}, info) => {
  const errors: string[] = [];
  page.on('pageerror', (error) => errors.push(error.message));
  await page.goto(session.url);
  await expect(page.getByText('Core connected', { exact: true })).toBeVisible();
  await page.screenshot({
    path: info.outputPath('overview.png'),
    fullPage: true,
    animations: 'disabled',
  });
  await page.getByRole('button', { name: 'Start a demo' }).click();
  const menu = page.getByRole('dialog', { name: 'What would you like to do?' });
  await expect(menu.getByRole('button', { name: 'Draft a reply', exact: true })).toBeVisible();
  await expect(menu.getByRole('button', { name: 'Add to calendar', exact: true })).toBeVisible();
  await page.screenshot({ path: info.outputPath('pie-menu.png'), animations: 'disabled' });
  await menu.getByRole('button', { name: 'Summarize', exact: true }).click();
  await expect(menu).toBeHidden();
  const row = page.getByRole('button', { name: /Summarize.*Completed/ });
  await expect(row).toBeVisible();
  await row.click();
  await expect(page.getByText('text.summarize executed (mocked)', { exact: true })).toBeVisible();
  const audit = JSON.parse((await readFile(session.audit, 'utf8')).trim());
  expect(audit.chosen.action_id).toBe('text.summarize');
  expect(audit.status).toBe('done');
  expect(audit.timings_ms.menu_latency).toBeGreaterThan(0);
  expect(errors).toEqual([]);
});

for (const verdict of ['approve', 'reject'] as const) {
  test(`a human can ${verdict} the actual prepared action`, async ({ page, session }, info) => {
    await page.goto(session.url);
    await page.getByRole('button', { name: 'Start a demo' }).click();
    await page.getByRole('button', { name: 'Draft a reply', exact: true }).click();
    const approval = page.getByRole('dialog', { name: 'Ready when you are.' });
    await expect(approval.getByText('Draft a reply (mocked)', { exact: true })).toBeVisible();
    await expect(
      approval.getByText('the context contains untrusted third-party content', { exact: true }),
    ).toBeVisible();
    await page.screenshot({ path: info.outputPath('approval.png'), animations: 'disabled' });
    await approval
      .getByRole('button', {
        name: verdict === 'approve' ? 'Approve action' : 'Reject action',
        exact: true,
      })
      .click();
    await expect(approval).toBeHidden();
    const result = verdict === 'approve' ? 'Completed' : 'Cancelled';
    await expect(
      page.getByRole('button', { name: new RegExp(`Draft a reply.*${result}`) }),
    ).toBeVisible();
    const audit = JSON.parse((await readFile(session.audit, 'utf8')).trim());
    expect(audit.approval.verdict).toBe(verdict);
    expect(audit.result === null).toBe(verdict === 'reject');
  });
}

test('keyboard navigation, quick launch and Escape work without automatic approval', async ({
  page,
  session,
}) => {
  await page.goto(session.url);
  await expect(page.getByText('Core connected', { exact: true })).toBeVisible();
  await page.keyboard.press('Control+Space');
  await expect(page.getByRole('button', { name: 'Draft a reply', exact: true })).toBeFocused();
  await page.keyboard.press('ArrowDown');
  await expect(page.getByRole('button', { name: 'Summarize', exact: true })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('button', { name: /Summarize.*Completed/ })).toBeVisible();
  await page.getByRole('button', { name: 'Start a demo' }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).toBeHidden();
  await expect(page.getByRole('button', { name: /Menu dismissed.*Cancelled/ })).toBeVisible();
});

test('preferences persist and pie previews never execute an action', async ({ page, session }) => {
  await page.goto(session.url);
  await page.getByRole('button', { name: 'Start a demo' }).click();
  await page.getByRole('button', { name: 'Summarize', exact: true }).click();
  await expect(page.getByRole('dialog')).toBeHidden();
  await page.getByRole('button', { name: 'Preferences', exact: true }).click();
  await page.getByRole('button', { name: 'Lilac accent' }).click();
  await page.getByRole('checkbox', { name: 'Reduce motion' }).check();
  await page.reload();
  await page.getByRole('button', { name: 'Preferences', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Lilac accent' })).toHaveAttribute(
    'aria-pressed',
    'true',
  );
  await expect(page.getByRole('checkbox', { name: 'Reduce motion' })).toBeChecked();
  await page.getByRole('button', { name: 'Pie menu', exact: true }).click();
  await page.getByRole('button', { name: 'On the right' }).click();
  await page.getByRole('button', { name: 'Draft a reply', exact: true }).click();
  await expect(page.getByRole('status')).toContainText('Preview selection: Draft a reply');
  expect((await readFile(session.audit, 'utf8')).trim().split('\n')).toHaveLength(1);
});

test('a separate pie surface shares the live run with the main window', async ({
  page,
  context,
  session,
}) => {
  await page.goto(session.url);
  const pie = await context.newPage();
  const url = new URL(session.url);
  url.searchParams.set('view', 'pie');
  await pie.goto(url.toString());
  await expect(pie.locator('html')).toHaveAttribute('data-surface', 'pie');
  await page.getByRole('button', { name: 'Preferences', exact: true }).click();
  await page.getByRole('button', { name: 'Lilac accent' }).click();
  await expect(pie.locator('html')).toHaveAttribute('data-accent', 'lilac');
  await page.getByRole('button', { name: 'Activity', exact: true }).click();
  await page.getByRole('button', { name: 'Start a demo' }).click();
  const overlay = pie.getByRole('dialog', { name: 'What would you like to do?' });
  await expect(overlay).toBeVisible();
  await expect(overlay.locator('.dialog-heading')).toHaveCount(0);
  for (const selector of ['html', 'body', '.radial-dialog']) {
    expect(
      await pie.locator(selector).evaluate((node) => getComputedStyle(node).backgroundColor),
    ).toBe('rgba(0, 0, 0, 0)');
  }
  await pie.setViewportSize({ width: 480, height: 480 });
  const circle = await pie.locator('.pie-stage').boundingBox();
  expect(circle?.width).toBe(480);
  expect(circle?.height).toBe(480);
  await pie.getByRole('button', { name: 'Summarize', exact: true }).click();
  await expect(pie.getByRole('dialog')).toBeHidden();
  await expect(page.getByRole('button', { name: /Summarize.*Completed/ })).toBeVisible();
  await page.getByRole('button', { name: 'Start a demo' }).click();
  await expect(overlay).toBeVisible();
  await pie.keyboard.press('Escape');
  await expect(overlay).toBeHidden();
  await expect(page.getByRole('button', { name: /Menu dismissed.*Cancelled/ })).toHaveCount(1);
  await page.getByRole('button', { name: 'Start a demo' }).click();
  await expect(overlay).toBeVisible();
  await pie.mouse.click(4, 4);
  await expect(overlay).toBeHidden();
  await expect(page.getByRole('button', { name: /Menu dismissed.*Cancelled/ })).toHaveCount(2);
  await pie.close();
});

test('the interface remains usable at a narrow viewport', async ({ page, session }, info) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(session.url);
  await page.getByRole('button', { name: 'Start a demo' }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.screenshot({ path: info.outputPath('narrow-pie.png'), animations: 'disabled' });
  await page.getByRole('button', { name: 'Summarize', exact: true }).click();
  await expect(page.getByRole('button', { name: /Summarize.*Completed/ })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
});

test('a background native trigger preserves its anchor and keeps the bridge awake', async ({
  page,
  context,
  session,
}) => {
  await page.addInitScript(() => {
    const callbacks = new Map<number, (event: unknown) => void>();
    const listeners = new Map<string, number>();
    const calls: Array<{ command: string; args: Record<string, unknown> }> = [];
    let id = 0;
    Object.assign(window, {
      isTauri: true,
      __TAURI_INTERNALS__: {
        transformCallback: (callback: (event: unknown) => void) => {
          callbacks.set(++id, callback);
          return id;
        },
        invoke: async (command: string, args: Record<string, unknown> = {}) => {
          calls.push({ command, args });
          if (command === 'plugin:event|listen') {
            listeners.set(args.event as string, args.handler as number);
            return args.handler;
          }
          if (command === 'configure_shortcut')
            return { registered: args.enabled, message: 'Native test shortcut' };
          if (command === 'show_pie') return true;
        },
      },
      __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener: () => {} },
      bloomNativeTest: {
        calls,
        trigger: () => {
          const handler = listeners.get('bloom-trigger');
          if (handler) callbacks.get(handler)?.({ event: 'bloom-trigger', payload: null });
        },
      },
    });
  });
  const native = () =>
    page.evaluate(() => {
      const mock = (
        window as unknown as {
          bloomNativeTest: { calls: Array<{ command: string; args: Record<string, unknown> }> };
        }
      ).bloomNativeTest;
      return mock.calls;
    });
  await page.goto(session.url);
  await expect(page.getByText('Core connected', { exact: true })).toBeVisible();
  await expect
    .poll(async () =>
      (await native()).some(
        ({ command, args }) => command === 'configure_shortcut' && args.enabled,
      ),
    )
    .toBe(true);
  await expect(page.getByRole('button', { name: /Minimize/ })).toHaveCount(0);
  const pie = await context.newPage();
  const url = new URL(session.url);
  url.searchParams.set('view', 'pie');
  await pie.goto(url.toString());
  await expect
    .poll(async () =>
      pie.evaluate(async () => (await navigator.locks.query()).held?.map((lock) => lock.name)),
    )
    .toContain('bloom-bridge-main');
  // The native event carries a cursor anchor already captured while the main view is behind.
  await page.evaluate(() => {
    (window as unknown as { bloomNativeTest: { trigger: () => void } }).bloomNativeTest.trigger();
  });
  await expect(pie.getByRole('button', { name: 'Summarize', exact: true })).toBeVisible();
  expect((await native()).filter(({ command }) => command === 'prepare_pie')).toHaveLength(0);
  await pie.getByRole('button', { name: 'Summarize', exact: true }).click();
  await expect(page.getByRole('button', { name: /Summarize.*Completed/ })).toBeVisible();
  // A button invocation still captures a new desktop position.
  await page.getByRole('button', { name: 'Start a demo' }).click();
  await expect(pie.getByRole('button', { name: 'Summarize', exact: true })).toBeVisible();
  expect((await native()).filter(({ command }) => command === 'prepare_pie')).toHaveLength(1);
  await pie.getByRole('button', { name: 'Cancel action selection' }).click();
  await expect(page.getByRole('button', { name: /Menu dismissed.*Cancelled/ })).toBeVisible();
  await page.goto('about:blank');
  await expect
    .poll(async () =>
      pie.evaluate(async () => (await navigator.locks.query()).held?.map((lock) => lock.name)),
    )
    .not.toContain('bloom-bridge-main');
  expect((await readFile(session.audit, 'utf8')).trim().split('\n')).toHaveLength(2);
});
