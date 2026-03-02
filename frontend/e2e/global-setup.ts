import { test as setup, expect } from '@playwright/test'
import path from 'node:path'
import fs from 'node:fs'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const AUTH_FILE = path.join(__dirname, '.auth', 'user.json')
const SEED_FILE = path.join(__dirname, '.auth', 'seed.json')

const API_URL = process.env.E2E_BASE_URL ?? 'http://localhost:4173'

setup('authenticate and seed project', async ({ request }) => {
  // Dev-login — creates user + org on first call
  const loginResp = await request.post(`${API_URL}/api/v1/auth/dev-login`, {
    data: { email: 'e2e@test.local', display_name: 'E2E User' },
    maxRedirects: 0,
  })
  expect([200, 302, 303]).toContain(loginResp.status())

  // Persist cookie-based auth state
  const storageState = await request.storageState()
  fs.mkdirSync(path.dirname(AUTH_FILE), { recursive: true })
  fs.writeFileSync(AUTH_FILE, JSON.stringify(storageState, null, 2))

  // Get current user info (needed for owner_id when creating stories)
  const meResp = await request.get(`${API_URL}/api/v1/auth/me`)
  expect(meResp.status()).toBe(200)
  const me = await meResp.json()

  // Seed a project (idempotent — if it already exists the test still passes)
  const projectResp = await request.post(`${API_URL}/api/v1/projects`, {
    data: { name: 'E2E Project' },
  })
  // 201 = created, 409 = already exists (unique name constraint)
  expect([201, 409]).toContain(projectResp.status())

  // Retrieve the project to get its ID
  const projectsResp = await request.get(`${API_URL}/api/v1/projects`)
  expect(projectsResp.status()).toBe(200)
  const projects = await projectsResp.json()
  const project = projects.find((p: { name: string }) => p.name === 'E2E Project')
  expect(project).toBeTruthy()

  const projectSlug = slugify(project.name)

  // Seed two stories so parallel tests have rows to interact with
  for (const title of ['Seeded E2E Story A', 'Seeded E2E Story B']) {
    const storyResp = await request.post(`${API_URL}/api/v1/stories`, {
      data: {
        project_id: project.id,
        title,
        description: `Story "${title}" seeded by global setup.`,
        story_type: 'feature',
        owner_id: me.user_id,
      },
    })
    // 201 = created; other codes okay if story already exists from previous run
    expect([201, 409]).toContain(storyResp.status())
  }

  fs.writeFileSync(SEED_FILE, JSON.stringify({ projectSlug }, null, 2))
})

function slugify(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '')
}
