use super::{family_proto, protos::fonts_public::FontProto};
use fontspector_checkapi::{prelude::*, FileTypeConvert, TestFont};
use skrifa::string::StringId;

#[check(
    id = "googlefonts/metadata/consistent_with_fonts",
    title = "Check METADATA.pb parses correctly",
    rationale = "
        The purpose of this check is to ensure that the information in the METADATA.pb file
        is consistent with the font binaries in the font family.

        This subsumes the following fontbakery checks:

        - googlefonts/metadata/filenames
        - googlefonts/metadata/canonical_style_names
        - googlefonts/metadata/valid_full_name_values
        - googlefonts/metadata/nameid/post_script_name
        ",
    proposal = "https://github.com/fonttools/fontbakery/issues/2597 and https://github.com/fonttools/fontbakery/issues/4829",
    implementation = "all"
)]
fn consistent_with_fonts(c: &TestableCollection, _context: &Context) -> CheckFnResult {
    let mut problems = vec![];
    let mdpb = c
        .get_file("METADATA.pb")
        .ok_or_else(|| CheckError::skip("no-mdpb", "No METADATA.pb file found"))?;
    let msg = family_proto(mdpb)?;
    let mut declared_files = msg
        .fonts
        .iter()
        .flat_map(|f| f.filename.as_ref())
        .cloned()
        .collect::<Vec<String>>();
    let provided_files = c
        .iter()
        .flat_map(|t| t.filename.file_name())
        .map(|t| t.to_string_lossy().to_string())
        .filter(|t| t.ends_with(".otf") || t.ends_with(".ttf"))
        .collect::<Vec<String>>();
    // Match up fonts in msg with fonts in fonts - googlefonts/metadata/filenames
    {
        for declared_not_present in declared_files
            .iter()
            .filter(|f| !provided_files.contains(f))
        {
            problems.push(Status::fail(
            "file-not-found",
            &format!("Filename \"{}\" is listed on METADATA.pb but an actual font file with that name was not found.", declared_not_present),
        ));
        }
        for provided_not_declared in provided_files
            .iter()
            .filter(|f| !declared_files.contains(f))
        {
            problems.push(Status::fail(
                "file-not-declared",
                &format!(
                    "Filename \"{}\" is not declared on METADATA.pb as a font.filename entry.",
                    provided_not_declared
                ),
            ));
        }
        declared_files.retain(|f| provided_files.contains(f));
    }
    let md_font_pairs: Vec<(&FontProto, TestFont)> = msg
        .fonts
        .iter()
        .filter(|f| declared_files.contains(&f.filename().to_string()))
        .flat_map(|font_proto| {
            c.get_file(font_proto.filename())
                .map(|font| (font_proto, font))
        })
        .flat_map(|(font_proto, testable)| TTF.from_testable(testable).map(|ttf| (font_proto, ttf)))
        .collect();

    for (proto, font) in md_font_pairs.iter() {
        // canonical_style_names
        if proto.style() == "italic" || proto.style() == "normal" {
            if font.is_italic()? && proto.style() != "italic" {
                problems.push(Status::fail(
                    "italic",
                    &format!(
                        "The font style for {} is \"{}\" but it should be \"italic\".",
                        proto.filename(),
                        proto.style()
                    ),
                ));
            } else if !font.is_italic()? && proto.style() != "normal" {
                problems.push(Status::fail(
                    "normal",
                    &format!(
                        "The font style for {} is \"{}\" but it should be \"normal\".",
                        proto.filename(),
                        proto.style()
                    ),
                ));
            }
        }

        // googlefonts/metadata/valid_full_name_values
        let family_name = if font.style().is_some() {
            font.best_familyname()
        } else {
            font.get_best_name(&[StringId::TYPOGRAPHIC_FAMILY_NAME, StringId::FAMILY_NAME])
        }
        .ok_or_else(|| CheckError::Error(format!("No family name found for {}", proto.name())))?;
        if !proto.full_name().contains(&family_name) {
            problems.push(Status::fail(
                "mismatch",
                &format!(
                    "METADATA.pb font.full_name field \"{}\"  does not match correct font name format \"{}\".",
                    proto.full_name(),
                    family_name
                ),
            ));
        }
        // googlefonts/metadata/nameid/family_and_full_names
        for full_name in font.get_name_entry_strings(StringId::FULL_NAME) {
            if proto.full_name() != full_name {
                problems.push(Status::fail(
                    "fullname-mismatch",
                    &format!(
                    "METADATA.pb full_name field \"{}\" does not match correct full name \"{}\".",
                    proto.full_name(),
                    full_name
                ),
                ));
            }
        }
        if font.is_ribbi() {
            for family_name in font.get_name_entry_strings(StringId::FAMILY_NAME) {
                if proto.name() != family_name {
                    problems.push(Status::fail(
                    "familyname-mismatch",
                    &format!(
                    "METADATA.pb family name field \"{}\" does not match correct family name \"{}\".",
                    proto.name(),
                    family_name
                ),
                ));
                }
            }
        }

        // googlefonts/metadata/nameid/post_script_name (make sure postscript name is consistent)
        let post_script_name = font
            .get_best_name(&[StringId::POSTSCRIPT_NAME])
            .ok_or_else(|| {
                CheckError::Error(format!("No post script name found for {}", proto.name()))
            })?;
        if proto.post_script_name() != post_script_name {
            problems.push(Status::fail(
                "mismatch",
                &format!(
                    "METADATA.pb post_script_name field \"{}\" does not match correct post script name \"{}\".",
                    proto.post_script_name(),
                    post_script_name
                ),
            ));
        }
        // googlefonts/metadata/valid_post_script_name_values (make sure postscript name is correct)
        let familyname = font
            .best_familyname()
            .ok_or_else(|| CheckError::Error(format!("No family name found for {}", proto.name())))?
            .replace(" ", "");
        if !proto
            .post_script_name()
            .replace("-", "")
            .contains(&familyname)
        {
            problems.push(Status::fail(
                "mismatch",
                &format!(
                    "METADATA.pb post_script_name field \"{}\" does not match correct font name \"{}\".",
                    proto.post_script_name(),
                    familyname
                ),
            ));
        }
    }
    return_result(problems)
}
