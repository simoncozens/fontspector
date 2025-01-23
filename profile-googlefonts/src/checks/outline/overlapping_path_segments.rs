use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use skrifa::MetadataProvider;

#[check(
    id = "overlapping_path_segments",
    rationale = "
        
        Some rasterizers encounter difficulties when rendering glyphs with
        overlapping path segments.

        A path segment is a section of a path defined by two on-curve points.
        When two segments share the same coordinates, they are considered
        overlapping.
    
    ",
    proposal = "https://github.com/google/fonts/issues/7594#issuecomment-2401909084",
    title = "Check there are no overlapping path segments"
)]
fn overlapping_path_segments(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    // def check_overlapping_path_segments(ttFont, outlines_dict, config):
    //     """Check there are no overlapping path segments"""
    //     failed = []
    //     for glyph, outlines in outlines_dict.items():
    //         seen = set()
    //         for p in outlines:
    //             for seg in p.asSegments():
    //                 normal = ((seg.start.x, seg.start.y), (seg.end.x, seg.end.y))
    //                 flipped = ((seg.end.x, seg.end.y), (seg.start.x, seg.start.y))
    //                 if normal in seen or flipped in seen:
    //                     failed.append(
    //                         f"{glyph[1]}: {seg} has the same coordinates as a previous segment."
    //                     )
    //                 seen.add(normal)
    //     if failed:
    //         yield WARN, Message(
    //             "overlapping-path-segments",
    //             f"The following glyphs have overlapping path segments:\n\n"
    //             f"{bullet_list(config, failed, bullet='*')}",
    //         )
    //     else:
    //         yield PASS, "No overlapping path segments found."
    //
    unimplemented!();
    return_result(problems)
}
