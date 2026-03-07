/// Resolve a short model ID (stored in qa_config) to the model string passed to Claude Code.
/// Claude Code CLI understands "haiku", "sonnet", and "opus" directly.
pub fn resolve_model_id(short_id: &str) -> &'static str {
    match short_id {
        "haiku" => "haiku",
        "sonnet" => "sonnet",
        "opus" => "opus",
        _ => "sonnet",
    }
}
