import { test, expect } from '@playwright/test'
import { getSeedData } from './helpers'

test.describe('Stories', () => {
  let projectSlug: string

  test.beforeAll(() => {
    projectSlug = getSeedData().projectSlug
  })

  // ── Scenario 2: Create story ────────────────────────────────────────────────

  test('create a new story via the creation modal', async ({ page }) => {
    await page.goto(`/projects/${projectSlug}`)

    // Use a unique title to avoid collisions with stories from previous runs
    const uniqueTitle = `E2E Story ${Date.now()}`

    // Click "New Story" button (exact match to avoid ambiguity with "New" button)
    await page.getByRole('button', { name: 'New Story' }).click()

    // Fill the creation form
    await page.getByLabel(/title/i).fill(uniqueTitle)
    await page.getByLabel(/brief description/i).fill('A story created by E2E tests.')

    // Select story type (Feature is default, click Bug to verify interactivity)
    await page.getByRole('button', { name: /bug/i }).click()

    // Submit the form — "Create Story" triggers AI expansion, but since there's
    // no agent running, the story should still be created with the brief description.
    await page.getByRole('button', { name: /create story/i }).click()

    // Verify the story appears in the table
    await expect(page.getByRole('row').filter({ hasText: uniqueTitle })).toBeVisible({
      timeout: 10_000,
    })
  })

  // ── Scenario 3: Story reorder via drag-and-drop ─────────────────────────────

  test('reorder stories via drag and drop', async ({ page }) => {
    await page.goto(`/projects/${projectSlug}`)
    await page.waitForSelector('[role="row"]', { timeout: 10_000 })

    const rows = page.locator('[role="row"]')
    const count = await rows.count()

    if (count < 2) {
      test.skip(true, 'Need at least 2 stories to test reorder')
      return
    }

    // Verify drag handles are present
    const firstHandle = rows.first().locator('svg').first()
    await expect(firstHandle).toBeVisible()

    // Extract titles before drag
    const firstTitle = await rows.first().locator('span.truncate').first().innerText()
    const secondTitle = await rows.nth(1).locator('span.truncate').first().innerText()
    expect(firstTitle).not.toBe(secondTitle)

    // Perform drag using low-level mouse events for @dnd-kit PointerSensor
    const handleBox = await firstHandle.boundingBox()
    const targetBox = await rows.nth(1).boundingBox()

    if (handleBox && targetBox) {
      const startX = handleBox.x + handleBox.width / 2
      const startY = handleBox.y + handleBox.height / 2
      const endY = targetBox.y + targetBox.height + 5

      await page.mouse.move(startX, startY)
      await page.mouse.down()
      await page.mouse.move(startX, startY + 10, { steps: 5 })
      await page.mouse.move(startX, endY, { steps: 10 })
      await page.mouse.up()

      // Wait for API mutation + refetch
      await page.waitForTimeout(1000)

      // Verify the drag had an effect — the first row should now be the former second
      const newFirst = await rows.first().locator('span.truncate').first().innerText()
      expect(newFirst).toBe(secondTitle)
    }
  })

  // ── Scenario 4: Story modal — open and close ───────────────────────────────

  test('open story modal by clicking a row and close via X button', async ({ page }) => {
    await page.goto(`/projects/${projectSlug}`)
    await page.waitForSelector('[role="row"]', { timeout: 10_000 })

    // Click first story row
    await page.locator('[role="row"]').first().click()

    // Modal should appear
    const modal = page.locator('[role="dialog"]')
    await expect(modal).toBeVisible()

    // Breadcrumb should show project slug and story name
    const breadcrumb = page.locator('nav[aria-label="breadcrumb"]')
    await expect(breadcrumb).toBeVisible()

    // Close via X button (aria-label="Close")
    await modal.getByLabel('Close').click()
    await expect(modal).not.toBeVisible()
  })

  test('close story modal via Escape key', async ({ page }) => {
    await page.goto(`/projects/${projectSlug}`)
    await page.waitForSelector('[role="row"]', { timeout: 10_000 })

    await page.locator('[role="row"]').first().click()

    const modal = page.locator('[role="dialog"]')
    await expect(modal).toBeVisible()

    await page.keyboard.press('Escape')
    // May show confirmation dialog if there are pending questions; handle both cases
    const confirmDialog = page.getByText(/pending questions/i)
    if (await confirmDialog.isVisible().catch(() => false)) {
      await page.getByRole('button', { name: /leave/i }).click()
    }
    await expect(modal).not.toBeVisible()
  })

  // ── Scenario 5: Story modal — tab switching ─────────────────────────────────

  test('switch between Q&A Thread and Decision Trail tabs', async ({ page }) => {
    await page.goto(`/projects/${projectSlug}`)
    await page.waitForSelector('[role="row"]', { timeout: 10_000 })

    await page.locator('[role="row"]').first().click()

    const modal = page.locator('[role="dialog"]')
    await expect(modal).toBeVisible()

    // Default tab should be Q&A Thread
    const qaTab = modal.getByRole('button', { name: /q&a thread/i })
    const decisionTab = modal.getByRole('button', { name: /decision trail/i })

    await expect(qaTab).toBeVisible()
    await expect(decisionTab).toBeVisible()

    // Switch to Decision Trail
    await decisionTab.click()

    // Switch back to Q&A Thread
    await qaTab.click()
  })

  // ── Scenario 6: Task drill-in ───────────────────────────────────────────────

  test('drill into a task and navigate back via breadcrumb', async ({ page }) => {
    await page.goto(`/projects/${projectSlug}`)
    await page.waitForSelector('[role="row"]', { timeout: 10_000 })

    // Open a story that has tasks — we need to find one or seed one.
    // Open first story and check if it has tasks.
    await page.locator('[role="row"]').first().click()
    const modal = page.locator('[role="dialog"]')
    await expect(modal).toBeVisible()

    // Look for task list items in the overview panel
    const taskItems = modal.locator('text=Tasks').locator('..')
    const taskButtons = taskItems.locator('button, [role="button"], [class*="cursor-pointer"]')

    const taskCount = await taskButtons.count()
    if (taskCount === 0) {
      test.skip(true, 'No tasks in story — seed tasks to enable this test')
      return
    }

    // Click first task
    await taskButtons.first().click()

    // Breadcrumb should now have 3 segments: project > story > task
    const breadcrumb = modal.locator('nav[aria-label="breadcrumb"]')
    const breadcrumbItems = breadcrumb.locator('li')
    await expect(breadcrumbItems).toHaveCount(5) // 3 segments + 2 chevron separators

    // Navigate back to story via breadcrumb (click story name — second segment button)
    const storyBreadcrumb = breadcrumb.locator('button').nth(1)
    await storyBreadcrumb.click()

    // Should be back in story view — breadcrumb has 2 segments now
    await expect(breadcrumbItems).toHaveCount(3) // 2 segments + 1 chevron
  })
})
