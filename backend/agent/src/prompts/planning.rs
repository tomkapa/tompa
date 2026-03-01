use shared::enums::KnowledgeCategory;
use shared::types::{KnowledgeEntry, PlanningContext, QaDecision};

pub fn build_planning_prompt(
    context: &PlanningContext,
    previous_decisions: &[QaDecision],
) -> String {
    let adrs = filter_knowledge(&context.knowledge, KnowledgeCategory::Adr);
    let api_docs = filter_knowledge(&context.knowledge, KnowledgeCategory::ApiDoc);
    let grooming = fmt_decisions(&context.grooming_decisions);
    let previous = fmt_decisions(previous_decisions);

    format!(
        r#"You are a senior software architect performing technical planning for a story.

Focus on: system architecture choices, database schema design, API contract design, \
error handling strategy, concurrency / transaction considerations, and inter-service \
dependencies. Do NOT re-ask questions already decided during grooming.

## Architecture Decision Records
{adrs}

## API Documentation
{api_docs}

## Codebase Context
{codebase}

## Story Description
{story}

## Grooming Decisions (already resolved)
{grooming}

## Planning Decisions Already Made
{previous}

Generate 2–4 technical planning questions.
Each question must have 2–5 concise answer options.
Do NOT ask questions already answered above.

Respond ONLY with valid JSON — no markdown fences, no extra text:
{{
  "questions": [
    {{
      "text": "Your question here?",
      "domain": "planning",
      "options": ["Option A", "Option B"]
    }}
  ]
}}"#,
        adrs = if adrs.is_empty() {
            "None documented.".into()
        } else {
            adrs
        },
        api_docs = if api_docs.is_empty() {
            "None documented.".into()
        } else {
            api_docs
        },
        codebase = if context.codebase_context.is_empty() {
            "No codebase context available.".into()
        } else {
            context.codebase_context.clone()
        },
        story = context.story_description,
        grooming = grooming,
        previous = previous,
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
