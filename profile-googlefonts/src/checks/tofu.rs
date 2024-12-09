use crate::metadata::{family_proto, FamilyProto};
use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use google_fonts_languages::{LanguageProto, LANGUAGES};
use google_fonts_subsets::SUBSETS;
use std::collections::HashSet;
use unicode_normalization::UnicodeNormalization;
use unicode_properties::{GeneralCategoryGroup, UnicodeGeneralCategory};

struct OurLang<'a> {
    id: String,
    name: String,
    bases: Vec<char>,
    samples: Vec<(&'a str, String)>,
    supported: Option<String>,
}

fn parse_chars(chars: &str) -> HashSet<char> {
    chars
        .split_whitespace()
        .flat_map(|x| {
            let mut s = x.to_string();
            if s.len() > 1 {
                s = s.trim_start_matches("{").trim_end_matches("}").to_string()
            }
            let normalized = s.nfc().collect::<String>();
            if normalized != s {
                vec![s, normalized]
            } else {
                vec![s]
            }
        })
        .filter(|x| !x.is_empty())
        .flat_map(|x| x.chars().collect::<Vec<char>>())
        .collect()
}

impl OurLang<'_> {
    fn new(lang: &LanguageProto, charset: &HashSet<char>, font_proto: &FamilyProto) -> Self {
        let bases: HashSet<char> = lang
            .exemplar_chars
            .as_ref()
            .map_or(HashSet::new(), |e| parse_chars(e.base()));
        let samples = lang.sample_text.as_ref().map_or(vec![], |s| {
            vec![
                ("masthead full", s.masthead_full().to_string()),
                ("masthead partial", s.masthead_partial().to_string()),
                ("poster lg", s.poster_lg().to_string()),
                ("poster md", s.poster_md().to_string()),
                ("poster sm", s.poster_sm().to_string()),
                ("specimen 16", s.specimen_16().to_string()),
                ("specimen 21", s.specimen_21().to_string()),
                ("specimen 32", s.specimen_32().to_string()),
            ]
            .iter()
            .filter(|(_title, s)| !s.is_empty())
            .map(|(title, s)| (*title, s.clone()))
            .collect()
        });

        let supported = if font_proto.primary_language() == lang.id() {
            Some("primary language was set".to_string())
        } else if !bases.is_empty() && charset.is_superset(&bases) {
            Some("the font contained all the base exemplars for the language".to_string())
        } else {
            let sample_set: HashSet<char> = samples
                .iter()
                .flat_map(|(_title, s)| s.chars())
                .filter(|c| {
                    !(c.is_whitespace()
                        || matches!(
                            c.general_category_group(),
                            GeneralCategoryGroup::Punctuation
                        ))
                })
                .collect();
            if !sample_set.is_empty() && charset.is_superset(&sample_set) {
                Some("the font contained all the codepoints for the sample text".to_string())
            } else {
                None
            }
        };
        // if supported.is_some() {
        // println!("{}: {:?}", lang.name(), supported);
        // }
        OurLang {
            id: lang.id().to_owned(),
            name: lang.name().to_owned(),
            bases: bases.into_iter().collect(),
            samples,
            supported,
        }
    }

    fn find_problems(
        &self,
        codepoints: &HashSet<char>,
        subsets: &[(&str, &[u32])],
        context: &Context,
    ) -> Vec<Status> {
        let mut problems = vec![];
        let mut missing_codepoints = HashSet::new();
        for (sample_name, sample) in self.samples.iter() {
            let sample = sample.replace("\u{0a}", "");
            let missing_for_sample: HashSet<char> =
                sample.chars().filter(|c| !codepoints.contains(c)).collect();
            let unique_missing: HashSet<char> = missing_for_sample
                .difference(&missing_codepoints.clone())
                .copied()
                .collect();
            if !unique_missing.is_empty() {
                let mut reason = "missing-codepoints";
                let mut supplement = "";
                if self.supported
                    == Some(
                        "the font contained all the base exemplars for the language".to_string(),
                    )
                {
                    reason = "bad-sample-text";
                    supplement = " (This suggests there's something wrong with the sample text)";
                }
                #[allow(clippy::unwrap_used)]
                problems.push(Status::fail(
                    reason,
                    &format!(
                    "We detected support for the {} language because {}, but the font is missing the following codepoints needed to render the {} sample text{}:\n{}",
                    self.name, self.supported.as_ref().unwrap(), sample_name, supplement,
                    bullet_list(context,unique_missing.iter().map(|c| format!("{} (U+{:04X})", c, *c as u32)).collect::<Vec<_>>())                    
                )));
                missing_codepoints.extend(unique_missing);
            }
        }
        if !problems.is_empty() {
            return problems;
        }
        // Now check the effect of subsetting
        let subsetted_codepoints = subsets
            .iter()
            .flat_map(|(_name, chars)| {
                codepoints
                    .intersection(
                        &chars
                            .iter()
                            .flat_map(|c| char::from_u32(*c))
                            .collect::<HashSet<_>>(),
                    )
                    .copied()
                    .collect::<Vec<_>>()
            })
            .collect::<HashSet<char>>();
        let mut missing_codepoints = HashSet::new();
        for (sample_name, sample) in self.samples.iter() {
            let sample = sample.replace("\u{0a}", "");
            let missing_for_sample: HashSet<char> = sample
                .chars()
                .filter(|c| !subsetted_codepoints.contains(c))
                .collect();
            let unique_missing: HashSet<char> = missing_for_sample
                .difference(&missing_codepoints.clone())
                .copied()
                .collect();
            if !unique_missing.is_empty() {
                #[allow(clippy::unwrap_used)]
                problems.push(Status::fail(
                    "missing-subsetted",
                    &format!(
                    "The font has the following codepoints needed to render the {} sample text for language {}, but although {}, tofu will still be produced because the codepoints do not appear in any of the subsets {}:\n{}",
                    sample_name, self.name, self.supported.as_ref().unwrap(),
                    subsets.iter().map(|(name, _)| name.to_string()).collect::<Vec<_>>().join(", "), 
                     bullet_list(context,unique_missing.iter().map(|c| format!("{} (U+{:04X})", c, *c as u32)).collect::<Vec<_>>())
                )));
                missing_codepoints.extend(unique_missing);
            }
        }

        problems
    }
}

