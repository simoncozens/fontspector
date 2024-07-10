mod check;
mod font;
mod profile;
mod registry;
mod status;
pub use check::{return_result, Check, CheckId, CheckResult};
pub use font::{FontCollection, TestFont};
pub use profile::{Override, Profile};
pub use registry::Registry;
pub use status::{Status, StatusCode, StatusList};

pub trait Plugin {
    fn register(&self, cr: &mut Registry);
}

pluginator::plugin_trait!(Plugin);
