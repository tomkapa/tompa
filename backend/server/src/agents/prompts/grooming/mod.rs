use std::sync::LazyLock;

use shared::enums::KnowledgeCategory;
use shared::types::{KnowledgeEntry, QaDecision};

#[derive(Debug, serde::Deserialize)]
pub struct GroomingRole {
    pub id: String,
    pub title: String,
    pub domain: String,
    pub instructions: String,
}

#[derive(serde::Deserialize)]
pub struct GroomingConfig {
    pub system_template: super::TemplateConfig,
    pub sequential_system_template: super::TemplateConfig,
    pub user_template: super::TemplateConfig,
    pub sequential_user_template: super::TemplateConfig,
    pub roles: Vec<GroomingRole>,
}

/// All grooming prompt templates and role configs, loaded once from TOML.
/// Edit `roles/grooming.toml` to change prompts or role order — no Rust rewrite needed.
pub static GROOMING_CONFIG: LazyLock<GroomingConfig> = LazyLock::new(|| {
    toml::from_str(include_str!("../roles/grooming.toml")).expect("roles/grooming.toml is valid TOML")
});

/// Returns `(system_prompt, user_prompt)`.
#[allow(clippy::too_many_arguments)]
pub fn build_grooming_prompt(
    role: &GroomingRole,
    story_description: &str,
    knowledge: &[KnowledgeEntry],
    codebase_context: &str,
    previous_decisions: &[QaDecision],
    detail_level_text: &str,
    max_questions: i64,
    round_number: i32,
    prior_decision_count: usize,
    convergence_guidance: &str,
) -> (String, String) {
    let conventions = super::coalesce(super::filter_knowledge(knowledge, KnowledgeCategory::Convention), "None documented.");
    let adrs = super::coalesce(super::filter_knowledge(knowledge, KnowledgeCategory::Adr), "None documented.");
    let codebase = super::coalesce(codebase_context.to_owned(), "No codebase context available.");
    let previous = super::fmt_decisions(previous_decisions);
    let max_q = max_questions.to_string();
    let round = round_number.to_string();
    let prior = prior_decision_count.to_string();

    let system = super::render(
        &GROOMING_CONFIG.system_template.text,
        &[
            ("role_title", &role.title),
            ("instructions", role.instructions.trim()),
            ("domain", &role.domain),
            ("detail_level_text", detail_level_text),
            ("max_questions", &max_q),
            ("round_number", &round),
            ("prior_decision_count", &prior),
            ("convergence_guidance", convergence_guidance),
        ],
    );
    let prompt = super::render(
        &GROOMING_CONFIG.user_template.text,
        &[
            ("conventions", &conventions),
            ("adrs", &adrs),
            ("codebase", &codebase),
            ("story", story_description),
            ("previous", &previous),
            ("role_title", &role.title),
            ("max_questions", &max_q),
        ],
    );
    (system, prompt)
}

/// A minimal view of an accumulated question used to build prompts for
/// subsequent roles (avoids importing the full server `QaQuestion` type here).
pub struct AccumulatedQuestion<'a> {
    pub index: usize,
    pub text: &'a str,
    pub domain: &'a str,
    pub rationale: &'a str,
    pub options: Vec<(&'a str, &'a str, &'a str)>, // (label, pros, cons)
}

/// Returns `(system_prompt, user_prompt)` for roles after the first.
#[allow(clippy::too_many_arguments)]
pub fn build_sequential_grooming_prompt(
    role: &GroomingRole,
    story_description: &str,
    knowledge: &[KnowledgeEntry],
    codebase_context: &str,
    previous_decisions: &[QaDecision],
    accumulated_questions: &[AccumulatedQuestion<'_>],
    detail_level_text: &str,
    max_questions: i64,
    round_number: i32,
    prior_decision_count: usize,
    convergence_guidance: &str,
) -> (String, String) {
    let conventions = super::coalesce(super::filter_knowledge(knowledge, KnowledgeCategory::Convention), "None documented.");
    let adrs = super::coalesce(super::filter_knowledge(knowledge, KnowledgeCategory::Adr), "None documented.");
    let codebase = super::coalesce(codebase_context.to_owned(), "No codebase context available.");
    let previous = super::fmt_decisions(previous_decisions);
    let existing = fmt_accumulated_questions(accumulated_questions);
    let max_q = max_questions.to_string();
    let round = round_number.to_string();
    let prior = prior_decision_count.to_string();

    let system = super::render(
        &GROOMING_CONFIG.sequential_system_template.text,
        &[
            ("role_title", &role.title),
            ("instructions", role.instructions.trim()),
            ("domain", &role.domain),
            ("detail_level_text", detail_level_text),
            ("max_questions", &max_q),
            ("round_number", &round),
            ("prior_decision_count", &prior),
            ("convergence_guidance", convergence_guidance),
        ],
    );
    let prompt = super::render(
        &GROOMING_CONFIG.sequential_user_template.text,
        &[
            ("conventions", &conventions),
            ("adrs", &adrs),
            ("codebase", &codebase),
            ("story", story_description),
            ("previous", &previous),
            ("existing", &existing),
            ("role_title", &role.title),
            ("max_questions", &max_q),
        ],
    );
    (system, prompt)
}

fn fmt_accumulated_questions(questions: &[AccumulatedQuestion<'_>]) -> String {
    if questions.is_empty() {
        return "None yet.".into();
    }
    questions
        .iter()
        .map(|q| {
            let options = q
                .options
                .iter()
                .enumerate()
                .map(|(i, (label, pros, cons))| {
                    format!("    Option {i}: {label}\n      Pros: {pros}\n      Cons: {cons}")
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "[{}] ({}) {}\n  Rationale: {}\n{}",
                q.index, q.domain, q.text, q.rationale, options
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}
