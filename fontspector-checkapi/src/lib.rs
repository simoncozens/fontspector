mod check;
mod font;
mod registry;
mod status;
pub use check::{return_result, Check, CheckResult};
pub use font::{FontCollection, TestFont};
pub use registry::CheckRegistry;
pub use status::{Status, StatusCode, StatusList};

pub trait Plugin {
    fn provide_checks(&self, cr: &mut CheckRegistry);
    // fn register_profile(&self, profile: Profile);
}

pluginator::plugin_trait!(Plugin);
