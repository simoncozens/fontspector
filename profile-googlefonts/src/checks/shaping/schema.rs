use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct ShapingInput {
    pub configuration: ShapingConfig,
    pub tests: Vec<ShapingTest>,
}

#[derive(Deserialize)]
pub struct ShapingConfig {
    #[serde(default)]
    pub defaults: ShapingOptions,
    #[serde(default)]
    pub forbidden_glyphs: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct ShapingTest {
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub only: Vec<String>,
    #[serde(flatten)]
    pub options: ShapingOptions,
    pub input: String,
    pub expectation: Option<String>,
    pub note: Option<String>,
}

impl ShapingTest {
    pub(crate) fn excluded(&self, file: &str) -> bool {
        let file = file.to_string();
        self.exclude.contains(&file) || (!self.only.is_empty() && !self.only.contains(&file))
    }

    pub(crate) fn note(&self) -> String {
        self.note
            .as_ref()
            .map(|n| format!(" ({})", n))
            .unwrap_or_default()
    }
}

#[derive(Deserialize, Clone, Default)]
pub struct ShapingOptions {
    pub script: Option<String>,
    pub language: Option<String>,
    pub direction: Option<String>,
    pub features: Option<HashMap<String, bool>>,
    pub shaper: Option<String>,
    pub variations: Option<HashMap<String, f32>>,
}

impl ShapingOptions {
    pub fn fill_from_defaults(&self, config: &ShapingConfig) -> ShapingOptions {
        ShapingOptions {
            script: self.script.clone().or(config.defaults.script.clone()),
            language: self.language.clone().or(config.defaults.language.clone()),
            direction: self.direction.clone().or(config.defaults.direction.clone()),
            features: self.features.clone().or(config.defaults.features.clone()),
            shaper: self.shaper.clone().or(config.defaults.shaper.clone()),
            variations: self
                .variations
                .clone()
                .or(config.defaults.variations.clone()),
        }
    }
}
