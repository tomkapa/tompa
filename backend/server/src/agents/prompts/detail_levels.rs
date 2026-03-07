/// Map a detail level integer (1–5) to the significance threshold description
/// injected into grooming/planning/implementation system prompts.
pub fn detail_level_threshold(level: i64) -> &'static str {
    match level {
        1 => {
            "Only raise questions about decisions that would be impossible or extremely expensive to reverse after implementation."
        }
        2 => {
            "Only raise questions about decisions that would require days of significant rework to change later."
        }
        3 => {
            "Raise questions about decisions that require meaningful effort to reverse or that meaningfully affect quality."
        }
        4 => {
            "Raise questions about decisions that could cause noticeable inefficiency, technical debt, or user experience degradation."
        }
        5 => {
            "Raise questions about all decisions where reasonable professionals might disagree on the best approach."
        }
        _ => {
            "Raise questions about decisions that require meaningful effort to reverse or that meaningfully affect quality."
        }
    }
}
