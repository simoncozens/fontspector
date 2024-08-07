#![deny(clippy::unwrap_used, clippy::expect_used)]
mod check;
mod constants;
mod filetype;
mod font;
mod profile;
mod registry;
mod status;
mod testable;
pub use check::{return_result, Check, CheckId, CheckResult};
pub use filetype::{FileType, FileTypeConvert};
pub use font::{FontCollection, TestFont, TTF};
pub use profile::{Override, Profile};
pub use registry::Registry;
pub use status::{CheckFnResult, Status, StatusCode, StatusList};
pub use testable::Testable;

pub mod prelude {
    pub type FixFnResult = Result<bool, String>;
    pub use crate::{
        return_result, Check, CheckFnResult, FileType, Profile, Registry, Status, StatusList,
        Testable, TTF,
    };
}
pub trait Plugin {
    fn register(&self, cr: &mut Registry) -> Result<(), String>;
}

pluginator::plugin_trait!(Plugin);
