use std::sync::LazyLock;

use shared::types::QaDecision;

#[derive(serde::Deserialize)]
struct RefinementConfig {
    system_template: super::TemplateConfig,
    user_template: super::TemplateConfig,
}

static CONFIG: LazyLock<RefinementConfig> = LazyLock::new(|| {
    toml::from_str(include_str!("roles/description_refinement.toml"))
        .expect("roles/description_refinement.toml is valid TOML")
});

/// Returns `(system_prompt, user_prompt)`.
pub fn build_refinement_prompt(
    story_title: &str,
    story_description: &str,
    decisions: &[QaDecision],
    stage: &str,
) -> (String, String) {
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
    let decisions_text = if decisions.is_empty() {
        "None.".to_owned()
    } else {
        decisions
            .iter()
            .map(|d| format!("- [{}] Q: {} → A: {}", d.domain, d.question_text, d.answer_text))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let system = super::render(&CONFIG.system_template.text, &[("stage_label", stage_label)]);
    let prompt = super::render(
        &CONFIG.user_template.text,
        &[
            ("story_title", story_title),
            ("description_section", &description_section),
            ("stage_label", stage_label),
            ("decisions_text", &decisions_text),
        ],
    );
    (system, prompt)
}
