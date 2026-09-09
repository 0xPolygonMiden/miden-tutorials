import { expect, test } from '@playwright/test';

const tutorialTimeoutMs = 30 * 60 * 1000;

const names = [
  'createMintConsume',
  'multiSendWithDelegatedProver',
  'unauthenticatedNoteTransfer',
] as const;

for (const name of names) {
  test(`react:${name}`, async ({ page }) => {
    test.setTimeout(tutorialTimeoutMs);
    const logs: string[] = [];
    const errors: string[] = [];
    page.on('console', (message) => {
      logs.push(`[${message.type()}] ${message.text()}`);
      if (message.type() === 'error') errors.push(message.text());
    });
    page.on('pageerror', (error) => {
      errors.push(error.message);
    });
    await page.goto(`/react-tutorials?tutorial=${name}`);
    const tutorial = page.getByTestId(`react-${name}`);
    await expect(tutorial).toBeVisible({ timeout: 120_000 });
    await expect
      .poll(
        async () => {
          const state = await tutorial.getAttribute('data-state');
          if (state === 'failed') throw new Error(await tutorial.innerText());
          return tutorial.getByRole('button').isEnabled();
        },
        { timeout: 120_000 },
      )
      .toBe(true);
    await tutorial.getByRole('button').click();
    await expect(tutorial).toHaveAttribute('data-state', /passed|failed/, {
      timeout: tutorialTimeoutMs,
    });
    expect(await tutorial.getByRole('status').innerText()).toBe('passed');
    expect(errors).toEqual([]);
    expect(logs.some((line) => line.includes('Transaction committed:'))).toBe(
      true,
    );
    expect(logs.some((line) => line.includes('Verified balance'))).toBe(true);
  });
}
