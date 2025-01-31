mod forbidden;
mod regression;
pub(crate) mod schema;
use std::str::FromStr;

use fontspector_checkapi::{CheckError, Context, Testable};
pub use forbidden::forbidden;
pub use regression::regression;

use rustybuzz::{ttf_parser, Face, GlyphBuffer, UnicodeBuffer};
use schema::{ShapingConfig, ShapingInput, ShapingOptions, ShapingTest};

pub(crate) struct FailedCheck {
    test: ShapingTest,
    detail: String,
}

pub(crate) fn create_buffer_and_run(
    face: &mut Face,
    input: &str,
    options: &ShapingOptions,
) -> Result<GlyphBuffer, CheckError> {
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(input);
    if let Some(script) = options.script.as_deref() {
        buffer.set_script(
            rustybuzz::Script::from_str(script)
                .map_err(|e| CheckError::Error(format!("Bad 'script' argument {}", e)))?,
        );
    }
    if let Some(language) = options.language.as_deref() {
        buffer.set_language(
            rustybuzz::Language::from_str(language)
                .map_err(|e| CheckError::Error(format!("Bad 'language' argument {}", e)))?,
        );
    }
    if let Some(direction) = options.direction.as_deref() {
        buffer.set_direction(
            rustybuzz::Direction::from_str(direction)
                .map_err(|e| CheckError::Error(format!("Bad 'direction' argument {}", e)))?,
        );
    }
    let features = options
        .features
        .clone() // Urgh, to avoid partial move
        .unwrap_or_default()
        .iter()
        .map(|(tag, value)| {
            rustybuzz::Feature::new(
                ttf_parser::Tag::from_bytes_lossy(tag.as_bytes()),
                *value as u32,
                ..,
            )
        })
        .collect::<Vec<_>>();
    if let Some(ref variations) = options.variations {
        let rb_variations: Vec<_> = variations
            .iter()
            .map(|(tag, value)| rustybuzz::Variation {
                tag: ttf_parser::Tag::from_bytes_lossy(tag.as_bytes()),
                value: *value,
            })
            .collect();
        face.set_variations(&rb_variations);
    } else {
        face.set_variations(&[]);
    };

    Ok(rustybuzz::shape(face, &features, buffer))
}

pub(crate) trait ShapingCheck {
    fn run(
        &self,
        t: &Testable,
        context: &Context,
    ) -> Result<Vec<(String, Vec<FailedCheck>)>, CheckError> {
        let mut face = Face::from_slice(&t.contents, 0)
            .ok_or(CheckError::Error("Failed to load font file".to_string()))?;

        let basename = t.basename().unwrap_or_default();
        let mut results = vec![];
        let shaping_file = context
            .configuration
            .get("shaping")
            .and_then(|shaping| shaping.as_object())
            .and_then(|shaping| shaping.get("test_directory"))
            .and_then(|test_directory: &serde_json::Value| test_directory.as_str())
            .ok_or(CheckError::skip(
                "no-tests",
                "Shaping test directory not defined in configuration file",
            ))?;
        let files = glob::glob(&format!("{}/*.json", shaping_file))?.flatten();

        for file in files {
            let file_contents = std::fs::read_to_string(&file)?;
            let input: ShapingInput = serde_json::from_str(&file_contents)?;
            let config = input.configuration;
            let mut failed_checks = vec![];
            for test in input.tests {
                if !self.applies(&config, &test) || test.excluded(&basename) {
                    continue;
                }
                let options = test.options.fill_from_defaults(&config);
                // Run the test
                let glyph_buffer = create_buffer_and_run(&mut face, &test.input, &options)?;
                if let Some(res) = self.pass_fail(&test, &config, &glyph_buffer, &face) {
                    failed_checks.push(FailedCheck {
                        test: test.clone(),
                        detail: res,
                    });
                }
            }
            results.push((file.to_string_lossy().to_string(), failed_checks));
        }
        Ok(results)
    }

    fn applies(&self, configuration: &ShapingConfig, test: &ShapingTest) -> bool;

    fn pass_fail(
        &self,
        test: &ShapingTest,
        configuration: &ShapingConfig,
        buffer: &GlyphBuffer,
        face: &Face,
    ) -> Option<String>;
}
