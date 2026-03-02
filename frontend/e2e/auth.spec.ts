import { test, expect } from '@playwright/test'
import { getSeedData } from './helpers'

test.describe('Authentication', () => {
  test('unauthenticated user is redirected to /login', async ({ browser }) => {
    const context = await browser.newContext({ storageState: undefined })
    const page = await context.newPage()
    await page.goto('/')
    await expect(page).toHaveURL(/\/login/)
    await context.close()
  })

  test('login page renders OAuth buttons', async ({ browser }) => {
    const context = await browser.newContext({ storageState: undefined })
    const page = await context.newPage()
    await page.goto('/login')
    await expect(page.getByRole('button', { name: /github/i })).toBeVisible()
    await expect(page.getByRole('button', { name: /google/i })).toBeVisible()
    await context.close()
  })

  test('authenticated user lands on project page', async ({ page }) => {
    const { projectSlug } = getSeedData()
    await page.goto('/')
    await expect(page).toHaveURL(new RegExp(`/projects/${projectSlug}`))
  })
})
