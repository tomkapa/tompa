pub mod business;
pub mod design;
pub mod development;
pub mod marketing;
pub mod security;

use shared::enums::KnowledgeCategory;
use shared::types::{GroomingContext, KnowledgeEntry, QaDecision};

pub struct GroomingRole {
    pub id: &'static str,
    pub title: &'static str,
    pub domain: &'static str,
    pub instructions: &'static str,
}

pub const GROOMING_ROLES: &[GroomingRole] = &[
    GroomingRole {
        id: "business_analyst",
        title: business::ROLE_TITLE,
        domain: business::DOMAIN,
        instructions: business::INSTRUCTIONS,
    },
    GroomingRole {
        id: "ux_designer",
        title: design::ROLE_TITLE,
        domain: design::DOMAIN,
        instructions: design::INSTRUCTIONS,
    },
    GroomingRole {
        id: "developer",
        title: development::ROLE_TITLE,
        domain: development::DOMAIN,
        instructions: development::INSTRUCTIONS,
    },
    GroomingRole {
        id: "marketing",
        title: marketing::ROLE_TITLE,
        domain: marketing::DOMAIN,
        instructions: marketing::INSTRUCTIONS,
    },
    GroomingRole {
        id: "security_engineer",
        title: security::ROLE_TITLE,
        domain: security::DOMAIN,
        instructions: security::INSTRUCTIONS,
    },
];

pub fn build_grooming_prompt(
    role: &GroomingRole,
    context: &GroomingContext,
    previous_decisions: &[QaDecision],
) -> String {
    let conventions = filter_knowledge(&context.knowledge, KnowledgeCategory::Convention);
    let adrs = filter_knowledge(&context.knowledge, KnowledgeCategory::Adr);
    let previous = fmt_decisions(previous_decisions);

    format!(
        r#"You are a {role_title} participating in a software story grooming session.

{instructions}

## Organization Conventions
{conventions}

## Architecture Decision Records
{adrs}

## Codebase Context
{codebase}

## Story Description
{story}

## Decisions Already Made
{previous}

Based on your {role_title} perspective, generate 2–4 clarifying questions about this story.
Each question must have 2–5 concise, mutually-exclusive predefined answer options.
Do NOT ask questions already answered in "Decisions Already Made".

Respond ONLY with valid JSON in exactly this format — no other text, no markdown fences:
{{
  "questions": [
    {{
      "text": "Your question here?",
      "domain": "{domain}",
      "options": ["Option A", "Option B", "Option C"]
    }}
  ]
}}"#,
        role_title = role.title,
        instructions = role.instructions,
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
        codebase = if context.codebase_context.is_empty() {
            "No codebase context available.".into()
        } else {
            context.codebase_context.clone()
        },
        story = context.story_description,
        previous = previous,
        domain = role.domain,
    )
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
