#![deny(clippy::unwrap_used, clippy::expect_used)]
#![deny(missing_docs, clippy::missing_docs_in_private_items)]

//! # fontspector-checkapi
//!
//! This crate provides an API for writing checks for fontspector. It is used by
//! check implementations in the various `profile-` crates. As well as the interfae
//! for "talking" to fontspector, it provides some useful functionality for
//! writing checks on TrueType font files.
//!
//! This crate also exports a `prelude` module containing the most common items you will
//! need when writing checks.
//!
//! Check authors should see also [fontspector-checkhelper](../fontspector-checkhelper)

/// Routines and data structures for defining a check
mod check;
/// Data structures representing the result of a check
mod checkresult;
/// Routines for testing checks
pub mod codetesting;
/// Font-related constants which may be useful to check implementors
pub mod constants;
/// Data structures for managing the context in which a check is run
mod context;
/// Managing a registry of file types
mod filetype;
/// Represents a TrueType font, together with useful routines for dealing with them
mod font;
/// Routines to make dealing with GSUB tables more tractable
mod gsub;
/// [OutlinePen](https://docs.rs/skrifa/latest/skrifa/outline/trait.OutlinePen.html) implementations useful for check implementors
pub mod pens;
/// Sets of checks that declare a particular "standard" of QA testing
mod profile;
/// The registry of checks and profiles
mod registry;
/// Data structures representing the most basic elements of a check's result
mod status;
/// Wraps a file or "thing" to be tested
mod testable;
/// Common utility functions for check implementors
mod utils;
pub use check::{return_result, Check, CheckFlags, CheckId, CheckImplementation, HotfixFunction};
pub use checkresult::{CheckResult, FixResult};
pub use context::Context;
pub use filetype::{FileType, FileTypeConvert};
pub use font::{TestFont, DEFAULT_LOCATION, TTF};
pub use gsub::{GetSubstitutionMap, SubstitutionMap};
pub use profile::{Override, Profile, ProfileBuilder};
pub use registry::Registry;
pub use status::{CheckError, CheckFnResult, Status, StatusCode, StatusList};
pub use testable::{Testable, TestableCollection, TestableType};

/// The prelude module contains the most common items you will need when writing checks
pub mod prelude {
    pub use fontspector_checkhelper::check;

    #[macro_export]
    /// Extract a TTF file from a [Testable] or return an error if you can't
    macro_rules! testfont {
        ($f: ident) => {
            TTF.from_testable($f)
                .ok_or(CheckError::Error("Not a TTF file".to_string()))?
        };
    }
    #[macro_export]
    /// The same as [testfont!] but for hotfixing.
    macro_rules! fixfont {
        ($f: ident) => {
            TTF.from_testable($f).ok_or("Not a TTF file")?
        };
    }
    /// Return a skip status with a code and message
    ///
    /// This macro has two forms:
    /// `skip!(code, message)` which will always return a skip status, and
    /// `skip!(condition, code, message)` which will return a skip status if the condition is true
    #[macro_export]
    macro_rules! skip {
        ($code: expr, $message: expr) => {
            return Ok(Status::just_one_skip($code, $message));
        };
        ($condition: expr, $code: expr, $message: expr) => {
            if $condition {
                return Ok(Status::just_one_skip($code, $message));
            }
        };
    }
    /// The expected return type of a hotfix function
    pub type FixFnResult = Result<bool, String>;
    pub use crate::{
        return_result, utils::*, Check, CheckError, CheckFlags, CheckFnResult, CheckImplementation,
        Context, FileType, Profile, ProfileBuilder, Registry, Status, StatusList, Testable,
        TestableCollection, TestableType, TTF,
    };
}

/// A plugin is a dynamic library that can be loaded by fontspector
///
/// Plugins contain checks and profiles that can be registered with the fontspector
/// registry. The plugin must implement this trait and provide a function that
/// returns an instance of the plugin. See [pluginator](https://docs.rs/pluginator/0.1.0/pluginator/)
pub trait Plugin {
    /// Register the checks and profiles in the plugin with the registry
    fn register(&self, cr: &mut Registry) -> Result<(), String>;
}

/// Load a plugin from a file
///
/// Loads a static library and returns a handle to the loaded plugin
///
/// # Safety
///
/// This function is unsafe because it loads a dynamic library from the filesystem.
/// You're running arbitrary code at this point. Don't use `--plugin` if that
/// bothers you.
//
// Sigh, this is a manual implementation of `pluginator::plugin_trait!` because
// since that crate was written, macros are now normal items and get tested for
// missing docs, but the pluginator macro doesn't produce any docs, and we can't
// document macro-produced code ourselves, and argh.
#[cfg(not(target_family = "wasm"))]
pub unsafe fn load_plugin<Path: AsRef<std::path::Path>>(
    path: Path,
) -> Result<pluginator::LoadedPlugin<dyn Plugin>, pluginator::plugin::LoadingError> {
    unsafe { pluginator::plugin::load(path) }
}
