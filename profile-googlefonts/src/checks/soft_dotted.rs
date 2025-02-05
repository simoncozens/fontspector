use fontspector_checkapi::{
    pens::BezGlyph, prelude::*, skip, testfont, FileTypeConvert, DEFAULT_LOCATION,
};
use rustybuzz::{Face, UnicodeBuffer};
use skrifa::MetadataProvider;
use unicode_canonical_combining_class::{get_canonical_combining_class, CanonicalCombiningClass};
use yeslogic_unicode_blocks::{
    COMBINING_DIACRITICAL_MARKS, COMBINING_DIACRITICAL_MARKS_EXTENDED,
    COMBINING_DIACRITICAL_MARKS_FOR_SYMBOLS, COMBINING_DIACRITICAL_MARKS_SUPPLEMENT, CYRILLIC,
    CYRILLIC_EXTENDED_A, CYRILLIC_EXTENDED_B, CYRILLIC_EXTENDED_C, CYRILLIC_EXTENDED_D,
    CYRILLIC_SUPPLEMENT,
};

const ORTHO_SOFT_DOTTED_STRINGS: [&str; 48] = [
    "i̋", "i̍", "i᷆", "i᷇", "i̓", "i̊", "i̐", "ɨ́", "ɨ̀", "ɨ̂", "ɨ̋", "ɨ̏", "ɨ̌", "ɨ̄", "ɨ̃", "ɨ̈", "ɨ̧́", "ɨ̧̀", "ɨ̧̂",
    "ɨ̧̌", "ɨ̱́", "ɨ̱̀", "ɨ̱̈", "į́", "į̀", "į̂", "į̄", "į̄́", "į̄̀", "į̄̂", "į̄̌", "į̃", "į̌", "ị́", "ị̀", "ị̂", "ị̄", "ị̃",
    "ḭ́", "ḭ̀", "ḭ̄", "j́", "j̀", "j̄", "j̑", "j̃", "j̈", "і́",
];
const SOFT_DOTTED_CHARS: [char; 47] = [
    'i', 'ⅈ', '𝐢', '𝑖', '𝒊', '𝒾', '𝓲', '𝔦', '𝕚', '𝖎', '𝗂', '𝗶', '𝘪', '𝙞', '𝚒', 'ⁱ', 'ᵢ', 'į', 'ị',
    'ḭ', 'ɨ', 'ᶤ', '𝼚', 'ᶖ', 'j', 'ⅉ', '𝐣', '𝑗', '𝒋', '𝒿', '𝓳', '𝔧', '𝕛', '𝖏', '𝗃', '𝗷', '𝘫', '𝙟',
    '𝚓', 'ʲ', 'ⱼ', 'ɉ', 'ʝ', 'ᶨ', 'ϳ', 'і', 'ј',
];

#[check(
    id = "soft_dotted",
    rationale = "
        
        An accent placed on characters with a \"soft dot\", like i or j, causes
        the dot to disappear.
        An explicit dot above can be added where required.
        See \"Diacritics on i and j\" in Section 7.1, \"Latin\" in The Unicode Standard.

        Characters with the Soft_Dotted property are listed in
        https://www.unicode.org/Public/UCD/latest/ucd/PropList.txt

        See also:
        https://googlefonts.github.io/gf-guide/diacritics.html#soft-dotted-glyphs
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4059",
    title = "Ensure soft_dotted characters lose their dot when combined with marks that
