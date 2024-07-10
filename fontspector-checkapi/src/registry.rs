use std::collections::HashMap;

use crate::{Check, CheckId, FileType, Profile, TTF};

#[derive(Default)]
pub struct Registry<'a> {
    pub checks: HashMap<CheckId, Check<'a>>,
    profiles: HashMap<String, Profile>,
    pub(crate) filetypes: HashMap<String, FileType<'a>>,
}

impl<'a> Registry<'a> {
    pub fn new() -> Registry<'static> {
        let mut reg = Registry::default();
        reg.register_filetype("TTF", TTF);
        reg
    }

    pub fn iter(&self) -> impl Iterator<Item = &Check> {
        self.checks.values()
    }

    pub fn load_plugin(&mut self, plugin_path: &str) {
        let plugin = unsafe { crate::load_plugin(plugin_path) }.unwrap_or_else(|e| {
            panic!("Could not load plugin {:?}: {:?}", plugin_path, e);
        });
        plugin.register(self);
    }

    pub fn register_profile(&mut self, name: &str, profile: Profile) {
        self.profiles.insert(name.to_string(), profile);
    }

    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    pub fn register_filetype(&mut self, name: &str, filetype: FileType<'a>) {
        self.filetypes.insert(name.to_string(), filetype);
    }

    pub fn register_check(&mut self, check: Check<'a>) {
        self.checks.insert(check.id.to_string(), check);
    }

    pub fn register_simple_profile(&mut self, name: &str, checks: Vec<Check<'a>>) {
        let mut profile = Profile::default();
        profile.sections.insert(
            name.to_string(),
            checks.iter().map(|c| c.id.to_string()).collect(),
        );
        self.register_profile(name, profile);
        for check in checks {
            self.register_check(check);
        }
    }
}
