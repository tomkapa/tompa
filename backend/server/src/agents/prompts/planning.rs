use shared::enums::KnowledgeCategory;
use shared::types::{KnowledgeEntry, QaDecision};

/// Returns `(system_prompt, prompt)`.
pub fn build_planning_prompt(
    story_description: &str,
    knowledge: &[KnowledgeEntry],
    codebase_context: &str,
    grooming_decisions: &[QaDecision],
    previous_decisions: &[QaDecision],
    detail_level_text: &str,
    max_questions: i64,
) -> (String, String) {
    let adrs = filter_knowledge(knowledge, KnowledgeCategory::Adr);
    let api_docs = filter_knowledge(knowledge, KnowledgeCategory::ApiDoc);
    let grooming = fmt_decisions(grooming_decisions);
    let previous = fmt_decisions(previous_decisions);

    let system = format!(
        r#"You are a senior software architect performing technical planning for a story.

Focus on: system architecture choices, database schema design, API contract design, \
error handling strategy, concurrency / transaction considerations, and inter-service \
dependencies. Do NOT re-ask questions already decided during grooming.

QUESTION SCOPE: {detail_level_text}
QUESTION LIMIT: Generate at most {max_questions} technical planning questions. Prioritize by impact — if you have more potential questions than your limit, keep only the most consequential ones.

For each question:
- "rationale": One sentence explaining why this decision matters and its downstream consequences. Be specific to the story context.
- "options": Each option is an object with "label" (concise choice), "pros" (2–4 sentences, honest advantages), and "cons" (2–4 sentences, honest disadvantages).
- "recommended_option_index": Zero-based index of the option you recommend, grounded in the story context.

If all critical decisions have already been made and you have no further questions, return `{{"questions": []}}`.

Respond ONLY with valid JSON — no markdown fences, no extra text:
{{
  "questions": [
    {{
      "text": "Your question here?",
      "domain": "planning",
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
        detail_level_text = detail_level_text,
        max_questions = max_questions,
    );

    let prompt = format!(
        r#"## Architecture Decision Records
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

Generate 0–{max_questions} technical planning questions.
Each question must have 2–5 mutually-exclusive answer options.
Do NOT ask questions already answered above."#,
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
        codebase = if codebase_context.is_empty() {
            "No codebase context available.".into()
        } else {
            codebase_context.to_owned()
        },
        story = story_description,
        grooming = grooming,
        previous = previous,
        max_questions = max_questions,
    );

    (system, prompt)
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
