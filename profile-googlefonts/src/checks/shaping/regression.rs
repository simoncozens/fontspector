use super::{schema::ShapingTest, ShapingCheck};
use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use itertools::Itertools;
use rustybuzz::{Face, GlyphBuffer};

#[check(
    id = "shaping/regression",
    rationale = "
        
        Fonts with complex layout rules can benefit from regression tests to ensure
        that the rules are behaving as designed. This checks runs a shaping test
        suite and compares expected shaping against actual shaping, reporting
        any differences.

        Shaping test suites should be written by the font engineer and referenced
        in the FontBakery configuration file. For more information about write
        shaping test files and how to configure FontBakery to read the shaping
        test suites, see https://simoncozens.github.io/tdd-for-otl/
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/3223",
    title = "Check that texts shape as per expectation"
)]
fn regression(t: &Testable, context: &Context) -> CheckFnResult {
    let _f = testfont!(t); // We just use this to make sure it's a font
    let message = "Shaping did not match";
    let mut problems = vec![];
    for (filename, fails) in RegressionTest.run(t, context)? {
        let mut report = String::new();
        for fail in fails {
            report.push_str(&format!(
                "{}: {}{}\n{}\n\n",
                message,
                fail.test.input,
                fail.test.note(),
                fail.detail
            ));
        }
        problems.push(Status::fail(
            "shaping-regression",
            &format!(
                "{}: Expected and actual shaping not matching\n\n{}",
                filename, report
            ),
        ))
        // Add a diff table
        // draw as svg
    }
    return_result(problems)
}

fn serialize_appropriately(buffer: &GlyphBuffer, face: &Face, test: &ShapingTest) -> String {
    let mut flags = rustybuzz::SerializeFlags::default();
    #[allow(clippy::unwrap_used)] // the .applies filter ensures there's an expectation
    if !test.expectation.as_ref().unwrap().contains("=") {
        flags |= rustybuzz::SerializeFlags::NO_POSITIONS
            | rustybuzz::SerializeFlags::NO_ADVANCES
            | rustybuzz::SerializeFlags::NO_CLUSTERS;
    }
    buffer.serialize(face, flags)
}

struct RegressionTest;

impl ShapingCheck for RegressionTest {
    fn pass_fail(&self, test: &ShapingTest, buffer: &GlyphBuffer, face: &Face) -> Option<String> {
        let serialized = serialize_appropriately(buffer, face, test);
        #[allow(clippy::unwrap_used)] // the .applies filter ensures there's an expectation
        let expected = test.expectation.as_ref().unwrap();
        if &serialized == expected {
            return None;
        }
        let diff = similar::TextDiff::from_chars(expected, &serialized)
            .iter_all_changes()
            .map(|d| match d.tag() {
                similar::ChangeTag::Equal => " ",
                similar::ChangeTag::Delete => "-",
                similar::ChangeTag::Insert => "",
            })
            .join("");
        let report = format!(
            "Expected: {}\nGot     : {}\nDiff    : {}\n",
            expected, serialized, diff
        );
        Some(report)
    }

    fn applies(&self, test: &ShapingTest) -> bool {
        test.expectation.is_some()
    }
}
