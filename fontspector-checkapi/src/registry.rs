use std::collections::HashMap;

use crate::{Check, CheckId, FileType, Profile, Testable, TTF};

#[derive(Default)]
/// The Registry object
///
/// This allows plugin modules to dynamically register new checks, profiles and filetypes.
pub struct Registry<'a> {
    /// All known checks, by ID
    pub checks: HashMap<CheckId, Check<'a>>,
    /// All known profiles, by name
    pub(crate) profiles: HashMap<String, Profile>,
    /// All known filetypes, by name
    pub(crate) filetypes: HashMap<String, FileType<'a>>,
}

impl<'a> Registry<'a> {
    /// Create a new Registry
    pub fn new() -> Registry<'static> {
        let mut reg = Registry::default();
        reg.register_filetype("TTF", TTF);
        reg
    }

    /// Get an iterator over all checks
    pub fn iter(&self) -> impl Iterator<Item = &Check> {
        self.checks.values()
    }

    /// Load a plugin from a path
    #[cfg(not(target_family = "wasm"))]
    pub fn load_plugin(&mut self, plugin_path: &str) -> Result<(), String> {
        let plugin = unsafe { crate::load_plugin(plugin_path) }.unwrap_or_else(|e| {
            panic!("Could not load plugin {:?}: {:?}", plugin_path, e);
        });
        plugin.register(self)
    }

    /// Register a new [Profile]
    pub fn register_profile(&mut self, name: &str, mut profile: Profile) -> Result<(), String> {
        profile.validate(self)?;
        self.profiles.insert(name.to_string(), profile);
        Ok(())
    }

    /// Get a profile by name
    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    /// Register a new filetype
    pub fn register_filetype(&mut self, name: &str, filetype: FileType<'a>) {
        self.filetypes.insert(name.to_string(), filetype);
    }

    /// Register a new check
    pub fn register_check(&mut self, check: Check<'a>) {
        self.checks.insert(check.id.to_string(), check);
    }

    /// Register a simple profile with a single section from a list of checks
    pub fn register_simple_profile(
        &mut self,
        name: &str,
        checks: Vec<Check<'a>>,
    ) -> Result<(), String> {
        let mut profile = Profile::default();
        profile.sections.insert(
            name.to_string(),
            checks.iter().map(|c| c.id.to_string()).collect(),
        );
        for check in checks {
            self.register_check(check);
        }
        self.register_profile(name, profile)
    }

    /// Returns true if a check has an "experimental" flag
    pub fn is_experimental(&self, check_id: &str) -> bool {
        self.checks
            .get(check_id)
            .is_some_and(|c| c.flags.experimental)
    }

    /// Returns true if a Testable is recognised by any filetype
    pub fn is_known_file(&self, file: &Testable) -> bool {
        self.filetypes.values().any(|ft| ft.applies(file))
    }
}
