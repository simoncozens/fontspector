use fontspector_checkapi::prelude::*;
use freetype;

#[check(
    id = "freetype_rasterizer",
    rationale = "Malformed fonts can cause FreeType to crash.",
    proposal = "https://github.com/fonttools/fontbakery/issues/3642",
    title = "Ensure that the font can be rasterized by FreeType."
)]
fn freetype_rasterizer(f: &Testable, _context: &Context) -> CheckFnResult {
    let library = freetype::Library::init().map_err(|e| {
        CheckError::Error(format!("Failed to initialize FreeType library: {:?}", e))
    })?;
    match library.new_memory_face(f.contents.clone(), 0) {
        Ok(face) => {
            if let Err(failed) = face
                .set_char_size(40 * 64, 0, 50, 0)
                .and_then(|_| face.load_char(0x2705, freetype::face::LoadFlag::RENDER))
            {
                return Ok(Status::just_one_fail("freetype-crash", &failed.to_string()));
            }
        }
        Err(err) => return Ok(Status::just_one_fail("freetype-crash", &err.to_string())),
    }
    Ok(Status::just_one_pass())
}
