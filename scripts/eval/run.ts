#!/usr/bin/env bun
/**
 * Tompa Prompt Eval Loop
 *
 * Usage:
 *   bun run run.ts                          # run all roles × all stories once
 *   bun run run.ts --watch                  # re-run on any roles/*.txt change
 *   bun run run.ts --role business_analyst  # single role, all stories
 *   bun run run.ts --story 0               # all roles, story index 0
 *   bun run run.ts --stage planning        # eval planning prompt instead
 *
 * Requires: ANTHROPIC_API_KEY env var
 */

import Anthropic from "@anthropic-ai/sdk";
import { readFileSync, writeFileSync, existsSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROLES_DIR = resolve(__dirname, "../../backend/server/src/agents/prompts/roles");
const GROOMING_TOML_PATH = resolve(ROLES_DIR, "grooming.toml");
const FIXTURES_PATH = resolve(__dirname, "fixtures/stories.json");
const SCORES_PATH = resolve(__dirname, ".last-scores.json");

// ─── Types ────────────────────────────────────────────────────────────────────

interface Story {
  id: string;
  title: string;
  type: "feature" | "bug";
  description: string;
}

interface GroomingRole {
  id: string;
  title: string;
  domain: string;
  instructions: string;
}

interface EvalScores {
  relevance: number;
  role_fit: number;
  atomicity: number;
  option_quality: number;
  no_duplication: number;
  overall: number;
  pass: boolean;
  notes: string;
  question_count: number;
  elapsed_ms: number;
}

type ScoreCache = Record<string, EvalScores>; // key: `${roleId}:${storyId}`

// ─── Config — loaded from grooming.toml (single source of truth) ─────────────

function loadGroomingRoles(): GroomingRole[] {
  const raw = readFileSync(GROOMING_TOML_PATH, "utf-8");
  // Minimal TOML parser for [[roles]] array of tables
  const roles: GroomingRole[] = [];
  const blocks = raw.split(/^\[\[roles\]\]/m).slice(1);
  for (const block of blocks) {
    const id           = block.match(/^id\s*=\s*"([^"]+)"/m)?.[1] ?? "";
    const title        = block.match(/^title\s*=\s*"([^"]+)"/m)?.[1] ?? "";
    const domain       = block.match(/^domain\s*=\s*"([^"]+)"/m)?.[1] ?? "";
    const instrMatch   = block.match(/^instructions\s*=\s*"""\n([\s\S]*?)\n"""/m);
    const instructions = instrMatch?.[1]?.trim() ?? "";
    if (id) roles.push({ id, title, domain, instructions });
  }
  return roles;
}

let GROOMING_ROLES = loadGroomingRoles();

const GEN_MODEL  = "claude-haiku-4-5-20251001"; // fast + cheap for generation
const JUDGE_MODEL = "claude-sonnet-4-6";          // more accurate for judging

// ─── Anthropic client ─────────────────────────────────────────────────────────

const client = new Anthropic({ apiKey: process.env.ANTHROPIC_API_KEY });

// ─── Prompt builders (mirrors Rust logic) ─────────────────────────────────────

function buildGroomingSystemPrompt(role: GroomingRole, instructions: string): string {
  return `You are a ${role.title} participating in a software story grooming session.

${instructions}

QUESTION SCOPE: Raise questions about decisions that require meaningful effort to reverse or that meaningfully affect quality.
QUESTION LIMIT: Generate at most 3 questions for your domain. Prioritize by impact — keep only the most consequential ones.

SESSION STATE: Round 1 — 0 decisions made so far.
CONVERGENCE: Focus on decisions with meaningful downstream impact. Most stories need at most 8–10 total decisions.
Every question costs the team review time — an unnecessary question is worse than a missing one. Return {"questions": []} when remaining unknowns would not change the implementation approach.

For each question:
- "rationale": One sentence explaining why this decision matters and its downstream consequences.
- "options": Each option is an object with "label" (concise choice), "pros" (2–4 sentences, honest advantages), and "cons" (2–4 sentences, honest disadvantages).
- "recommended_option_index": Zero-based index of the option you recommend.

Respond ONLY with valid JSON in exactly this format — no other text, no markdown fences:
{
  "questions": [
    {
      "text": "Your question here?",
      "domain": "${role.domain}",
      "rationale": "This decision matters because...",
      "recommended_option_index": 0,
      "options": [
        { "label": "Option A", "pros": "Advantages.", "cons": "Disadvantages." },
        { "label": "Option B", "pros": "Advantages.", "cons": "Disadvantages." }
      ]
    }
  ]
}`;
}

