pub mod business;
pub mod design;
pub mod development;
pub mod marketing;
pub mod security;

use shared::enums::KnowledgeCategory;
use shared::types::{KnowledgeEntry, QaDecision};

pub struct GroomingRole {
    pub id: &'static str,
    pub title: &'static str,
    pub domain: &'static str,
    pub instructions: &'static str,
}

/// Roles are called sequentially in this order:
/// business first (sets scope), then dev/design (technical), then security/marketing (cross-cutting).
pub const GROOMING_ROLES: &[GroomingRole] = &[
    GroomingRole {
        id: "business_analyst",
        title: business::ROLE_TITLE,
        domain: business::DOMAIN,
        instructions: business::INSTRUCTIONS,
    },
    GroomingRole {
        id: "developer",
        title: development::ROLE_TITLE,
        domain: development::DOMAIN,
        instructions: development::INSTRUCTIONS,
    },
    GroomingRole {
        id: "ux_designer",
        title: design::ROLE_TITLE,
        domain: design::DOMAIN,
        instructions: design::INSTRUCTIONS,
    },
    GroomingRole {
        id: "security_engineer",
        title: security::ROLE_TITLE,
        domain: security::DOMAIN,
        instructions: security::INSTRUCTIONS,
    },
    GroomingRole {
        id: "marketing",
        title: marketing::ROLE_TITLE,
        domain: marketing::DOMAIN,
        instructions: marketing::INSTRUCTIONS,
    },
];

/// Returns `(system_prompt, prompt)`.
pub fn build_grooming_prompt(
    role: &GroomingRole,
    story_description: &str,
    knowledge: &[KnowledgeEntry],
    codebase_context: &str,
    previous_decisions: &[QaDecision],
) -> (String, String) {
    let conventions = filter_knowledge(knowledge, KnowledgeCategory::Convention);
    let adrs = filter_knowledge(knowledge, KnowledgeCategory::Adr);
    let previous = fmt_decisions(previous_decisions);

    let system = format!(
        r#"You are a {role_title} participating in a software story grooming session.

{instructions}

For each question:
- "rationale": One sentence explaining why this decision matters and its downstream consequences. Be specific to the story context.
- "options": Each option is an object with "label" (concise choice), "pros" (2–4 sentences, honest advantages), and "cons" (2–4 sentences, honest disadvantages).
- "recommended_option_index": Zero-based index of the option you recommend, grounded in the story context.

If all critical decisions for your domain have already been made and you have no further questions, return `{{"questions": []}}`.

Respond ONLY with valid JSON in exactly this format — no other text, no markdown fences:
{{
  "questions": [
    {{
      "text": "Your question here?",
      "domain": "{domain}",
      "rationale": "This decision matters because...",
      "recommended_option_index": 0,
      "options": [
        {{
          "label": "Option A",
          "pros": "Advantages of option A.",
          "cons": "Disadvantages of option A."
        }},
        {{
          "label": "Option B",
          "pros": "Advantages of option B.",
          "cons": "Disadvantages of option B."
        }}
      ]
    }}
  ]
}}"#,
        role_title = role.title,
        instructions = role.instructions,
        domain = role.domain,
    );

    let prompt = format!(
        r#"## Organization Conventions
{conventions}

## Architecture Decision Records
{adrs}

## Codebase Context
{codebase}

## Story Description
{story}

## Decisions Already Made
{previous}

Based on your {role_title} perspective, generate 0–4 clarifying questions about this story.
Each question must have 2–5 mutually-exclusive predefined answer options.
Do NOT ask questions already answered in "Decisions Already Made"."#,
        conventions = if conventions.is_empty() {
            "None documented.".into()
        } else {
            conventions
        },
        adrs = if adrs.is_empty() {
            "None documented.".into()
        } else {
            adrs
        },
        codebase = if codebase_context.is_empty() {
            "No codebase context available.".into()
        } else {
            codebase_context.to_owned()
        },
        story = story_description,
        previous = previous,
        role_title = role.title,
    );

    (system, prompt)
}

/// A minimal view of an accumulated question used to build prompts for
/// subsequent roles (avoids importing the full server `QaQuestion` type here).
pub struct AccumulatedQuestion<'a> {
    pub index: usize,
    pub text: &'a str,
    pub domain: &'a str,
    pub rationale: &'a str,
    pub options: Vec<(&'a str, &'a str, &'a str)>, // (label, pros, cons)
}

