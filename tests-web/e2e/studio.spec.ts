import { test, expect } from '@playwright/test';
import { unzipSync } from 'fflate';
import fs from 'node:fs/promises';
import path from 'node:path';
import { CATALOG, DEFAULT_SETTINGS } from '../../shared/catalog';

test.beforeEach(async ({ page, request }) => {
  const reset = await request.put('/api/project', {
    data: {
      edits: {},
      custom: [],
      selected: CATALOG.map((t) => t.name),
      favorites: [],
      settings: DEFAULT_SETTINGS,
    },
  });
  expect(reset.ok()).toBe(true);
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'Import material' })).toBeEnabled();
});

test('library search, categories, favorites, view modes and empty state', async ({ page }) => {
  await expect(page.locator('.material-card')).toHaveCount(24);
  await page.getByRole('textbox', { name: 'Search materials' }).fill('oak');
  await expect(page.locator('.material-card')).toHaveCount(3);
  await page.getByRole('textbox', { name: 'Search materials' }).fill('does-not-exist');
  await expect(page.getByRole('heading', { name: 'No materials found' })).toBeVisible();
  await page.getByRole('button', { name: 'Show all materials' }).click();
  await page.getByRole('button', { name: 'Metal', exact: true }).click();
  await expect(page.locator('.material-card')).toHaveCount(3);
  await page.getByRole('button', { name: 'Favorite Iron block', exact: true }).click();
  await page.getByRole('button', { name: 'Favorites', exact: true }).click();
  await expect(page.locator('.material-card')).toHaveCount(1);
  await page.getByRole('button', { name: 'List view' }).click();
  await expect(page.locator('.material-grid')).toHaveClass(/list-view/);
  await page.reload();
  await expect(page.getByRole('button', { name: 'Unfavorite Iron block', exact: true })).toHaveAttribute(
    'aria-pressed',
    'true',
  );
});

test('material edits generate a new preview and survive reload; reset restores defaults', async ({
  page,
}) => {
  await page.getByRole('button', { name: 'Inspect Stone', exact: true }).click();
  await page.getByRole('slider', { name: 'Roughness', exact: true }).fill('0.44');
  await page.getByRole('button', { name: 'Apply & preview' }).click();
  await expect(page.locator('.toast')).toContainText('Stone updated');
  await expect(page.locator('.save-status')).toContainText('All changes saved');
  await page.reload();
  await expect(page.getByRole('slider', { name: 'Roughness', exact: true })).toHaveValue('0.44');
  await page.getByRole('button', { name: 'Reset material', exact: true }).click();
  await expect(page.getByRole('slider', { name: 'Roughness', exact: true })).toHaveValue('0.78');
  await page.getByRole('button', { name: 'Specular', exact: true }).click();
  await expect(page.getByRole('img', { name: 'Stone specular map' })).toBeVisible();
});

test('settings and selection control a real build, ZIP download, history and deletion', async ({ page }) => {
  await page.getByRole('button', { name: 'Pack settings', exact: true }).click();
  await page.getByLabel('Pack name', { exact: true }).fill('Browser Test Pack');
  await page.getByLabel('Output resolution').selectOption('64');
  await page.getByRole('button', { name: 'Save settings' }).click();
  await page.getByRole('checkbox', { name: '24 materials', exact: true }).uncheck();
  await page.getByRole('checkbox', { name: 'Include Iron block in pack', exact: true }).click();
  await page.getByRole('button', { name: 'Build pack' }).click();
  await expect(page.getByRole('dialog')).toContainText('1materials');
  await page.getByRole('button', { name: 'Build resource pack', exact: true }).click();
  await expect(page.locator('.job-status').first()).toHaveText('Ready to download', { timeout: 30000 });
  const promise = page.waitForEvent('download');
  await page.getByRole('link', { name: 'Download pack' }).first().click();
  const download = await promise;
  expect(download.suggestedFilename()).toBe('Browser Test Pack-64.zip');
  const file = await download.path();
  const zip = unzipSync(await fs.readFile(file!));
  expect(Object.keys(zip)).toHaveLength(5);
  expect(zip['pack.mcmeta']).toBeDefined();
  expect(zip['assets/minecraft/textures/block/iron_block_s.png']).toBeDefined();
  await page.getByRole('button', { name: 'Export history' }).click();
  await expect(page.locator('.job-card.complete')).toHaveCount(1);
  await page.getByRole('button', { name: 'Delete export Browser Test Pack' }).click();
  await page.getByRole('button', { name: 'Delete export', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Your first export is waiting' })).toBeVisible();
});

test('custom PNG import creates a usable material with a generated preview', async ({ page }) => {
  await page.getByRole('button', { name: 'Import material' }).click();
  await page.getByLabel('Display name', { exact: true }).fill('Browser limestone');
  await expect(page.getByLabel('Texture filename')).toHaveValue('browser_limestone');
  await page.getByLabel('Albedo PNG file').setInputFiles(path.resolve('test_assets/diffuse.png'));
  await page.getByRole('dialog').getByRole('button', { name: 'Import material', exact: true }).click();
  await expect(page.getByRole('dialog')).not.toBeVisible({ timeout: 20000 });
  await expect(page.locator('.material-card')).toHaveCount(25);
  await expect(page.locator('.inspector-title')).toContainText('Browser limestone');
  await expect(page.locator('.preview-loading')).not.toBeVisible({ timeout: 20000 });
  await expect(page.locator('.inspector .inline-error')).not.toBeVisible();
  await page.reload();
  await expect(page.locator('.material-card')).toHaveCount(25);
});

test('import validation and dialog keyboard dismissal', async ({ page }) => {
  await page.getByRole('button', { name: 'Import material' }).click();
  await page.getByLabel('Display name', { exact: true }).fill('Stone');
  await page.getByLabel('Albedo PNG file').setInputFiles(path.resolve('test_assets/diffuse.png'));
  await page.getByRole('dialog').getByRole('button', { name: 'Import material', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('already exists');
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).not.toBeVisible();
});

test('mobile navigation, inspector and layouts stay within the viewport', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(page.locator('.inspector')).not.toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.getByRole('button', { name: 'Inspect Stone', exact: true }).click();
  await expect(page.locator('.inspector')).toBeVisible();
  await page.getByRole('button', { name: 'Close inspector', exact: true }).click();
  await page.getByRole('button', { name: 'Open navigation' }).click();
  await page.getByRole('button', { name: 'Getting started', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'A better world starts here.' })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.getByRole('button', { name: 'Back to library' }).click();
  await expect(page.locator('.material-card')).toHaveCount(24);
});
