use std::sync::LazyLock;

use shared::types::{KnowledgeEntry, QaDecision};

#[derive(serde::Deserialize)]
struct TaskQaConfig {
    system_template: super::TemplateConfig,
    user_template: super::TemplateConfig,
}

static CONFIG: LazyLock<TaskQaConfig> = LazyLock::new(|| {
    toml::from_str(include_str!("roles/task_qa.toml")).expect("roles/task_qa.toml is valid TOML")
});

/// Returns `(system_prompt, user_prompt)`.
pub fn build_task_qa_prompt(
    task_description: &str,
    knowledge: &[KnowledgeEntry],
    story_decisions: &[QaDecision],
    sibling_decisions: &[QaDecision],
    previous_decisions: &[QaDecision],
) -> (String, String) {
    let knowledge_text = super::fmt_all_knowledge(knowledge);
    let story_text = super::fmt_decisions(story_decisions);
    let sibling_text = super::fmt_decisions(sibling_decisions);
    let previous = super::fmt_decisions(previous_decisions);

    let system = CONFIG.system_template.text.trim().to_owned();
    let prompt = super::render(
        &CONFIG.user_template.text,
        &[
            ("knowledge", &knowledge_text),
            ("story_decisions", &story_text),
            ("sibling_decisions", &sibling_text),
            ("task", task_description),
            ("previous", &previous),
        ],
    );
    (system, prompt)
}
