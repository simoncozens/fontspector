mod axes;
mod can_render_samples;
mod category;
mod consistent_with_fonts;
mod copyright;
mod escaped_strings;
mod familyname;
mod has_regular;
mod license;
mod regular_is_400;
mod reserved_font_name;
mod subsets_correct;
mod validate;
pub use axes::axes;
pub use can_render_samples::can_render_samples;
pub use category::category;
pub use consistent_with_fonts::consistent_with_fonts;
pub use copyright::copyright;
pub use escaped_strings::escaped_strings;
pub use familyname::familyname;
pub use has_regular::has_regular;
pub use license::license;
pub use regular_is_400::regular_is_400;
pub use reserved_font_name::reserved_font_name;
pub use subsets_correct::subsets_correct;
pub use validate::validate;

mod protos {
    #![allow(clippy::all, clippy::unwrap_used)]
    include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
}
pub(crate) use fonts_public::FamilyProto;
use fontspector_checkapi::{CheckError, Testable};
use protos::fonts_public;

pub(crate) fn family_proto(t: &Testable) -> Result<FamilyProto, CheckError> {
    let mdpb = std::str::from_utf8(&t.contents)
        .map_err(|_| CheckError::Error("METADATA.pb is not valid UTF-8".to_string()))?;
    protobuf::text_format::parse_from_str::<FamilyProto>(mdpb)
        .map_err(|e| CheckError::Error(format!("Error parsing METADATA.pb: {}", e)))
}
mod valid_nameid25;
pub use valid_nameid25::valid_nameid25;
#[cfg(not(target_family = "wasm"))]
mod broken_links;
#[cfg(not(target_family = "wasm"))]
pub use broken_links::broken_links;
mod consistent_repo_urls;
pub use consistent_repo_urls::consistent_repo_urls;
mod primary_script;
pub use primary_script::primary_script;
