mod check;
mod font;
mod registry;
mod status;
pub use check::{return_result, Check, CheckResult};
pub use font::{FontCollection, TestFont};
pub use registry::Registry;
pub use status::{Status, StatusCode, StatusList};

pub trait Plugin {
    fn register(&self, cr: &mut Registry);
    // fn register_profile(&self, profile: Profile);
}

pluginator::plugin_trait!(Plugin);