function buildGroomingUserPrompt(role: GroomingRole, story: Story): string {
  return `## Story Description
${story.description}

## Decisions Already Made
None yet.

Based on your ${role.title} perspective, generate 0–3 clarifying questions about this story.
Each question must have 2–5 mutually-exclusive predefined answer options.`;
}

// ─── Generation ───────────────────────────────────────────────────────────────

async function generateGroomingQuestions(
  role: GroomingRole,
  story: Story,
): Promise<{ questions: unknown[]; elapsed_ms: number }> {
  const system = buildGroomingSystemPrompt(role, role.instructions);
  const userPrompt = buildGroomingUserPrompt(role, story);

  const t0 = Date.now();
  const response = await client.messages.create({
    model: GEN_MODEL,
    max_tokens: 2000,
    system,
    messages: [{ role: "user", content: userPrompt }],
  });
  const elapsed_ms = Date.now() - t0;

  const raw = response.content[0].type === "text" ? response.content[0].text : "";
  try {
    const parsed = JSON.parse(raw) as { questions?: unknown[] };
    return { questions: parsed.questions ?? [], elapsed_ms };
  } catch {
    console.error("[eval] JSON parse failed for generation output:", raw.slice(0, 200));
    return { questions: [], elapsed_ms };
  }
}

// ─── Judge ────────────────────────────────────────────────────────────────────

function buildJudgePrompt(role: GroomingRole, story: Story, questions: unknown[]): string {
  const questionsText =
    questions.length === 0
      ? "(no questions generated)"
      : (questions as Array<{ text: string; rationale?: string; options?: Array<{ label: string }> }>)
          .map(
            (q, i) =>
              `Q${i + 1}: ${q.text}\n  Rationale: ${q.rationale ?? "none"}\n  Options: ${
                q.options?.map((o) => o.label).join(", ") ?? "none"
              }`,
          )
          .join("\n\n");

  return `You are evaluating grooming questions generated by an AI ${role.title} agent for a software project.

STORY TITLE: ${story.title}
STORY TYPE: ${story.type}
STORY DESCRIPTION:
${story.description}

ROLE BEING EVALUATED: ${role.title} (domain: ${role.domain})

GENERATED QUESTIONS:
${questionsText}

Rate the generated questions as a whole on these criteria (score 1–5 each):

1. RELEVANCE (1-5): Do the questions address real ambiguities in this story that would affect implementation?
   - 5: All questions target genuine unknowns that could significantly change how the feature is built
   - 3: Mix of useful and redundant questions
   - 1: Questions are generic or answerable from the story description already

2. ROLE_FIT (1-5): Are questions correctly scoped to the ${role.domain} domain, not bleeding into other roles?
   - 5: Perfectly domain-scoped; a ${role.title} would naturally own these decisions
   - 3: Mostly on-domain with minor overreach
   - 1: Questions belong to another role entirely

3. ATOMICITY (1-5): Does each question ask exactly one decision, not bundling multiple choices?
   - 5: Each question is a single, clear decision point
   - 3: Some questions combine two concerns
   - 1: Questions are compound or ambiguous

4. OPTION_QUALITY (1-5): Are the options mutually exclusive, meaningfully different, and realistic?
   - 5: Options are distinct, exhaustive, and represent real tradeoffs
   - 3: Some options overlap or are trivially different
   - 1: Options are arbitrary or not mutually exclusive

5. NO_DUPLICATION (1-5): Are questions distinct from each other and not already answered by the story description?
   - 5: Each question adds new value; nothing redundant
   - 3: Some overlap between questions or with story context
   - 1: Questions repeat or are already answered in the story

Also set "pass" to true if overall quality would be acceptable for production use (overall >= 3.5).

Respond ONLY with valid JSON (no markdown fences):
{"relevance":N,"role_fit":N,"atomicity":N,"option_quality":N,"no_duplication":N,"pass":bool,"notes":"brief explanation of main strengths and weaknesses in 1-2 sentences"}`;
}

