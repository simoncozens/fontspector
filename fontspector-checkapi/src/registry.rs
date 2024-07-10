use crate::{Check, Plugin};

#[derive(Default)]
pub struct Registry<'a> {
    pub checks: Vec<Check<'a>>,
}

impl Registry<'_> {
    pub fn new() -> Registry<'static> {
        Registry { checks: vec![] }
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
}
