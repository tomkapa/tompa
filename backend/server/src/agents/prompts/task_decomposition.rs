use std::sync::LazyLock;

use shared::types::QaDecision;

#[derive(serde::Deserialize)]
struct DecompositionConfig {
    system_template: super::TemplateConfig,
    user_template: super::TemplateConfig,
}

static CONFIG: LazyLock<DecompositionConfig> = LazyLock::new(|| {
    toml::from_str(include_str!("roles/decomposition.toml")).expect("roles/decomposition.toml is valid TOML")
});

/// Returns `(system_prompt, user_prompt)`.
pub fn build_decomposition_prompt(
    story_description: &str,
    codebase_context: &str,
    grooming_decisions: &[QaDecision],
    planning_decisions: &[QaDecision],
) -> (String, String) {
    let grooming = super::fmt_decisions(grooming_decisions);
    let planning = super::fmt_decisions(planning_decisions);
    let codebase = super::coalesce(codebase_context.to_owned(), "No codebase context available.");

    let system = CONFIG.system_template.text.trim().to_owned();
    let prompt = super::render(
        &CONFIG.user_template.text,
        &[
            ("story", story_description),
            ("grooming", &grooming),
            ("planning", &planning),
            ("codebase", &codebase),
        ],
    );
    (system, prompt)
}
