pub mod description_refinement;
pub mod detail_levels;
pub mod grooming;
pub mod implementation;
pub mod models;
pub mod planning;
pub mod task_decomposition;
pub mod task_qa;

use shared::enums::KnowledgeCategory;
use shared::types::{KnowledgeEntry, QaDecision};

/// A single template string loaded from a TOML `[section]`.
#[derive(serde::Deserialize)]
pub struct TemplateConfig {
    pub text: String,
}

/// Substitute `{variable}` placeholders in `template` with the provided values.
/// Only exact matches are replaced; literal `{` in JSON schema examples are safe
/// because they are followed by `"` or whitespace, not an identifier.
pub fn render(template: &str, vars: &[(&str, &str)]) -> String {
    let mut s = template.trim().to_string();
    for (key, value) in vars {
        s = s.replace(&format!("{{{key}}}"), value);
    }
    s
}

/// Return `s` if non-empty, otherwise `fallback`.
pub(super) fn coalesce(s: String, fallback: &str) -> String {
    if s.is_empty() { fallback.to_owned() } else { s }
}

/// Format a slice of QA decisions as a bullet list, or "None yet." when empty.
pub(super) fn fmt_decisions(decisions: &[QaDecision]) -> String {
    if decisions.is_empty() {
        return "None yet.".into();
    }
    decisions
        .iter()
        .map(|d| format!("- [{}] Q: {} → A: {}", d.domain, d.question_text, d.answer_text))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Filter knowledge entries by category and format as Markdown sections.
pub(super) fn filter_knowledge(knowledge: &[KnowledgeEntry], category: KnowledgeCategory) -> String {
    let items: Vec<String> = knowledge
        .iter()
        .filter(|k| k.category == category)
        .map(|k| format!("### {}\n{}", k.title, k.content))
        .collect();
    items.join("\n\n")
}

/// Format all knowledge entries (all categories) as Markdown sections, or "None documented." when empty.
pub(super) fn fmt_all_knowledge(knowledge: &[KnowledgeEntry]) -> String {
    if knowledge.is_empty() {
        return "None documented.".into();
    }
    knowledge
        .iter()
        .map(|k| format!("### {} [{:?}]\n{}", k.title, k.category, k.content))
        .collect::<Vec<_>>()
        .join("\n\n")
}
