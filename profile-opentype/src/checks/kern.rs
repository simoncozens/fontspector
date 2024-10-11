// Fontations doesn't yet support kern table

// use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert};
// use read_fonts::TableProvider;

// #[check(
//     id = "opentype/kern_table",
//     rationale = "
//         Even though all fonts should have their kerning implemented in the GPOS table,
//         there may be kerning info at the kern table as well.

//         Some applications such as MS PowerPoint require kerning info on the kern table.
//         More specifically, they require a format 0 kern subtable from a kern table
//         version 0 with only glyphs defined in the cmap table, which is the only one
//         that Windows understands (and which is also the simplest and more limited
//         of all the kern subtables).

//         Google Fonts ingests fonts made for download and use on desktops, and does
//         all web font optimizations in the serving pipeline (using libre libraries
//         that anyone can replicate.)

//         Ideally, TTFs intended for desktop users (and thus the ones intended for
//         Google Fonts) should have both KERN and GPOS tables.

//         Given all of the above, we currently treat kerning on a v0 kern table
//         as a good-to-have (but optional) feature.
//     ",
//     proposal = "https://github.com/fonttools/fontbakery/issues/1675",
//     title = "Is there a usable 'kern' table declared in the font?"
// )]
// fn kern_table(f: &Testable, _context: &Context) -> CheckFnResult {
//     let font = testfont!(f);
//     if !font.has_table(b"kern") {
//         return Ok(Status::just_one_pass());
//     }
//     let kern = font.font().kern().ok();
//     Ok(Status::just_one_pass())
// }
