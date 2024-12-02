use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
use humansize::{format_size, DECIMAL};

#[check(
    id = "file_size",
    rationale = "
        Serving extremely large font files causes usability issues.
        This check ensures that file sizes are reasonable.
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3320",
    title = "Ensure files are not too large."
)]
fn file_size(t: &Testable, context: &Context) -> CheckFnResult {
    let _ = testfont!(t); // Using this for the skip return
    let size = t.contents.len();
    let fail_size = context
        .configuration
        .get("FAIL_SIZE")
        .and_then(|v| v.as_u64());
    let warn_size = context
        .configuration
        .get("WARN_SIZE")
        .and_then(|v| v.as_u64());
    skip!(
        fail_size.is_none() && warn_size.is_none(),
        "no-size-limits",
        "No size limits configured"
    );

    if let Some(fail_size) = fail_size {
        if size as u64 > fail_size {
            return Ok(Status::just_one_fail(
                "massive-font",
                &format!(
                    "Font file is {}, larger than limit {}",
                    format_size(size, DECIMAL),
                    format_size(fail_size, DECIMAL),
                ),
            ));
        }
    }
    if let Some(warn_size) = warn_size {
        if size as u64 > warn_size {
            return Ok(Status::just_one_warn(
                "large-font",
                &format!(
                    "Font file is {}; ideally it should be less than {}",
                    format_size(size, DECIMAL),
                    format_size(warn_size, DECIMAL),
                ),
            ));
        }
    }

    Ok(Status::just_one_pass())
}