replace the dot."
)]
fn soft_dotted(t: &Testable, context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let codepoints = f
        .codepoints(Some(context))
        .iter()
        .flat_map(|c| char::from_u32(*c))
        .collect::<Vec<_>>();
    let soft_dotted_chars = codepoints
        .iter()
        .filter(|c| SOFT_DOTTED_CHARS.contains(c))
        .copied()
        .collect::<Vec<_>>();
    let mark_above_chars = codepoints
        .iter()
        .filter(|c| {
            let block = yeslogic_unicode_blocks::find_unicode_block(**c);
            let combining = get_canonical_combining_class(**c);
            combining == CanonicalCombiningClass::Above
                && (block == Some(COMBINING_DIACRITICAL_MARKS)
                    || block == Some(COMBINING_DIACRITICAL_MARKS_EXTENDED)
                    || block == Some(COMBINING_DIACRITICAL_MARKS_FOR_SYMBOLS)
                    || block == Some(COMBINING_DIACRITICAL_MARKS_SUPPLEMENT)
                    || block == Some(CYRILLIC)
                    || block == Some(CYRILLIC_EXTENDED_A)
                    || block == Some(CYRILLIC_EXTENDED_B)
                    || block == Some(CYRILLIC_EXTENDED_C)
                    || block == Some(CYRILLIC_EXTENDED_D)
                    || block == Some(CYRILLIC_SUPPLEMENT))
        })
        .copied()
        .collect::<Vec<_>>();

    let mark_non_above_chars = codepoints
        .iter()
        .filter(|c| {
            let block = yeslogic_unicode_blocks::find_unicode_block(**c);
            let combining = get_canonical_combining_class(**c);
            combining < CanonicalCombiningClass::Above
                && (block == Some(COMBINING_DIACRITICAL_MARKS)
                    || block == Some(COMBINING_DIACRITICAL_MARKS_EXTENDED)
                    || block == Some(COMBINING_DIACRITICAL_MARKS_FOR_SYMBOLS)
                    || block == Some(COMBINING_DIACRITICAL_MARKS_SUPPLEMENT))
        })
        .copied()
        .collect::<Vec<_>>();

    if soft_dotted_chars.is_empty() || mark_above_chars.is_empty() {
        skip!(
            "no-soft-dotted",
            "Font has no soft dotted characters or no mark above characters."
        );
    }

    let mut i_paths = BezGlyph::default();
    if let Some(i_gid) = f.font().charmap().map('i') {
        f.draw_glyph(i_gid, &mut i_paths, DEFAULT_LOCATION)?;
    }
    let mut capi_paths = BezGlyph::default();
    if let Some(capi_gid) = f.font().charmap().map('I') {
        f.draw_glyph(capi_gid, &mut capi_paths, DEFAULT_LOCATION)?;
    }
    let mut dotlessi_paths = BezGlyph::default();
    if let Some(dotlessi_gid) = f.font().charmap().map('ı') {
        f.draw_glyph(dotlessi_gid, &mut dotlessi_paths, DEFAULT_LOCATION)?;
    }

    if i_paths.0.len() == capi_paths.0.len() || i_paths.0.len() == dotlessi_paths.0.len() {
        skip!(
            "unclear",
            "It is not clear if the soft dotted characters have glyphs with dots."
        );
    }

    let face = Face::from_slice(&t.contents, 0)
        .ok_or(CheckError::Error("Failed to load font file".to_string()))?;

    let mut fail_unchanged_strings = vec![];
    let mut warn_unchanged_strings = vec![];
    for soft in soft_dotted_chars.iter().copied() {
        for non_above in mark_non_above_chars
            .iter()
            .copied()
            .chain(std::iter::once('\0'))
        {
            for above in mark_above_chars.iter().copied() {
                let (text, unchanged) = if non_above != '\0' {
                    (
                        format!("{}{}{}", soft, non_above, above),
                        format!(
                            "{}|{}|{}",
                            f.font().charmap().map(soft).unwrap_or_default().to_u32(),
                            f.font()
                                .charmap()
                                .map(non_above)
                                .unwrap_or_default()
                                .to_u32(),
                            f.font().charmap().map(above).unwrap_or_default().to_u32()
                        ),
                    )
                } else {
                    (
                        format!("{}{}", soft, above),
                        format!(
                            "{}|{}",
                            f.font().charmap().map(soft).unwrap_or_default().to_u32(),
                            f.font().charmap().map(above).unwrap_or_default().to_u32()
                        ),
                    )
                };
                let mut buffer = UnicodeBuffer::new();
                buffer.push_str(&text);
                let buffer = rustybuzz::shape(&face, &[], buffer);
                let flags = rustybuzz::SerializeFlags::NO_POSITIONS
                    | rustybuzz::SerializeFlags::NO_ADVANCES
                    | rustybuzz::SerializeFlags::NO_CLUSTERS
                    | rustybuzz::SerializeFlags::NO_GLYPH_NAMES;
                let output = buffer.serialize(&face, flags);
                if output == unchanged {
                    if ORTHO_SOFT_DOTTED_STRINGS.contains(&text.as_str()) {
                        fail_unchanged_strings.push(text);
                    } else {
                        warn_unchanged_strings.push(text);
                    }
                }
            }
        }
    }
    let mut message = "".to_string();
    if !fail_unchanged_strings.is_empty() {
        message.push_str(&format!(
            "The dot of soft dotted characters used in orthographies _must_ disappear in the following strings: {}",
            bullet_list(context, fail_unchanged_strings)
        ));
    }
    if !warn_unchanged_strings.is_empty() {
        message.push_str(&format!(
            "The dot of soft dotted characters _should_ disappear in other cases, for example: {}",
            bullet_list(context, warn_unchanged_strings)
        ));
    }
    if message.is_empty() {
        return Ok(Status::just_one_pass());
    } else {
        return Ok(Status::just_one_warn("soft-dotted", &message));
    }
}
