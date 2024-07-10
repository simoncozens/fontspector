use std::collections::HashMap;

use crate::{Check, Plugin, Profile};

#[derive(Default)]
pub struct Registry<'a> {
    pub checks: Vec<Check<'a>>,
    profiles: HashMap<String, Profile>,
}

impl Registry<'_> {
    pub fn new() -> Registry<'static> {
        Registry::default()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Check> {
        self.checks.iter()
    }

    pub fn load_plugin(&mut self, plugin_path: &str) {
        let plugin = unsafe { crate::load_plugin(&plugin_path) }.unwrap_or_else(|e| {
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
}
