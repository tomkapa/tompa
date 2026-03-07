use shared::types::QaDecision;

/// Returns `(system_prompt, prompt)`.
pub fn build_refinement_prompt(
    story_title: &str,
    story_description: &str,
    decisions: &[QaDecision],
    stage: &str,
) -> (String, String) {
    let decisions_text = if decisions.is_empty() {
        "None.".into()
    } else {
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
    };

    let stage_label = match stage {
        "grooming" => "Grooming",
        "planning" => "Planning",
        _ => stage,
    };

    let description_section = if story_description.trim().is_empty() {
        "(No description provided — elaborate the title into a full story description.)".to_owned()
    } else {
        story_description.to_owned()
    };

    let system = format!(
        "You are refining a story description after {stage_label} Q&A has converged.\n\n\
        Your task: Write a comprehensive, refined story description that incorporates ALL of the \
        decisions into the original description.\n\n\
        Format the output as **Markdown** using this structure:\n\
        - A brief overview paragraph\n\
        - `## Requirements` section with a bullet list of concrete requirements\n\
        - `## Constraints & Decisions` section integrating the Q&A decisions\n\
        - `## Acceptance Criteria` as a numbered list\n\n\
        If the story involves a user flow, data pipeline, or architecture interaction that benefits \
        from a visual diagram, include a Mermaid flow diagram in a ```mermaid code block \
        (flowchart LR or TD, max 10 nodes, keep it focused).\n\n\
        If no description or decisions are provided, write a clear story description based \
        solely on the story title.\n\n\
        Output ONLY the Markdown document — no JSON, no extra preamble.",
    );

    let prompt = format!(
        r#"## Story Title
{story_title}

## Original Description
{description_section}

## Decisions Made During {stage_label}
{decisions_text}"#,
    );

    (system, prompt)
}
