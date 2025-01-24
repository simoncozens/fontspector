mod can_render_samples;
mod copyright;
mod license;
mod subsets_correct;
mod validate;
pub use can_render_samples::can_render_samples;
pub use copyright::copyright;
pub use license::license;
pub use subsets_correct::subsets_correct;
pub use validate::validate;

include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
pub(crate) use fonts_public::FamilyProto;
use fontspector_checkapi::{Testable, CheckError};

pub(crate) fn family_proto(t: &Testable) -> Result<FamilyProto, CheckError> {
    let mdpb = std::str::from_utf8(&t.contents)
        .map_err(|_| CheckError::Error("METADATA.pb is not valid UTF-8".to_string()))?;
    protobuf::text_format::parse_from_str::<FamilyProto>(mdpb)
        .map_err(|e| CheckError::Error(format!("Error parsing METADATA.pb: {}", e)))
}
