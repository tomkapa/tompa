import { test, expect } from '@playwright/test'
import { getSeedData } from './helpers'

test.describe('Projects', () => {
  let projectSlug: string

  test.beforeAll(() => {
    projectSlug = getSeedData().projectSlug
  })

  // ── Scenario 4: Create a new project ────────────────────────────────────────

  test('create a new project via the project selector', async ({ page }) => {
    await page.goto(`/projects/${projectSlug}`)
    await page.waitForSelector('[role="row"]', { timeout: 10_000 })

    const uniqueName = `E2E Project ${Date.now()}`

    // Open the project selector dropdown
    await page.getByText(projectSlug.replace(/-/g, ' '), { exact: false }).first().click()

    // Click "Create new project" at the bottom of the dropdown
    await page.getByText('Create new project').click()

    // The create project dialog should appear
    const dialog = page.locator('[role="dialog"]').filter({ hasText: 'Create New Project' })
    await expect(dialog).toBeVisible()

    // Fill in the project name
    await dialog.getByLabel(/project name/i).fill(uniqueName)
    await dialog.getByLabel(/description/i).fill('Project created by E2E test')

    // Submit
    await dialog.getByRole('button', { name: /create project/i }).click()

    // Should navigate to the new project's page
    const expectedSlug = uniqueName.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '')
    await expect(page).toHaveURL(new RegExp(`/projects/${expectedSlug}`), { timeout: 10_000 })
  })

  // ── Scenario 5: Switch between projects ─────────────────────────────────────

  test('switch between projects via the selector dropdown', async ({ page }) => {
    await page.goto(`/projects/${projectSlug}`)
    await page.waitForSelector('[role="row"]', { timeout: 10_000 })

    // Open the project selector
    const selectorTrigger = page.locator('button').filter({ hasText: /E2E Project/i }).first()
    await selectorTrigger.click()

    // The dropdown should show the current project with a checkmark
    const dropdown = page.locator('.bg-popover')
    await expect(dropdown).toBeVisible()

    // The current project should be listed
    await expect(dropdown.getByText('E2E Project').first()).toBeVisible()

    // Close the dropdown by pressing Escape
    await page.keyboard.press('Escape')
    await expect(dropdown).not.toBeVisible()
  })

  // ── Scenario 6: Project selector search ─────────────────────────────────────

  test('project selector filters projects by search text', async ({ page }) => {
    await page.goto(`/projects/${projectSlug}`)
    await page.waitForSelector('[role="row"]', { timeout: 10_000 })

    // Open the project selector
    const selectorTrigger = page.locator('button').filter({ hasText: /E2E Project/i }).first()
    await selectorTrigger.click()

    const dropdown = page.locator('.bg-popover')
    await expect(dropdown).toBeVisible()

    // Type a search query that won't match any project
    const searchInput = dropdown.getByPlaceholder('Search projects...')
    await searchInput.fill('nonexistent-xyz-query')

    // Should show "No projects found" message
    await expect(dropdown.getByText('No projects found')).toBeVisible()

    // Clear and type a matching query
    await searchInput.fill('E2E')

    // The E2E Project should still be visible
    await expect(dropdown.getByText('E2E Project').first()).toBeVisible()
    await expect(dropdown.getByText('No projects found')).not.toBeVisible()
  })

  // ── Scenario: Create project validation — name required ─────────────────────

  test('create project button is disabled when name is empty', async ({ page }) => {
    await page.goto(`/projects/${projectSlug}`)
    await page.waitForSelector('[role="row"]', { timeout: 10_000 })

    // Open project selector → create new
    const selectorTrigger = page.locator('button').filter({ hasText: /E2E Project/i }).first()
    await selectorTrigger.click()
    await page.getByText('Create new project').click()

    const dialog = page.locator('[role="dialog"]').filter({ hasText: 'Create New Project' })
    await expect(dialog).toBeVisible()

    // Create Project button should be disabled when name is empty
    const createBtn = dialog.getByRole('button', { name: /create project/i })
    await expect(createBtn).toBeDisabled()

    // Fill in name
    await dialog.getByLabel(/project name/i).fill('Test Project')
    await expect(createBtn).toBeEnabled()

    // Cancel to clean up
    await dialog.getByRole('button', { name: /cancel/i }).click()
  })
})
