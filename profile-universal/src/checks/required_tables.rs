use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};

const OPTIONAL_TABLE_TAGS: [&[u8; 4]; 20] = [
    b"cvt ", b"fpgm", b"loca", b"prep", b"VORG", b"EBDT", b"EBLC", b"EBSC", b"BASE", b"GPOS",
    b"GSUB", b"JSTF", b"gasp", b"hdmx", b"LTSH", b"PCLT", b"VDMX", b"vhea", b"vmtx", b"kern",
];

#[check(
    id = "required_tables",
    title = "Font contains all required tables?",
    rationale = "
        According to the OpenType spec
        https://docs.microsoft.com/en-us/typography/opentype/spec/otff#required-tables

        Whether TrueType or CFF outlines are used in an OpenType font, the following
        tables are required for the font to function correctly:

        - cmap (Character to glyph mapping)⏎
        - head (Font header)⏎
        - hhea (Horizontal header)⏎
        - hmtx (Horizontal metrics)⏎
        - maxp (Maximum profile)⏎
        - name (Naming table)⏎
        - OS/2 (OS/2 and Windows specific metrics)⏎
        - post (PostScript information)

        The spec also documents that variable fonts require the following table:

        - STAT (Style attributes)

        Depending on the typeface and coverage of a font, certain tables are
        recommended for optimum quality.

        For example:⏎
        - the performance of a non-linear font is improved if the VDMX, LTSH,
          and hdmx tables are present.⏎
        - Non-monospaced Latin fonts should have a kern table.⏎
        - A gasp table is necessary if a designer wants to influence the sizes
          at which grayscaling is used under Windows. Etc.
    ",
    proposal = "legacy:check/052"
)]
fn required_tables(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut required_table_tags: Vec<_> = vec![
        b"cmap", b"head", b"hhea", b"hmtx", b"maxp", b"name", b"OS/2", b"post",
    ];

    if f.is_variable_font() {
        // According to https://github.com/fonttools/fontbakery/issues/1671
        // STAT table is required on WebKit on MacOS 10.12 for variable fonts.
        required_table_tags.push(b"STAT");
    }

    // See https://github.com/fonttools/fontbakery/issues/617
    //
    // We should collect the rationale behind the need for each of the
    // required tables above. Perhaps split it into individual checks
    // with the correspondent rationales for each subset of required tables.
    //
    // opentype/kern_table is a good example of a separate
    // check for a specific table providing a detailed description of
    // the rationale behind it.

    let mut problems: Vec<Status> = vec![];
    let mut optional: Vec<String> = vec![];

    for tag in OPTIONAL_TABLE_TAGS {
        if f.has_table(tag) {
            optional.push(String::from_utf8(tag.to_vec()).map_err(|_| {
                CheckError::Error(format!("Font tag '{:?}' wasn't UTF8?", tag.to_vec()))
            })?)
        }
    }
    if !optional.is_empty() {
        problems.push(Status::info(
            "optional-tables",
            &format!(
                "This font contains the following optional tables:\n\n    {}",
                optional.join("\n    ")
            ),
        ))
    }

    let mut missing = vec![];
    for tag in required_table_tags {
        if !f.has_table(tag) {
            missing.push(String::from_utf8(tag.to_vec()).map_err(|_| {
                CheckError::Error(format!("Font tag '{:?}' wasn't UTF8?", tag.to_vec()))
            })?);
        }
    }

    // Note (from the OpenType spec):
    // OpenType fonts that contain TrueType outlines should use the value of 0x00010000
    // for sfntVersion. OpenType fonts containing CFF data (version 1 or 2) should use
    // 0x4F54544F ('OTTO', when re-interpreted as a Tag) for sfntVersion.
    let version = f.font().table_directory.sfnt_version();
    if version == 0x4F54544F && (!f.has_table(b"CFF ") && !f.has_table(b"CFF2")) {
        if f.has_table(b"fvar") {
            missing.push("CFF2".to_string());
        } else {
            missing.push("CFF ".to_string());
        }
    } else if version == 0x00010000 && !f.has_table(b"glyf") {
        missing.push("glyf".to_string());
    }

    if !missing.is_empty() {
        problems.push(Status::fail(
            "required-tables",
            &format!(
                "This font is missing the following required tables:\n\n    {}",
                missing.join("\n    ")
            ),
        ))
    }

    return_result(problems)
}
