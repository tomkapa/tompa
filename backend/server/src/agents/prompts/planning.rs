use std::sync::LazyLock;

use shared::enums::KnowledgeCategory;
use shared::types::{KnowledgeEntry, QaDecision};

#[derive(serde::Deserialize)]
struct PlanningConfig {
    system_template: super::TemplateConfig,
    user_template: super::TemplateConfig,
}

static CONFIG: LazyLock<PlanningConfig> = LazyLock::new(|| {
    toml::from_str(include_str!("roles/planning.toml")).expect("roles/planning.toml is valid TOML")
});

/// Returns `(system_prompt, user_prompt)`.
pub fn build_planning_prompt(
    story_description: &str,
    knowledge: &[KnowledgeEntry],
    codebase_context: &str,
    grooming_decisions: &[QaDecision],
    previous_decisions: &[QaDecision],
    detail_level_text: &str,
    max_questions: i64,
) -> (String, String) {
    let adrs = super::coalesce(super::filter_knowledge(knowledge, KnowledgeCategory::Adr), "None documented.");
    let api_docs = super::coalesce(super::filter_knowledge(knowledge, KnowledgeCategory::ApiDoc), "None documented.");
    let codebase = super::coalesce(codebase_context.to_owned(), "No codebase context available.");
    let grooming = super::fmt_decisions(grooming_decisions);
    let previous = super::fmt_decisions(previous_decisions);
    let max_q = max_questions.to_string();

    let system = super::render(
        &CONFIG.system_template.text,
        &[
            ("detail_level_text", detail_level_text),
            ("max_questions", &max_q),
        ],
    );
    let prompt = super::render(
        &CONFIG.user_template.text,
        &[
            ("adrs", &adrs),
            ("api_docs", &api_docs),
            ("codebase", &codebase),
            ("story", story_description),
            ("grooming", &grooming),
            ("previous", &previous),
            ("max_questions", &max_q),
        ],
    );
    (system, prompt)
}
