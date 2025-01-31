use super::{
    schema::{ShapingConfig, ShapingTest},
    ShapingCheck,
};
use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use hashbrown::HashSet;
use itertools::Itertools;
use rustybuzz::{Face, GlyphBuffer};

#[check(
    id = "shaping/forbidden",
    rationale = "
        
        Fonts with complex layout rules can benefit from regression tests to ensure
        that the rules are behaving as designed. This checks runs a shaping test
        suite and reports if any glyphs are generated in the shaping which should
        not be produced. (For example, .notdef glyphs, visible viramas, etc.)

        Shaping test suites should be written by the font engineer and referenced
        in the FontBakery configuration file. For more information about write
        shaping test files and how to configure FontBakery to read the shaping
        test suites, see https://simoncozens.github.io/tdd-for-otl/
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/3223",
    title = "Check that no forbidden glyphs are found while shaping"
)]
fn forbidden(t: &Testable, context: &Context) -> CheckFnResult {
    let _f = testfont!(t); // We just use this to make sure it's a font
    let mut problems = vec![];
    for (filename, fails) in ForbiddenTest.run(t, context)? {
        let mut report = String::new();
        for fail in fails {
            report.push_str(&format!(
                "{} produced forbidden {}\n",
                fail.test.input, fail.detail
            ));
        }
        problems.push(Status::fail(
            "shaping-regression",
            &format!(
                "{}: Forbidden glyphs found while shaping:\n\n{}",
                filename, report
            ),
        ))
        // Add a diff table
        // draw as svg
    }
    return_result(problems)
}

fn serialize(buffer: &GlyphBuffer, face: &Face) -> String {
    let flags = rustybuzz::SerializeFlags::NO_POSITIONS
        | rustybuzz::SerializeFlags::NO_ADVANCES
        | rustybuzz::SerializeFlags::NO_CLUSTERS;
    buffer.serialize(face, flags)
}

struct ForbiddenTest;

impl ShapingCheck for ForbiddenTest {
    fn pass_fail(
        &self,
        _test: &ShapingTest,
        configuration: &ShapingConfig,
        buffer: &GlyphBuffer,
        face: &Face,
    ) -> Option<String> {
        let serialized = serialize(buffer, face);
        let glyphs: HashSet<&str> = serialized.split('|').collect();
        let forbidden_glyphs: HashSet<&str> = configuration
            .forbidden_glyphs
            .iter()
            .map(|s| s.as_str())
            .collect();
        let found = glyphs.intersection(&forbidden_glyphs).collect_vec();
        if found.is_empty() {
            return None;
        }
        Some(found.into_iter().join(", "))
    }

    fn applies(&self, configuration: &ShapingConfig, _test: &ShapingTest) -> bool {
        !configuration.forbidden_glyphs.is_empty()
    }
}
