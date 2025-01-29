#![allow(non_snake_case)]
mod axes_match;
mod color_fonts;
mod font_names;
mod fstype;
mod fvar_instances;
mod has_ttfautohint_params;
mod render_own_name;
mod tofu;
mod use_typo_metrics;
mod vendor_id;
mod weightclass;

pub mod STAT;
pub mod axisregistry;
pub mod description;
pub mod family;
pub mod gasp;
pub mod metadata;
pub mod name;

pub use axes_match::axes_match;
pub use color_fonts::color_fonts;
pub use font_names::font_names;
pub use fstype::fstype;
pub use fvar_instances::fvar_instances;
pub use gasp::gasp;
pub use has_ttfautohint_params::has_ttfautohint_params;
pub use render_own_name::render_own_name;
pub use tofu::tofu;
pub use use_typo_metrics::use_typo_metrics;
pub use vendor_id::vendor_id;
pub use weightclass::weightclass;
