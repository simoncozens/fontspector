use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use markdown_table::MarkdownTable;
use read_fonts::{tables::gasp::GaspRangeBehavior, TableProvider};

const NON_HINTING_MESSAGE: &str =  "If you are dealing with an unhinted font, it can be fixed by running the fonts through the command 'gftools fix-nonhinting'\nGFTools is available at https://pypi.org/project/gftools/";

fn gasp_meaning(value: GaspRangeBehavior) -> String {
    let mut meaning = vec![];
    if value.intersects(GaspRangeBehavior::GASP_GRIDFIT) {
        meaning.push("- Use grid-fitting");
    }
    if value.intersects(GaspRangeBehavior::GASP_DOGRAY) {
        // 🗦🐶🗧
        meaning.push("- Use grayscale rendering");
    }
    if value.intersects(GaspRangeBehavior::GASP_SYMMETRIC_GRIDFIT) {
        meaning.push("- Use gridfitting with ClearType symmetric smoothing");
    }
    if value.intersects(GaspRangeBehavior::GASP_SYMMETRIC_SMOOTHING) {
        meaning.push("- Use smoothing along multiple axes with ClearType®");
    }
    meaning.join("\n\t")
}

#[check(
    id = "googlefonts/gasp",
    rationale = "
        
        Traditionally version 0 'gasp' tables were set so that font sizes below 8 ppem
        had no grid fitting but did have antialiasing. From 9-16 ppem, just grid
        fitting.
        And fonts above 17ppem had both antialiasing and grid fitting toggled on.
        The use of accelerated graphics cards and higher resolution screens make this
        approach obsolete. Microsoft's DirectWrite pushed this even further with much
        improved rendering built into the OS and apps.

        In this scenario it makes sense to simply toggle all 4 flags ON for all font
        sizes.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Is the Grid-fitting and Scan-conversion Procedure ('gasp') table
set to optimize rendering?"
)]
fn gasp(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    if !f.has_table(b"gasp") {
        return Ok(Status::just_one_fail(
            "lacks-gasp",
            &format!("Font is missing the 'gasp' table. Try exporting the font with autohinting enabled.\n{}"
                , NON_HINTING_MESSAGE)
        ));
    }
    let gasp_table = f.font().gasp()?;
    if gasp_table.gasp_ranges().is_empty() {
        return Ok(Status::just_one_fail(
            "empty",
            &format!("The 'gasp' table has no values.\n{}", NON_HINTING_MESSAGE),
        ));
    }
    if !gasp_table
        .gasp_ranges()
        .iter()
        .any(|r| r.range_max_ppem == 0xFFFF)
    {
        return Ok(Status::just_one_warn(
            "lacks-ffff-range",
            "The 'gasp' table does not have an entry that applies for all font sizes. The gaspRange value for such entry should be set to 0xFFFF.",
        ));
    }
    let md_table = MarkdownTable::new(
        gasp_table
            .gasp_ranges()
            .iter()
            .map(|r| {
                vec![
                    format!("PPM <= {}", r.range_max_ppem),
                    gasp_meaning(r.range_gasp_behavior.get()),
                ]
            })
            .collect(),
    );
    problems.push(Status::info(
        "ranges",
        &format!(
            "These are the ppm ranges declared on the gasp table:\n\n{}\n",
            md_table.to_string()
        ),
    ));
    for range in gasp_table.gasp_ranges() {
        if range.range_max_ppem != 0xFFFF {
            problems.push(Status::warn(
                "non-ffff-range",
                &format!(
                    "The gasp table has a range of {} that may be unneccessary.",
                    range.range_max_ppem
                ),
            ));
        } else if range.range_gasp_behavior.get().bits() != 0x0f {
            problems.push(Status::warn(
                "unset-flags",
                &format!(
                    "The gasp range 0xFFFF value 0x{:02X} should be set to 0x0F.",
                    range.range_gasp_behavior.get().bits()
                ),
            ));
        }
    }
    return_result(problems)
}