async function judgeQuestions(
  role: GroomingRole,
  story: Story,
  questions: unknown[],
  elapsed_ms: number,
): Promise<EvalScores> {
  const judgePrompt = buildJudgePrompt(role, story, questions);

  const response = await client.messages.create({
    model: JUDGE_MODEL,
    max_tokens: 512,
    messages: [{ role: "user", content: judgePrompt }],
  });

  const raw = response.content[0].type === "text" ? response.content[0].text : "{}";
  try {
    const s = JSON.parse(raw) as Omit<EvalScores, "overall" | "question_count" | "elapsed_ms">;
    const overall =
      Math.round(((s.relevance + s.role_fit + s.atomicity + s.option_quality + s.no_duplication) / 5) * 10) / 10;
    return { ...s, overall, question_count: questions.length, elapsed_ms };
  } catch {
    console.error("[eval] JSON parse failed for judge output:", raw.slice(0, 200));
    return {
      relevance: 0, role_fit: 0, atomicity: 0, option_quality: 0, no_duplication: 0,
      overall: 0, pass: false, notes: "Judge parse error", question_count: 0, elapsed_ms,
    };
  }
}

// ─── Display ──────────────────────────────────────────────────────────────────

const C = {
  reset:  "\x1b[0m",
  bold:   "\x1b[1m",
  dim:    "\x1b[2m",
  green:  "\x1b[32m",
  red:    "\x1b[31m",
  yellow: "\x1b[33m",
  cyan:   "\x1b[36m",
  gray:   "\x1b[90m",
} as const;

function colorScore(score: number): string {
  const s = score.toFixed(1);
  if (score >= 4) return `${C.green}${s}${C.reset}`;
  if (score >= 3) return `${C.yellow}${s}${C.reset}`;
  return `${C.red}${s}${C.reset}`;
}

function colorDiff(diff: number): string {
  if (diff > 0)  return `${C.green}+${diff.toFixed(1)}${C.reset}`;
  if (diff < 0)  return `${C.red}${diff.toFixed(1)}${C.reset}`;
  return `${C.gray}  — ${C.reset}`;
}

function printResult(role: GroomingRole, story: Story, scores: EvalScores, prev?: EvalScores): void {
  const pass = scores.pass ? `${C.green}✓ PASS${C.reset}` : `${C.red}✗ FAIL${C.reset}`;
  console.log(`\n${C.bold}${C.cyan}${"━".repeat(60)}${C.reset}`);
  console.log(`${C.bold}${role.title}${C.reset} × ${C.bold}"${story.title}"${C.reset}  ${pass}`);
  console.log(`${C.gray}${scores.question_count} questions generated in ${scores.elapsed_ms}ms${C.reset}`);
  console.log(`${C.cyan}${"━".repeat(60)}${C.reset}`);

  const criteria: Array<[string, keyof EvalScores]> = [
    ["Relevance",      "relevance"     ],
    ["Role fit",       "role_fit"      ],
    ["Atomicity",      "atomicity"     ],
    ["Option quality", "option_quality"],
    ["No duplication", "no_duplication"],
  ];

  for (const [label, key] of criteria) {
    const score = scores[key] as number;
    const prevScore = prev ? (prev[key] as number) : undefined;
    const diff = prevScore !== undefined ? score - prevScore : undefined;
    const pad = " ".repeat(16 - label.length);
    const diffStr = diff !== undefined ? `  ${colorDiff(diff)}` : "";
    console.log(`  ${label}${pad}${colorScore(score)} / 5${diffStr}`);
  }

  const prevOverall = prev?.overall;
  const overallDiff = prevOverall !== undefined ? scores.overall - prevOverall : undefined;
  const overallDiffStr = overallDiff !== undefined ? `  ${colorDiff(overallDiff)}` : "";
  console.log(`  ${"─".repeat(32)}`);
  console.log(`  ${"Overall"}         ${C.bold}${colorScore(scores.overall)}${C.reset} / 5${overallDiffStr}`);
  console.log(`\n  ${C.dim}${scores.notes}${C.reset}`);
}

