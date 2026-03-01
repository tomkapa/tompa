#![allow(dead_code)]

use shared::types::{KnowledgeEntry, QaDecision, TaskContext};

pub fn build_task_qa_prompt(context: &TaskContext, previous_decisions: &[QaDecision]) -> String {
    let knowledge = fmt_all_knowledge(&context.knowledge);
    let story_decisions = fmt_decisions(&context.story_decisions);
    let sibling_decisions = fmt_decisions(&context.sibling_decisions);
    let previous = fmt_decisions(previous_decisions);

    format!(
        r#"You are a senior developer reviewing an implementation task before writing code.

Focus on concrete implementation decisions: which library or function to use, \
how to handle a specific edge case, naming conventions for this feature, \
data validation rules, and integration points with existing code. \
Do NOT ask high-level architecture questions — those were decided in planning.

## Knowledge Base
{knowledge}

## Story-Level Decisions
{story_decisions}

## Decisions from Sibling Tasks
{sibling_decisions}

## Task Description
{task}

## Decisions Already Made for This Task
{previous}

Generate 2–4 specific, implementation-level questions.
Each question must have 2–5 concise answer options.
Do NOT re-ask anything already decided above.

Respond ONLY with valid JSON — no markdown fences, no extra text:
{{
  "questions": [
    {{
      "text": "Your question here?",
      "domain": "development",
      "options": ["Option A", "Option B"]
    }}
  ]
}}"#,
        knowledge = knowledge,
        story_decisions = story_decisions,
        sibling_decisions = if sibling_decisions.is_empty() {
            "None yet.".into()
        } else {
            sibling_decisions
        },
        task = context.task_description,
        previous = previous,
    )
}

fn fmt_all_knowledge(knowledge: &[KnowledgeEntry]) -> String {
    if knowledge.is_empty() {
        return "None documented.".into();
    }
    knowledge
        .iter()
        .map(|k| format!("### {} [{:?}]\n{}", k.title, k.category, k.content))
        .collect::<Vec<_>>()
        .join("\n\n")
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
