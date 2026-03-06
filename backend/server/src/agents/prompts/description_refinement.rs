use shared::types::QaDecision;

/// Returns `(system_prompt, prompt)`.
pub fn build_refinement_prompt(
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

    let system = format!(
        "You are refining a story description after {stage_label} Q&A has converged.\n\n\
        Your task: Write a comprehensive, refined story description that incorporates ALL of the \
        decisions into the original description. The refined description should:\n\n\
        1. Preserve the original intent and scope of the story\n\
        2. Integrate every decision as concrete requirements or constraints\n\
        3. Be written in clear, specific language suitable for implementation\n\
        4. Organize information logically (overview, requirements, constraints, acceptance criteria)\n\
        5. Remove any ambiguity that was resolved by the Q&A decisions\n\n\
        Output ONLY the refined description text — no JSON, no markdown fences, no preamble.",
    );

    let prompt = format!(
        r#"## Original Story Description
{story_description}

## Decisions Made During {stage_label}
{decisions_text}"#,
    );

    (system, prompt)
}
