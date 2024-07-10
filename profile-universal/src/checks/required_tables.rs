use fontspector_checkapi::{return_result, Check, Status, StatusList, TestFont};

use skrifa::Tag;

fn required_tables(f: &TestFont) -> StatusList {
    let mut required_table_tags: Vec<Tag> = vec![
        Tag::new(b"cmap"),
        Tag::new(b"head"),
        Tag::new(b"hhea"),
        Tag::new(b"hmtx"),
        Tag::new(b"maxp"),
        Tag::new(b"name"),
        Tag::new(b"OS/2"),
        Tag::new(b"post"),
    ];

    if f.is_variable_font() {
        // According to https://github.com/fonttools/fontbakery/issues/1671
        // STAT table is required on WebKit on MacOS 10.12 for variable fonts.
        required_table_tags.push(Tag::new(b"STAT"));
    }

    const OPTIONAL_TABLE_TAGS: [Tag; 20] = [
        Tag::new(b"cvt "),
        Tag::new(b"fpgm"),
        Tag::new(b"loca"),
        Tag::new(b"prep"),
        Tag::new(b"VORG"),
        Tag::new(b"EBDT"),
        Tag::new(b"EBLC"),
        Tag::new(b"EBSC"),
        Tag::new(b"BASE"),
        Tag::new(b"GPOS"),
        Tag::new(b"GSUB"),
        Tag::new(b"JSTF"),
        Tag::new(b"gasp"),
        Tag::new(b"hdmx"),
        Tag::new(b"LTSH"),
        Tag::new(b"PCLT"),
        Tag::new(b"VDMX"),
        Tag::new(b"vhea"),
        Tag::new(b"vmtx"),
        Tag::new(b"kern"),
    ];

    // See https://github.com/fonttools/fontbakery/issues/617
    //
    // We should collect the rationale behind the need for each of the
    // required tables above. Perhaps split it into individual checks
    // with the correspondent rationales for each subset of required tables.
    //
    // com.google.fonts/check/kern_table is a good example of a separate
    // check for a specific table providing a detailed description of
    // the rationale behind it.

    let mut problems: Vec<Status> = vec![];

    let mut optional = vec![];
    for tag in OPTIONAL_TABLE_TAGS.iter() {
        if f.font().table_data(*tag).is_some() {
            optional.push(format!("{}", *tag));
        }
    }
    if !optional.is_empty() {
        problems.push(Status::info(&format!(
            "This font contains the following optional tables:\n\n{}",
            optional.join("\n")
        )))
    }

    let mut missing = vec![];
    for tag in required_table_tags.iter() {
        if !f.font().table_data(*tag).is_some() {
            missing.push(format!("{}", *tag));
        }
    }

    // Note (from the OpenType spec):
    // OpenType fonts that contain TrueType outlines should use the value of 0x00010000
    // for sfntVersion. OpenType fonts containing CFF data (version 1 or 2) should use
    // 0x4F54544F ('OTTO', when re-interpreted as a Tag) for sfntVersion.
    if f.font().table_directory.sfnt_version() == 0x4F54544F
        && (!f.font().table_data(Tag::new(b"CFF ")).is_some()
            && !f.font().table_data(Tag::new(b"CFF2")).is_some())
    {
        if f.font().table_data(Tag::new(b"fvar")).is_some() {
            missing.push("CFF2".to_string());
        } else {
            missing.push("CFF ".to_string());
        }
    } else {
        if f.font().table_directory.sfnt_version() == 0x00010000
            && !f.font().table_data(Tag::new(b"glyf")).is_some()
        {
            missing.push("glyf".to_string());
        }
    }

    if !missing.is_empty() {
        problems.push(Status::fail(&format!(
            "This font is missing the following required tables:\n\n{}",
            missing.join("\n")
        )))
    }

    if problems.is_empty() {
        Status::just_one_pass()
    } else {
        return_result(problems)
    }
}

pub const REQUIRED_TABLES_CHECK: Check = Check {
    id: "com.google.fonts/check/required_tables",
    title: "Font contains all required tables?",
    rationale: Some(
        "
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
    ),
    proposal: Some("legacy:check/052"),
    check_one: Some(&required_tables),
    check_all: None,
};
