use std::sync::LazyLock;

#[derive(serde::Deserialize)]
struct DetailLevelsConfig {
    thresholds: LevelTexts,
    convergence: LevelTexts,
    round_suffix: super::TemplateConfig,
}

#[derive(serde::Deserialize)]
struct LevelTexts {
    level_1: String,
    level_2: String,
    level_3: String,
    level_4: String,
    level_5: String,
    default: String,
}

static CONFIG: LazyLock<DetailLevelsConfig> = LazyLock::new(|| {
    toml::from_str(include_str!("roles/detail_levels.toml"))
        .expect("roles/detail_levels.toml is valid TOML")
});

/// Map a detail level integer (1–5) to the significance threshold description
/// injected into grooming/planning system prompts.
pub fn detail_level_threshold(level: i64) -> &'static str {
    let t = &CONFIG.thresholds;
    match level {
        1 => &t.level_1,
        2 => &t.level_2,
        3 => &t.level_3,
        4 => &t.level_4,
        5 => &t.level_5,
        _ => &t.default,
    }
}

/// Convergence guidance telling the LLM when to stop asking questions.
/// Combines detail-level expectations with round-aware pressure.
pub fn convergence_guidance(level: i64, round_number: i32) -> String {
    let c = &CONFIG.convergence;
    let expectation = match level {
        1 => c.level_1.as_str(),
        2 => c.level_2.as_str(),
        3 => c.level_3.as_str(),
        4 => c.level_4.as_str(),
        5 => c.level_5.as_str(),
        _ => c.default.as_str(),
    };
    if round_number > 1 {
        format!(
            "{} {}",
            expectation,
            super::render(
                &CONFIG.round_suffix.text,
                &[("round_number", &round_number.to_string())],
            )
        )
    } else {
        expectation.to_owned()
    }
}
