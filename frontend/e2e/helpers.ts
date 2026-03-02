import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const SEED_FILE = path.join(__dirname, '.auth', 'seed.json')

interface SeedData {
  projectSlug: string
}

export function getSeedData(): SeedData {
  const raw = fs.readFileSync(SEED_FILE, 'utf-8')
  return JSON.parse(raw) as SeedData
}
