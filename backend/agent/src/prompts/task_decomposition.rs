use shared::types::{PlanningContext, QaDecision};

pub fn build_decomposition_prompt(
    context: &PlanningContext,
    planning_decisions: &[QaDecision],
) -> String {
    let grooming = fmt_decisions(&context.grooming_decisions);
    let planning = fmt_decisions(planning_decisions);

    format!(
        r#"You are a senior engineer decomposing a story into atomic implementation tasks.

Rules:
- Each task must be completable in a single Claude Code session (roughly 10–15 file changes).
- Task types: "code" (implementation), "test" (test planning / test case generation), \
  "design" (wireframe or component description).
- For feature stories include design, test, and code tasks where appropriate.
- For bug stories use only "code" tasks.
- Assign positions starting at 1. Use depends_on to list position numbers of prerequisite tasks.
- Design and test-planning tasks may run in parallel; code tasks should depend on design.

## Story Description
{story}

## Grooming Decisions
{grooming}

## Planning Decisions
{planning}

## Codebase Context
{codebase}

Respond ONLY with valid JSON — no markdown fences, no extra text:
{{
  "tasks": [
    {{
      "name": "Short task name",
      "description": "Detailed description of exactly what to implement.",
      "task_type": "code",
      "position": 1,
      "depends_on": []
    }}
  ]
}}"#,
        story = context.story_description,
        grooming = grooming,
        planning = planning,
        codebase = if context.codebase_context.is_empty() {
            "No codebase context available.".into()
        } else {
            context.codebase_context.clone()
        },
    )
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
