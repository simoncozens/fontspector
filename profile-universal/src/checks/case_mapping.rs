use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use markdown_table::{Heading, MarkdownTable};
use skrifa::MetadataProvider;
use unicode_properties::{GeneralCategoryGroup, UnicodeGeneralCategory};

fn swapcase(c: &char) -> Option<char> {
    if c.is_uppercase() {
        let mut lc = c.to_lowercase();
        if lc.len() == 1 {
            lc.next()
        } else {
            None
        }
    } else if c.is_lowercase() {
        let mut uc = c.to_uppercase();
        if uc.len() == 1 {
            uc.next()
        } else {
            None
        }
    } else {
        None
    }
}
const CASE_MAPPING_EXCEPTIONS: [u32; 22] = [
    0x0192, // ƒ - Latin Small Letter F with Hook
    0x00B5, // µ - Micro Sign
    0x03C0, // π - Greek Small Letter Pi
    0x2126, // Ω - Ohm Sign
    0x03BC, // μ - Greek Small Letter Mu
    0x03A9, // Ω - Greek Capital Letter Omega
    0x0394, // Δ - Greek Capital Letter Delta
    0x0251, // ɑ - Latin Small Letter Alpha
    0x0261, // ɡ - Latin Small Letter Script G
    0x00FF, // ÿ - Latin Small Letter Y with Diaeresis
    0x0250, // ɐ - Latin Small Letter Turned A
    0x025C, // ɜ - Latin Small Letter Reversed Open E
    0x0252, // ɒ - Latin Small Letter Turned Alpha
    0x0271, // ɱ - Latin Small Letter M with Hook
    0x0282, // ʂ - Latin Small Letter S with Hook
    0x029E, // ʞ - Latin Small Letter Turned K
    0x0287, // ʇ - Latin Small Letter Turned T
    0x0127, // ħ - Latin Small Letter H with Stroke
    0x0140, // ŀ - Latin Small Letter L with Middle Dot
    0x023F, // ȿ - Latin Small Letter S with Swash Tail
    0x0240, // ɀ - Latin Small Letter Z with Swash Tail
    0x026B, // ɫ - Latin Small Letter L with Middle Tilde
];

#[check(
    id = "case_mapping",
    rationale = "
        Ensure that no glyph lacks its corresponding upper or lower counterpart
        (but only when unicode supports case-mapping).
    ",
    proposal = "https://github.com/googlefonts/fontbakery/issues/3230",
    title = "Ensure the font supports case swapping for all its glyphs."
)]
fn case_mapping(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut missing_counterparts_table = vec![];
    for codepoint in f.codepoints() {
        if CASE_MAPPING_EXCEPTIONS.contains(&codepoint) {
            continue;
        }
        if let Some(c) = char::from_u32(codepoint)
            .filter(|c| matches!(c.general_category_group(), GeneralCategoryGroup::Letter))
        {
            if let Some(swapped) = swapcase(&c) {
                if f.font().charmap().map(swapped as u32).is_none() {
                    let have = format!(
                        "U+{:04X}: {}",
                        codepoint,
                        unicode_names2::name(c)
                            .map(|s| s.to_string())
                            .unwrap_or("Unknown".to_string()),
                    );
                    let have_not = format!(
                        "U+{:04X}: {}",
                        swapped as u32,
                        unicode_names2::name(swapped)
                            .map(|s| s.to_string())
                            .unwrap_or("Unknown".to_string()),
                    );
                    missing_counterparts_table.push(vec![have, have_not]);
                }
            }
        }
    }
    if missing_counterparts_table.is_empty() {
        Ok(Status::just_one_pass())
    } else {
        let mut table = MarkdownTable::new(missing_counterparts_table);
        table.with_headings(vec![
            Heading::new("Glyph present in the font".to_string(), None),
            Heading::new("Missing case-swapping counterpart".to_string(), None),
        ]);
        Ok(Status::just_one_fail(
            "missing-case-counterparts",
            &format!(
                "The following glyphs are missing case-swapping counterparts:\n{}",
                table.as_markdown().map_err(|_| CheckError::Error(
                    "Can't happen (table creation failed)".to_string()
                ))?
            ),
        ))
    }
}
