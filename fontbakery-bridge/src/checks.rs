use crate::run_a_python_test;
use fontspector_checkapi::prelude::*;

pub(crate) const hinting_impact: Check = Check {
    id: "hinting_impact",
    title: "Show hinting filesize impact",
    rationale: r#"""
        This check is merely informative, displaying an useful comparison of filesizes
        of hinted versus unhinted font files.
    """#,
    proposal: "https://github.com/fonttools/fontbakery/issues/4829",
    hotfix: None,
    fix_source: None,
    applies_to: "TTF",
    flags: CheckFlags::default(),
    implementation: CheckImplementation::CheckOne(&run_a_python_test),
    _metadata: Some(
        r#" { "module": "fontbakery.checks.hinting", "function": "check_hinting_impact" } "#,
    ),
};

pub(crate) const opentype_name_empty_records: Check = Check {
    id: "opentype/name/empty_records",
    title: "Check name table for empty records",
    rationale: "Check the name table for empty records, as this can cause problems in Adobe apps.",
    proposal: "https://github.com/fonttools/fontbakery/pull/2369",
    hotfix: None,
    fix_source: None,
    applies_to: "TTF",
    flags: CheckFlags::default(),
    implementation: CheckImplementation::CheckOne(&run_a_python_test),
    _metadata: Some(
        r#" { "module": "fontbakery.checks.opentype.name", "function": "check_name_empty_records" } "#,
    ),
};

pub(crate) const monospace: Check = Check {
    id: "opentype/monospace",
    title: "Check name table for empty records",
    rationale: r#"""
        There are various metadata in the OpenType spec to specify if a font is
        monospaced or not. If the font is not truly monospaced, then no monospaced
        metadata should be set (as sometimes they mistakenly are...)

        Requirements for monospace fonts:

        * post.isFixedPitch - "Set to 0 if the font is proportionally spaced,
          non-zero if the font is not proportionally spaced (monospaced)"
          (https://www.microsoft.com/typography/otspec/post.htm)

        * hhea.advanceWidthMax must be correct, meaning no glyph's width value
          is greater. (https://www.microsoft.com/typography/otspec/hhea.htm)

        * OS/2.panose.bProportion must be set to 9 (monospace) on latin text fonts.

        * OS/2.panose.bSpacing must be set to 3 (monospace) on latin hand written
          or latin symbol fonts.

        * Spec says: "The PANOSE definition contains ten digits each of which currently
          describes up to sixteen variations. Windows uses bFamilyType, bSerifStyle
          and bProportion in the font mapper to determine family type. It also uses
          bProportion to determine if the font is monospaced."
          (https://www.microsoft.com/typography/otspec/os2.htm#pan
           https://monotypecom-test.monotype.de/services/pan2)

        * OS/2.xAvgCharWidth must be set accurately.
          "OS/2.xAvgCharWidth is used when rendering monospaced fonts,
          at least by Windows GDI"
          (http://typedrawers.com/discussion/comment/15397/#Comment_15397)

        Also we should report an error for glyphs not of average width.


        Please also note:

        Thomas Phinney told us that a few years ago (as of December 2019), if you gave
        a font a monospace flag in Panose, Microsoft Word would ignore the actual
        advance widths and treat it as monospaced.

        Source: https://typedrawers.com/discussion/comment/45140/#Comment_45140
"""#,
    proposal: "https://github.com/fonttools/fontbakery/issues/4829",
    hotfix: None,
    fix_source: None,
    applies_to: "TTF",
    flags: CheckFlags::default(),
    implementation: CheckImplementation::CheckOne(&run_a_python_test),
    _metadata: Some(
        r#" { "module": "fontbakery.checks.opentype.name", "function": "check_monospace" } "#,
    ),
};