#[check(
    id = "googlefonts/tofu",
    rationale = "
        When the Google Fonts backend determines that a language is supported for a font,
        then the font should not show tofu for the sample texts for that language.
    ",
    proposal = "",
    title = "Tofu should not be shown in sample texts.",
    implementation = "all"
)]
fn googlefonts_tofu(c: &TestableCollection, context: &Context) -> CheckFnResult {
    let mdpb = c
        .get_file("METADATA.pb")
        .ok_or_else(|| CheckError::skip("no-mdpb", "No METADATA.pb file found"))?;
    let msg = family_proto(mdpb)?;
    let testable = msg
        .fonts
        .first()
        .and_then(|f| f.filename.as_ref())
        .and_then(|f| c.get_file(f))
        .ok_or_else(|| CheckError::skip("no-fonts", "No font files found in METADATA.pb"))?;
    let font = testfont!(testable);
    // Determine language support: (a) is primary_language in metadata, or (b) supports all base
    // exemplars for a lang or (c) support all sample text except punctuation and spaces
    let codepoints = font
        .codepoints(Some(context))
        .iter()
        .flat_map(|cp| char::from_u32(*cp))
        .collect();
    let supported: Vec<OurLang> = LANGUAGES
        .iter()
        .map(|(_name, lang)| OurLang::new(lang, &codepoints, &msg))
        .filter(|l| l.supported.is_some())
        .collect();
    let font_subsets = msg.subsets.into_iter().collect::<HashSet<String>>();
    let subsets: Vec<(&str, _)> = SUBSETS
        .into_iter()
        .filter(|(name, _subset)| font_subsets.contains(*name))
        .collect();

    let problems = supported
        .iter()
        .flat_map(|ourlang| ourlang.find_problems(&codepoints, &subsets, context))
        .collect();
    return_result(problems)
}
