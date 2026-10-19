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
    state.__TAURI_INTERNALS__ = { invoke: async (cmd: string, args: any) => {
      state.calls.push({ cmd, args });
      if (cmd === 'catalog') return catalog;
      if (cmd === 'font_status') return [];
      if (cmd === 'preview_font') return fixture;
      if (cmd === 'install_font') throw new Error('Simulated offline download');
      throw new Error(cmd);
    } };
  }, { catalog, fixture });
  await page.goto('/');
  await expect(page.locator('#install')).toBeEnabled();
  await page.getByRole('searchbox').fill('Anuphan');
  await page.locator('.font-card').click();
  await page.locator('#preview').click();
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
  await page.addInitScript(({ catalog }) => {
    (window as any).__TAURI_INTERNALS__ = { invoke: async (cmd: string) => {
      if (cmd === 'catalog') return catalog;
      if (cmd === 'font_status') return [{ id: 'sarabun', installed: 0, changed: 0, total: 16, managed: 1 }];
      if (cmd === 'uninstall_font') return { count: 0, warning: '' };
      throw new Error(cmd);
    } };
  }, { catalog });
  await page.goto('/');
  await expect(page.locator('#uninstall')).toBeEnabled();
  await page.locator('[data-filter="installed"]').click();
  await expect(page.locator('.font-card')).toHaveCount(1);
});
