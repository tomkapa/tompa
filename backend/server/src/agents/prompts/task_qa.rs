use shared::types::{KnowledgeEntry, QaDecision};

/// Returns `(system_prompt, prompt)`.
pub fn build_task_qa_prompt(
    task_description: &str,
    knowledge: &[KnowledgeEntry],
    story_decisions: &[QaDecision],
    sibling_decisions: &[QaDecision],
    previous_decisions: &[QaDecision],
) -> (String, String) {
    let knowledge_text = fmt_all_knowledge(knowledge);
    let story_text = fmt_decisions(story_decisions);
    let sibling_text = fmt_decisions(sibling_decisions);
    let previous = fmt_decisions(previous_decisions);

    let system = r#"You are a senior developer reviewing an implementation task before writing code.

Focus on concrete implementation decisions: which library or function to use, \
how to handle a specific edge case, naming conventions for this feature, \
data validation rules, and integration points with existing code. \
Do NOT ask high-level architecture questions — those were decided in planning.

For each question:
- "rationale": One sentence explaining why this decision matters and its downstream consequences. Be specific to the task context.
- "options": Each option is an object with "label" (concise choice), "pros" (2–4 sentences, honest advantages), and "cons" (2–4 sentences, honest disadvantages).
- "recommended_option_index": Zero-based index of the option you recommend, grounded in the task context.

If all critical decisions have already been made and you have no further questions, return `{"questions": []}`.

Respond ONLY with valid JSON — no markdown fences, no extra text:
{
  "questions": [
    {
      "text": "Your question here?",
      "domain": "development",
      "rationale": "This decision matters because...",
      "recommended_option_index": 0,
      "options": [
        {
          "label": "Option A",
          "pros": "Advantages of option A.",
          "cons": "Disadvantages of option A."
        },
        {
          "label": "Option B",
          "pros": "Advantages of option B.",
          "cons": "Disadvantages of option B."
        }
      ]
    }
  ]
}"#
    .to_owned();

    let prompt = format!(
        r#"## Knowledge Base
{knowledge}

## Story-Level Decisions
{story_decisions}

## Decisions from Sibling Tasks
{sibling_decisions}

## Task Description
{task}

## Decisions Already Made for This Task
{previous}

Generate 0–4 specific, implementation-level questions.
Each question must have 2–5 mutually-exclusive answer options.
Do NOT re-ask anything already decided above."#,
        knowledge = knowledge_text,
        story_decisions = story_text,
        sibling_decisions = if sibling_text.is_empty() {
            "None yet.".into()
        } else {
            sibling_text
        },
        task = task_description,
        previous = previous,
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
