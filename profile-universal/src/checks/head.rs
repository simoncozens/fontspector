use font_types::NameId;
use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::{tables::head::MacStyle, TableProvider};
use skrifa::MetadataProvider;

#[check(
    id = "font_version",
    proposal = "legacy:check/044",
    title = "Checking font version fields (head and name table).",
    rationale = "
    The OpenType specification provides for two fields which contain
    the version number of the font: fontRevision in the head table,
    and nameID 5 in the name table. If these fields do not match,
    different applications will report different version numbers for
    the font.
    "
)]
fn font_version(f: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let head_version = font.font().head()?.font_revision().to_f32();
    let name_id_5_version_str = font
        .font()
        .localized_strings(NameId::VERSION_STRING)
        .english_or_first()
        .ok_or(CheckError::Error("No name ID 5".to_string()))?
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect::<String>();
    if name_id_5_version_str.is_empty() {
        return Err(CheckError::Error(
            "No version string in name table".to_string(),
        ));
    }
    let name_id_5_version = name_id_5_version_str.parse::<f32>().map_err(|e| {
        CheckError::Error(format!("Could not parse name ID 5 version as float: {}", e))
    })?;
    let warn_tolerance = 1.0 / (0x10000 as f32);
    let fail_tolerance = 1.0 / 2000.0;
    if (head_version - name_id_5_version).abs() > fail_tolerance {
        return Ok(Status::just_one_fail(
            "mismatch",
            &format!(
                "Font version mismatch: head table: {}, name table: {}",
                head_version, name_id_5_version
            ),
        ));
    }
    if (head_version - name_id_5_version).abs() > warn_tolerance {
        return Ok(Status::just_one_warn(
            "mismatch",
            &format!(
                "Font version mismatch: head table: {}, name table: {}",
                head_version, name_id_5_version
            ),
        ));
    }
    Ok(Status::just_one_pass())
}

#[check(
    id = "opentype/mac_style",
    proposal = "legacy:check/031",
    title = "Checking head.macStyle value.",
    rationale = "
    The values of the flags on the macStyle entry on the 'head' OpenType table
    that describe whether a font is bold and/or italic must be coherent with the
    actual style of the font as inferred by its filename.
    "
)]
fn mac_style(f: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let head = font.font().head()?;
    let style = font
        .style()
        .ok_or(CheckError::skip("no-style", "No style detected"))?;
    let bold = style == "Bold" || style == "BoldItalic";
    let italic = style.contains("Italic");
    let bits = head.mac_style();
    let bold_ok = bits.contains(MacStyle::BOLD) == bold;
    let italic_ok = bits.contains(MacStyle::ITALIC) == italic;
    let mut problems = vec![];
    if !bold_ok {
        problems.push(Status::warn(
            "bold-mismatch",
            &format!(
                "macStyle bold flag {} does not match font style {}",
                bits.contains(MacStyle::BOLD),
                style
            ),
        ));
    }
    if !italic_ok {
        problems.push(Status::warn(
            "italic-mismatch",
            &format!(
                "macStyle italic flag {} does not match font style {}",
                bits.contains(MacStyle::ITALIC),
                italic
            ),
        ));
    }
    return_result(problems)
}
