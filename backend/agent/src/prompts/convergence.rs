use shared::types::QaDecision;

/// Build a prompt asking Claude Code whether enough information has been gathered
/// to proceed to the next pipeline stage.
///
/// The model must respond with either the word `SUFFICIENT` or `CONTINUE`.
pub fn build_convergence_prompt(story_description: &str, decisions: &[QaDecision]) -> String {
    let decisions_text = if decisions.is_empty() {
        "None yet.".into()
    } else {
        decisions
            .iter()
            .map(|d| format!("- [{}] Q: {} → A: {}", d.domain, d.question_text, d.answer_text))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"You are evaluating whether enough information has been gathered to proceed.

## Story Description
{story}

## Decisions Made So Far
{decisions}

Assess whether the decisions above provide sufficient clarity to move to the next stage.
Consider: Are critical ambiguities resolved? Are scope boundaries clear? \
Are there obvious gaps that would cause rework?

Respond with ONLY one of these two words — nothing else:
- SUFFICIENT  (enough information to proceed)
- CONTINUE    (more questions are needed)"#,
        story = story_description,
        decisions = decisions_text,
    )
}