function printSummary(results: Array<{ role: GroomingRole; story: Story; scores: EvalScores }>): void {
  console.log(`\n${C.bold}${"═".repeat(60)}${C.reset}`);
  console.log(`${C.bold}SUMMARY${C.reset}`);
  console.log(`${"═".repeat(60)}`);

  const passed = results.filter((r) => r.scores.pass).length;
  const total  = results.length;
  const avgOverall = results.reduce((s, r) => s + r.scores.overall, 0) / total;

  for (const { role, story, scores } of results) {
    const icon = scores.pass ? `${C.green}✓${C.reset}` : `${C.red}✗${C.reset}`;
    const name = `${role.title} × ${story.id}`.padEnd(50);
    console.log(`  ${icon}  ${name}  ${colorScore(scores.overall)}`);
  }

  console.log(`\n  ${C.bold}Passed: ${passed}/${total}  Avg overall: ${colorScore(avgOverall)}${C.reset}`);
  console.log();
}

// ─── Load / save score cache ──────────────────────────────────────────────────

function loadCache(): ScoreCache {
  if (!existsSync(SCORES_PATH)) return {};
  try {
    return JSON.parse(readFileSync(SCORES_PATH, "utf-8")) as ScoreCache;
  } catch {
    return {};
  }
}

function saveCache(cache: ScoreCache): void {
  writeFileSync(SCORES_PATH, JSON.stringify(cache, null, 2));
}

// ─── Main eval cycle ──────────────────────────────────────────────────────────

async function runEval(opts: { roleFilter?: string; storyIndex?: number }): Promise<void> {
  const stories = JSON.parse(readFileSync(FIXTURES_PATH, "utf-8")) as Story[];
  const cache   = loadCache();

  const targetRoles   = opts.roleFilter
    ? GROOMING_ROLES.filter((r) => r.id === opts.roleFilter)
    : GROOMING_ROLES;
  const targetStories = opts.storyIndex !== undefined
    ? [stories[opts.storyIndex]]
    : stories;

  if (targetRoles.length === 0) {
    console.error(`[eval] Unknown role: ${opts.roleFilter}`);
    console.error(`       Available: ${GROOMING_ROLES.map((r) => r.id).join(", ")}`);
    process.exit(1);
  }

  const results: Array<{ role: GroomingRole; story: Story; scores: EvalScores }> = [];
  const newCache: ScoreCache = { ...cache };

  for (const story of targetStories) {
    for (const role of targetRoles) {
      const cacheKey = `${role.id}:${story.id}`;
      const prev = cache[cacheKey];

      process.stdout.write(`${C.dim}  Running ${role.title} × "${story.title}"...${C.reset}\r`);

      const { questions, elapsed_ms } = await generateGroomingQuestions(role, story);
      const scores = await judgeQuestions(role, story, questions, elapsed_ms);

      newCache[cacheKey] = scores;
      results.push({ role, story, scores });
      printResult(role, story, scores, prev);
    }
  }

  saveCache(newCache);
  if (results.length > 1) printSummary(results);
}

// ─── CLI + watch mode ─────────────────────────────────────────────────────────

const args = process.argv.slice(2);
const watchMode   = args.includes("--watch");
const roleFilter  = args[args.indexOf("--role")  + 1] as string | undefined;
const storyArg    = args[args.indexOf("--story") + 1];
const storyIndex  = storyArg !== undefined ? parseInt(storyArg, 10) : undefined;

if (!process.env.ANTHROPIC_API_KEY) {
  console.error(`${C.red}Error: ANTHROPIC_API_KEY env var not set${C.reset}`);
  process.exit(1);
}

async function runAndCatch(): Promise<void> {
  console.log(`\n${C.bold}${C.cyan}Tompa Eval Loop${C.reset}  ${C.gray}${new Date().toLocaleTimeString()}${C.reset}\n`);
  try {
    await runEval({ roleFilter, storyIndex });
  } catch (err) {
    console.error(`${C.red}Eval error:${C.reset}`, err);
  }
}

if (watchMode) {
  const { watch } = await import("chokidar");

  console.log(`${C.cyan}Watching${C.reset} roles/grooming.toml — edit any role to trigger re-eval`);
  console.log(`${C.gray}Press Ctrl+C to stop${C.reset}\n`);

  await runAndCatch();

  let debounce: ReturnType<typeof setTimeout> | null = null;
  watch(GROOMING_TOML_PATH).on("change", () => {
    console.log(`\n${C.yellow}↺ Changed:${C.reset} grooming.toml`);
    GROOMING_ROLES = loadGroomingRoles(); // reload from disk
    if (debounce) clearTimeout(debounce);
    debounce = setTimeout(runAndCatch, 300);
  });
} else {
  await runAndCatch();
}
