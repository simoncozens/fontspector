use fontspector_checkapi::{prelude::*, skip};

struct Designspace;

fn direction(contour: &norad::Contour) -> bool {
    let mut total = 0.0;
    for (i, point) in contour.points.iter().enumerate() {
        let next = contour.points.get((i + 1) % contour.points.len()).unwrap();
        total += (next.x - point.x) * (next.y + point.y);
    }
    total > 0.0
}

#[check(
    id = "designspace/path_direction",
    rationale = "Make sure the paths have the same direction across all masters.",
    proposal = "chat",
    title = "Check path direction.",
    applies_to = "DESIGNSPACE"
)]
fn path_direction(t: &Testable, _context: &Context) -> CheckFnResult {
    let ds_contents = std::str::from_utf8(&t.contents)
        .map_err(|_| CheckError::Error("designspace is not valid UTF-8".to_string()))?;
    let ds = quick_xml::de::from_str::<norad::designspace::DesignSpaceDocument>(ds_contents)?;
    #[allow(clippy::unwrap_used)] // Life is short
    let dirname = t.filename.parent().unwrap();
    let master_names = ds.sources.iter().map(|s| &s.filename).collect::<Vec<_>>();
    let ufos = ds
        .sources
        .iter()
        .flat_map(|s| {
            let source_full_path = dirname.join(&s.filename);
            norad::Font::load(&source_full_path)
        })
        .collect::<Vec<_>>();
    if ufos.is_empty() {
        return Ok(Status::just_one_fail(
            "no-sources",
            "Couldn't load any sources",
        ));
    }
    skip!(
        ufos.len() < 2,
        "not-enough-sources",
        "Not enough sources to compare"
    );
    let glyphnames = ufos.first().unwrap().iter_names().collect::<Vec<_>>();
    let mut problems = vec![];
    for glyph in glyphnames {
        // println!("Checking glyph {}", glyph);
        let all_glyphs = ufos
            .iter()
            .flat_map(|s| s.get_glyph(&glyph))
            .collect::<Vec<_>>();
        let first = all_glyphs.first().unwrap();
        let others = all_glyphs.iter().skip(1);
        for (other_ix, other) in others.enumerate() {
            for (contour_ix, (first_contour, other_contour)) in
                first.contours.iter().zip(other.contours.iter()).enumerate()
            {
                if direction(first_contour) != direction(other_contour) {
                    problems.push((glyph.to_string(), other_ix, contour_ix));
                }
            }
        }
    }
    return_result(
        problems
            .iter()
            .map(|(glyph, other_ix, contour_ix)| {
                Status::fail(
                    "path-direction",
                    &format!(
                        "Glyph {} has different path direction in master {} contour {}",
                        glyph,
                        master_names[1 + other_ix],
                        contour_ix
                    ),
                )
            })
            .collect::<Vec<_>>(),
    )
}

impl fontspector_checkapi::Plugin for Designspace {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        let designspace = FileType::new("*.designspace");
        let ufo = FileType::new("*.ufo");
        cr.register_filetype("DESIGNSPACE", designspace);
        cr.register_filetype("UFO", ufo);

        cr.register_simple_profile("designspace", vec![path_direction])
    }
}

#[cfg(not(target_family = "wasm"))]
pluginator::plugin_implementation!(fontspector_checkapi::Plugin, Designspace);
