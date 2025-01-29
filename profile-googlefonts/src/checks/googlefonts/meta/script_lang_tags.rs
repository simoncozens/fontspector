use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use itertools::Itertools;
use read_fonts::{
    tables::meta::{DataMapRecord, Metadata::ScriptLangTags},
    FontData, TableProvider,
};

fn dump_data_map(
    d: &DataMapRecord,
    typ: &str,
    offset_data: FontData,
    problems: &mut Vec<Status>,
) -> Result<(), CheckError> {
    match d.data(offset_data)? {
        ScriptLangTags(var_len_array) => {
            let tags = var_len_array
                .iter()
                .flatten()
                .map(|x| x.as_str().to_string())
                .join(", ");
            problems.push(Status::info(
                &format!("{}-tag", typ),
                &format!("{:?}", tags),
            ));
        }
        _ => problems.push(Status::fail(
            &format!("invalid-{}-tag", typ),
            &format!(
                "The '{}' tag in the 'meta' table is not a ScriptLangTags data map.",
                typ
            ),
        )),
    };
    Ok(())
}
#[check(
    id = "googlefonts/meta/script_lang_tags",
    rationale = "
        
        The OpenType 'meta' table originated at Apple. Microsoft added it to OT with
        just two DataMap records:

        - dlng: comma-separated ScriptLangTags that indicate which scripts,
          or languages and scripts, with possible variants, the font is designed for.

        - slng: comma-separated ScriptLangTags that indicate which scripts,
          or languages and scripts, with possible variants, the font supports.


        The slng structure is intended to describe which languages and scripts the
        font overall supports. For example, a Traditional Chinese font that also
        contains Latin characters, can indicate Hant,Latn, showing that it supports
        Hant, the Traditional Chinese variant of the Hani script, and it also
        supports the Latn script.

        The dlng structure is far more interesting. A font may contain various glyphs,
        but only a particular subset of the glyphs may be truly \"leading\" in the design,
        while other glyphs may have been included for technical reasons. Such a
        Traditional Chinese font could only list Hant there, showing that it’s designed
        for Traditional Chinese, but the font would omit Latn, because the developers
        don’t think the font is really recommended for purely Latin-script use.

        The tags used in the structures can comprise just script, or also language
        and script. For example, if a font has Bulgarian Cyrillic alternates in the
        locl feature for the cyrl BGR OT languagesystem, it could also indicate in
        dlng explicitly that it supports bul-Cyrl. (Note that the scripts and languages
        in meta use the ISO language and script codes, not the OpenType ones).

        This check ensures that the font has the meta table containing the
        slng and dlng structures.

        All families in the Google Fonts collection should contain the 'meta' table.
        Windows 10 already uses it when deciding on which fonts to fall back to.
        The Google Fonts API and also other environments could use the data for
        smarter filtering. Most importantly, those entries should be added
        to the Noto fonts.

        In the font making process, some environments store this data in external
        files already. But the meta table provides a convenient way to store this
        inside the font file, so some tools may add the data, and unrelated tools
        may read this data. This makes the solution much more portable and universal.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3349",
    title = "Ensure fonts have ScriptLangTags declared on the 'meta' table."
)]
fn script_lang_tags(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    if let Ok(meta) = f.font().meta() {
        let offset_data = meta.offset_data();
        let data = meta.data_maps();
        if let Some(dlng) = data.iter().find(|d| d.tag.get() == "dlng") {
            dump_data_map(dlng, "dlng", offset_data, &mut problems)?;
        } else {
            problems.push(Status::fail(
                "missing-dlng-tag",
                "Please specify which languages and scripts this font is designed for.",
            ));
        }
        if let Some(slng) = data.iter().find(|d| d.tag.get() == "slng") {
            dump_data_map(slng, "slng", offset_data, &mut problems)?;
        } else {
            problems.push(Status::fail(
                "missing-slng-tag",
                "Please specify which languages and scripts this font supports.",
            ));
        }
    } else {
        problems.push(Status::warn(
            "lacks-meta-table",
            "This font file does not have a 'meta' table.",
        ));
    }
    return_result(problems)
}
