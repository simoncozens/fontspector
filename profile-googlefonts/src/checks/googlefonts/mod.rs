#![allow(non_snake_case)]
#[cfg(not(target_family = "wasm"))]
mod axes_match;

mod color_fonts;
mod font_names;
mod fstype;
mod fvar_instances;
mod has_ttfautohint_params;
mod old_ttfautohint;
mod render_own_name;
mod tofu;
mod unitsperem;
mod use_typo_metrics;
mod vendor_id;
mod weightclass;

pub mod STAT;
pub mod axisregistry;
pub mod description;
pub mod family;
pub mod gasp;
pub mod meta;
pub mod metadata;
pub mod name;
pub mod varfont;

#[cfg(not(target_family = "wasm"))]
pub use axes_match::axes_match;

pub use color_fonts::color_fonts;
pub use font_names::font_names;
pub use fstype::fstype;
pub use fvar_instances::fvar_instances;
pub use gasp::gasp;
pub use has_ttfautohint_params::has_ttfautohint_params;
pub use old_ttfautohint::old_ttfautohint;
pub use render_own_name::render_own_name;
pub use tofu::tofu;
pub use unitsperem::unitsperem;
pub use use_typo_metrics::use_typo_metrics;
pub use vendor_id::vendor_id;
pub use weightclass::weightclass;
mod canonical_filename;
pub use canonical_filename::canonical_filename;
mod version_bump;
pub use version_bump::version_bump;
mod glyph_coverage;
pub use glyph_coverage::glyph_coverage;
mod font_copyright;
pub use font_copyright::font_copyright;
pub mod license;
mod vertical_metrics;
pub use vertical_metrics::vertical_metrics;
