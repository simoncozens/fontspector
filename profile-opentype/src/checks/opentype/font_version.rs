use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::{types::NameId, TableProvider};
use skrifa::MetadataProvider;

#[check(
    id = "opentype/font_version",
    title = "Checking font version fields (head and name table).",
    rationale = "
        The OpenType specification provides for two fields which contain
        the version number of the font: fontRevision in the head table,
        and nameID 5 in the name table. If these fields do not match,
        different applications will report different version numbers for
        the font.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",  // legacy check
)]
fn font_version(f: &Testable, _context: &Context) -> CheckFnResult {
    let font = testfont!(f);
    let head_version = font.font().head()?.font_revision().to_f32();
    let name_id_5_version = font
        .font()
        .localized_strings(NameId::VERSION_STRING)
        .english_or_first()
        .ok_or(CheckError::Error("No name ID 5".to_string()))?
        .chars()
        .skip_while(|c| !c.is_ascii_digit());
    let mut name_id_5_version_str = String::new();
    let mut periods = 0;
    for c in name_id_5_version {
        if c.is_ascii_digit() {
            name_id_5_version_str.push(c);
        } else if c == '.' {
            periods += 1;
            if periods > 1 {
                break;
            }
            name_id_5_version_str.push(c);
        }
    }
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
    if (head_version - name_id_5_version).abs() >= warn_tolerance {
        return Ok(Status::just_one_warn(
            "near-mismatch",
            &format!(
                "Font version mismatch: head table: {}, name table: {}",
                head_version, name_id_5_version
            ),
        ));
    }
    Ok(Status::just_one_pass())
}
