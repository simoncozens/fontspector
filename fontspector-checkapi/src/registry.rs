use pluginator::LoadedPlugin;

use crate::{Check, Plugin};

#[derive(Default)]
pub struct CheckRegistry<'a> {
    // checks_by_id: HashMap<String, Check<'a>>,
    pub checks: Vec<Check<'a>>,
    plugins: Vec<Box<LoadedPlugin<dyn Plugin>>>,
}

impl CheckRegistry<'_> {
    pub fn new() -> CheckRegistry<'static> {
        CheckRegistry {
            checks: vec![], // checks_by_id: HashMap::new(),
            plugins: vec![],
        }
    }

    // pub fn add_checks(&mut self, plugin: &'static mut dyn Plugin) {
    //     for check in plugin.provide_checks() {
    //         // self.checks_by_id.insert(check.id.to_string(), check);
    //         self.checks.push(check);
    //     }
    // }

    pub fn iter(&self) -> impl Iterator<Item = &Check> {
        // self.checks_by_id.values()
        self.checks.iter()
    }

    pub fn load_plugin(&mut self, plugin_path: &str) {
        let plugin = unsafe { crate::load_plugin(&plugin_path) }.unwrap_or_else(|e| {
            panic!("Could not load plugin {:?}: {:?}", plugin_path, e);
        });
        plugin.provide_checks(self);
    }
}
