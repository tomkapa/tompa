use shared::types::QaDecision;

/// Build a prompt instructing the model to produce a refined story description
/// incorporating all accumulated Q&A decisions from the given stage.
///
/// Plain text output (same pattern as `assess_convergence` — no JSON).
pub fn build_refinement_prompt(
    story_description: &str,
    decisions: &[QaDecision],
    stage: &str,
) -> String {
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

    format!(
        r#"You are refining a story description after {stage_label} Q&A has converged.

## Original Story Description
{story_description}

## Decisions Made During {stage_label}
{decisions_text}

Your task: Write a comprehensive, refined story description that incorporates ALL of the decisions above into the original description. The refined description should:

1. Preserve the original intent and scope of the story
2. Integrate every decision as concrete requirements or constraints
3. Be written in clear, specific language suitable for implementation
4. Organize information logically (overview, requirements, constraints, acceptance criteria)
5. Remove any ambiguity that was resolved by the Q&A decisions

Output ONLY the refined description text — no JSON, no markdown fences, no preamble."#,
    )
}
