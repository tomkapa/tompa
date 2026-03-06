use shared::types::{KnowledgeEntry, QaDecision};

/// Returns `(system_prompt, prompt)`.
pub fn build_implementation_prompt(
    task_description: &str,
    knowledge: &[KnowledgeEntry],
    story_decisions: &[QaDecision],
    sibling_decisions: &[QaDecision],
) -> (String, String) {
    let knowledge_text = fmt_all_knowledge(knowledge);
    let story_text = fmt_decisions(story_decisions);
    let sibling_text = fmt_decisions(sibling_decisions);

    let system =
        r#"You are implementing a task as part of a larger story. Follow all decisions exactly.

## Communication Protocol
When you need a human decision before continuing, output EXACTLY this on its own line:
[DECISION_NEEDED]
Then on the VERY NEXT line output a JSON object (no whitespace before it):
{"text":"Your question?","domain":"development","options":["Option A","Option B"]}

When the task is fully implemented and committed, output EXACTLY this on its own line:
[COMPLETED]<commit_sha>

Output all other progress information as plain text lines (the supervisor forwards them as \
status updates)."#
            .to_owned();

    let prompt = format!(
        r#"## Knowledge Base
{knowledge}

## Story-Level Decisions
{story_decisions}

## Decisions from Sibling Tasks
{sibling_decisions}

## Task to Implement
{task}

Implement the task now. Follow all decisions above. Ask via [DECISION_NEEDED] if and only \
if you encounter an ambiguity not covered by the decisions. Do not ask about anything already \
decided. When done, commit your changes and output [COMPLETED]<commit_sha>."#,
        knowledge = knowledge_text,
        story_decisions = story_text,
        sibling_decisions = if sibling_text.is_empty() {
            "None yet.".into()
        } else {
            sibling_text
        },
        task = task_description,
    );

    (system, prompt)
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