/// Build the prompt for a role that runs AFTER the first role.
/// The accumulated questions from previous roles are included so this role can
/// either augment existing questions (add its perspective to pros/cons) or raise
/// entirely new questions.
///
/// Returns `(system_prompt, prompt)`.
pub fn build_sequential_grooming_prompt(
    role: &GroomingRole,
    story_description: &str,
    knowledge: &[KnowledgeEntry],
    codebase_context: &str,
    previous_decisions: &[QaDecision],
    accumulated_questions: &[AccumulatedQuestion<'_>],
) -> (String, String) {
    let conventions = filter_knowledge(knowledge, KnowledgeCategory::Convention);
    let adrs = filter_knowledge(knowledge, KnowledgeCategory::Adr);
    let previous = fmt_decisions(previous_decisions);
    let existing = fmt_accumulated_questions(accumulated_questions);

    let system = format!(
        r#"You are a {role_title} participating in a software story grooming session.

{instructions}

Other roles have already raised the questions listed under "Existing Questions".
Your job is to:
1. Augment existing questions where you have additional perspective — add your angle to the pros/cons of each relevant option.
2. Raise NEW questions that are not yet covered by any existing question.

Response format — ONLY valid JSON, no markdown fences:
{{
  "augmentations": [
    {{
      "question_index": 0,
      "rationale_addition": "From {domain} perspective: why this decision also matters for your domain.",
      "options": [
        {{
          "pros_addition": "Additional advantage from {domain} view.",
          "cons_addition": "Additional drawback from {domain} view."
        }}
      ]
    }}
  ],
  "questions": [
    {{
      "text": "New question not already covered?",
      "domain": "{domain}",
      "rationale": "Why this decision matters.",
      "recommended_option_index": 0,
      "options": [
        {{
          "label": "Option A",
          "pros": "Advantages.",
          "cons": "Disadvantages."
        }}
      ]
    }}
  ]
}}

Rules:
- "augmentations" contains one entry per existing question you want to enrich. "question_index" is the 0-based index from "Existing Questions".
- "options" in an augmentation must have exactly the same count as the original question's options. If you have nothing to add for an option use empty strings.
- "questions" contains only NEW questions not already covered; each needs 2–5 mutually-exclusive options.
- Return empty arrays when you have nothing to add: {{"augmentations": [], "questions": []}}."#,
        role_title = role.title,
        instructions = role.instructions,
        domain = role.domain,
    );

    let prompt = format!(
        r#"## Organization Conventions
{conventions}

## Architecture Decision Records
{adrs}

## Codebase Context
{codebase}

## Story Description
{story}

## Decisions Already Made
{previous}

## Existing Questions (raised by earlier roles)
{existing}

Based on your {role_title} perspective:
- Augment existing questions where you add meaningful cross-domain insight.
- Raise 0–3 NEW questions not already covered above.
Do NOT duplicate questions already in "Existing Questions"."#,
        conventions = if conventions.is_empty() {
            "None documented.".into()
        } else {
            conventions
        },
        adrs = if adrs.is_empty() {
            "None documented.".into()
        } else {
            adrs
        },
        codebase = if codebase_context.is_empty() {
            "No codebase context available.".into()
        } else {
            codebase_context.to_owned()
        },
        story = story_description,
        previous = previous,
        existing = existing,
        role_title = role.title,
    );

    (system, prompt)
}

fn fmt_accumulated_questions(questions: &[AccumulatedQuestion<'_>]) -> String {
    if questions.is_empty() {
        return "None yet.".into();
    }
    questions
        .iter()
        .map(|q| {
            let options = q
                .options
                .iter()
                .enumerate()
                .map(|(i, (label, pros, cons))| {
                    format!("    Option {i}: {label}\n      Pros: {pros}\n      Cons: {cons}")
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "[{}] ({}) {}\n  Rationale: {}\n{}",
                q.index, q.domain, q.text, q.rationale, options
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn filter_knowledge(knowledge: &[KnowledgeEntry], category: KnowledgeCategory) -> String {
    let items: Vec<String> = knowledge
        .iter()
        .filter(|k| k.category == category)
        .map(|k| format!("### {}\n{}", k.title, k.content))
        .collect();
    items.join("\n\n")
}

fn fmt_decisions(decisions: &[QaDecision]) -> String {
    if decisions.is_empty() {
        return "None yet.".into();
    }
    decisions
        .iter()
        .map(|d| {
            format!(
                "- [{}] Q: {} → A: {}",
                d.domain, d.question_text, d.answer_text
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
