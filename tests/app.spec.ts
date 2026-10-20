import { test, expect } from '@playwright/test';
import { readFileSync } from 'node:fs';

const catalog = JSON.parse(readFileSync(new URL('../core/catalog.json', import.meta.url), 'utf8'));
const fixture = Array.from(readFileSync(new URL('../core/assets/Anuphan.ttf', import.meta.url)));

test('browser preview is honest, search works, and installation is disabled', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('#notice')).toContainText('ตัวอย่างหน้าจอ');
  await expect(page.locator('.font-card')).toHaveCount(34);
  await page.getByRole('searchbox').fill('Google Sans');
  await expect(page.locator('.font-card')).toHaveCount(1);
  await page.locator('.font-card').click();
  await expect(page.locator('#family-name')).toHaveText('Google Sans');
  await page.getByRole('searchbox').fill('Kanit');
  await expect(page.locator('.font-card')).toHaveCount(1);
  await page.locator('.font-card').click();
  await expect(page.locator('#family-name')).toHaveText('Kanit');
  await expect(page.locator('#install')).toBeDisabled();
});

test('native bridge: preview is real, install needs confirmation, failures recover', async ({ page }) => {
  await page.addInitScript(({ catalog, fixture }) => {
    const state: any = window;
    state.calls = [];
    state.__TAURI_INTERNALS__ = { transformCallback: () => 1, invoke: async (cmd: string, args: any) => {
      state.calls.push({ cmd, args });
      if (cmd === 'catalog') return catalog;
      if (cmd === 'font_status') return [];
      if (cmd === 'preview_font') return new Uint8Array(fixture).buffer;
      if (cmd === 'install_font') throw new Error('Simulated offline download');
      throw new Error(cmd);
    } };
  }, { catalog, fixture });
  await page.goto('/');
  await expect(page.locator('#install')).toBeEnabled();
  await page.getByRole('searchbox').fill('Anuphan');
  await page.locator('.font-card').click();
  await expect(page.locator('#preview-state')).toHaveText('แสดงด้วย Anuphan จริง');
  await page.locator('#install').click();
  await expect(page.locator('#confirm-dialog')).toBeVisible();
  await page.locator('#cancel').click();
  expect(await page.evaluate(() => (window as any).calls.filter((c: any) => c.cmd === 'install_font').length)).toBe(0);
  await page.locator('#install').click();
  await page.locator('#proceed').click();
  await expect(page.locator('#notice')).toContainText('Simulated offline');
  await expect(page.locator('#install')).toBeEnabled();
  await expect(page.getByRole('searchbox')).toBeEnabled();
});

test('managed missing files remain accessible for cleanup', async ({ page }) => {
  await page.addInitScript(({ catalog, fixture }) => {
    (window as any).__TAURI_INTERNALS__ = { transformCallback: () => 1, invoke: async (cmd: string) => {
      if (cmd === 'catalog') return catalog;
      if (cmd === 'preview_font') return fixture;
      if (cmd === 'font_status') return [{ id: 'sarabun', installed: 0, changed: 0, total: 16, managed: 1 }];
      if (cmd === 'uninstall_font') return { count: 0, warning: '' };
      throw new Error(cmd);
    } };
  }, { catalog, fixture });
  await page.goto('/');
  await expect(page.locator('#uninstall')).toBeEnabled();
  await page.locator('[data-filter="installed"]').click();
  await expect(page.locator('.font-card')).toHaveCount(1);
});


test('automatic preview, live progress and consecutive installs stay responsive', async ({ page }) => {
  await page.addInitScript(({ catalog, fixture }) => {
    const state: any = window;
    state.installs = 0;
    state.__TAURI_INTERNALS__ = { transformCallback: () => 1, invoke: async (cmd: string, args: any) => {
      if (cmd === 'catalog') return catalog;
      if (cmd === 'font_status') return [];
      if (cmd === 'preview_font') return new Uint8Array(fixture).buffer;
      if (cmd === 'install_font') {
        args.onProgress.onmessage({phase:'download', done:1, total:3});
        await new Promise(r => setTimeout(r, 250));
        args.onProgress.onmessage({phase:'install', done:2, total:3});
        await new Promise(r => setTimeout(r, 250));
        state.installs++;
        return {count:3, warning:''};
      }
      throw new Error(cmd);
    }};
  }, { catalog, fixture });
  await page.goto('/');
  for (const name of ['Sarabun', 'Prompt', 'Kanit', 'Anuphan']) {
    await expect(page.locator('#install')).toBeEnabled();
    await page.getByRole('searchbox').fill(name);
    await page.locator('.font-card').first().click();
    await expect(page.locator('#preview-state')).toHaveText(`แสดงด้วย ${name} จริง`);
    await page.locator('#install').click();
    await page.locator('#proceed').click();
    await expect(page.locator('#work-status')).toBeVisible();
    await expect(page.locator('#install')).toBeDisabled();
    await expect(page.locator('#notice')).toContainText(`ติดตั้ง ${name} แล้ว`);
    await expect(page.locator('#work-status')).toBeHidden();
  }
  expect(await page.evaluate(() => (window as any).installs)).toBe(4);
  expect(await page.evaluate(() => document.fonts.size)).toBeLessThanOrEqual(2);
  await expect(page.locator('#install')).toHaveCSS('background-color', 'rgb(253, 106, 13)');
  await expect(page.locator('#install')).toHaveCSS('color', 'rgb(3, 26, 62)');
});
