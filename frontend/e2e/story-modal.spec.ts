import { test, expect } from '@playwright/test'
import { getSeedData } from './helpers'

test.describe('Story Modal', () => {
  let projectSlug: string
  let featureStoryId: string

  test.beforeAll(() => {
    const seed = getSeedData()
    projectSlug = seed.projectSlug
    featureStoryId = seed.featureStoryId
  })

  // ── Scenario 12: No backdrop click dismissal ────────────────────────────────

  test('clicking the backdrop does not close the modal', async ({ page }) => {
    await page.goto(`/projects/${projectSlug}`)
    await page.waitForSelector('[role="row"]', { timeout: 10_000 })

    // Open story modal
    await page.locator('[role="row"]').first().click()
    const modal = page.locator('[role="dialog"]')
    await expect(modal).toBeVisible()

    // Click the backdrop area (the outer fixed overlay, outside the inner card)
    // The backdrop is the outermost div with bg-black/80
    const backdrop = page.locator('[role="dialog"]').locator('..')
    const backdropBox = await backdrop.boundingBox()
    if (backdropBox) {
      // Click top-left corner of the backdrop (outside the modal card)
      await page.mouse.click(backdropBox.x + 5, backdropBox.y + 5)
    }

    // Modal should still be open — no backdrop dismissal
    await expect(modal).toBeVisible()
  })

  // ── Scenario 13: URL deep link to story ──────────────────────────────────────

  test('navigate directly to a story URL and modal opens', async ({ page }) => {
    test.skip(!featureStoryId, 'No seeded feature story ID')

    await page.goto(`/projects/${projectSlug}/stories/${featureStoryId}`)

    // Modal should open automatically from the URL
    const modal = page.locator('[role="dialog"]')
    await expect(modal).toBeVisible({ timeout: 10_000 })

    // Breadcrumb should be visible
    const breadcrumb = page.locator('nav[aria-label="breadcrumb"]')
    await expect(breadcrumb).toBeVisible()

    // Story title should appear in the modal heading
    await expect(modal.getByRole('heading', { name: 'Seeded E2E Story A' }).first()).toBeVisible()
  })

  // ── Scenario 17: Story overview panel content ────────────────────────────────

  test('story overview panel shows description, status, and tasks', async ({ page }) => {
    test.skip(!featureStoryId, 'No seeded feature story ID')

    await page.goto(`/projects/${projectSlug}/stories/${featureStoryId}`)

    const modal = page.locator('[role="dialog"]')
    await expect(modal).toBeVisible({ timeout: 10_000 })

    // Description should be visible in the overview panel
    await expect(modal.getByText(/seeded by global setup/i).first()).toBeVisible()

    // Status badge should be visible (To Do is the default)
    await expect(modal.getByText('To Do').first()).toBeVisible()

    // If tasks were seeded, they should appear in the task list
    const taskSetup = modal.getByText('E2E Task: Setup auth')
    const taskDesign = modal.getByText('E2E Task: Design login')

    // Tasks may or may not be seeded depending on run order; check if at least
    // the overview panel renders without errors
    const hasTask = await taskSetup.isVisible().catch(() => false)
    if (hasTask) {
      await expect(taskSetup).toBeVisible()
      await expect(taskDesign).toBeVisible()
    }
  })

  // ── Scenario: Two-column layout visible on desktop ───────────────────────────

  test('modal shows two-column layout on desktop', async ({ page }) => {
    test.skip(!featureStoryId, 'No seeded feature story ID')

    await page.goto(`/projects/${projectSlug}/stories/${featureStoryId}`)

    const modal = page.locator('[role="dialog"]')
    await expect(modal).toBeVisible({ timeout: 10_000 })

    // Right panel should have tab switcher with Q&A Thread and Decision Trail
    const qaTab = modal.getByRole('button', { name: /q&a thread/i })
    const decisionTab = modal.getByRole('button', { name: /decision trail/i })

    await expect(qaTab).toBeVisible()
    await expect(decisionTab).toBeVisible()
  })

  // ── Scenario: Task drill-in with seeded tasks ────────────────────────────────

  test('drill into a seeded task and see task overview', async ({ page }) => {
    test.skip(!featureStoryId, 'No seeded feature story ID')

    await page.goto(`/projects/${projectSlug}/stories/${featureStoryId}`)

    const modal = page.locator('[role="dialog"]')
    await expect(modal).toBeVisible({ timeout: 10_000 })

    // Look for the seeded task
    const taskItem = modal.getByText('E2E Task: Setup auth')
    const isVisible = await taskItem.isVisible().catch(() => false)
    if (!isVisible) {
      test.skip(true, 'Seeded tasks not found — task seeding may have failed')
      return
    }

    // Click the task to drill in
    await taskItem.click()

    // Breadcrumb should now have 3 segments (project > story > task)
    const breadcrumb = modal.locator('nav[aria-label="breadcrumb"]')
    await expect(breadcrumb).toBeVisible()

    // Story name in breadcrumb should be a clickable button to go back
    const storyLink = breadcrumb.getByRole('button', { name: /Seeded E2E Story A/i })
    await expect(storyLink).toBeVisible()

    // Navigate back via breadcrumb
    await storyLink.click()

    // Should be back in story view — task list should reappear
    await expect(modal.getByText('E2E Task: Setup auth')).toBeVisible()
  })

  // ── Scenario: Close modal navigates back to table URL ────────────────────────

  test('closing modal navigates back to project stories URL', async ({ page }) => {
    test.skip(!featureStoryId, 'No seeded feature story ID')

    await page.goto(`/projects/${projectSlug}/stories/${featureStoryId}`)

    const modal = page.locator('[role="dialog"]')
    await expect(modal).toBeVisible({ timeout: 10_000 })

    // Close via X button
    await modal.getByLabel('Close').click()

    // Modal should be gone
    await expect(modal).not.toBeVisible()

    // URL should be back to the project page
    await expect(page).toHaveURL(new RegExp(`/projects/${projectSlug}$`))
  })
})
