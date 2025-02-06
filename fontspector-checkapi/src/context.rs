use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use serde_json::{Map, Value};

use crate::{Check, Profile};

#[derive(Debug, Clone, Default)]
/// The context of a check
///
/// Generally this means options set by the user on the command line which
/// should be provided to each check.
pub struct Context {
    /// Whether to skip network operations
    pub skip_network: bool,
    /// The network timeout in seconds
    pub network_timeout: Option<u64>,
    /// Additional configuration
    pub configuration: Map<String, Value>,
    /// Metadata in the check's definition
    ///
    /// The purpose of this is to allow multiple checks to share an implementation
    /// function, but differ only in the metadata. For example, in `fontbakery-bridge`
    /// all checks have the same Rust implementation function ("Call a function in a Python
    /// and marshal the results back into Rust"), but need to know which Python function to
    /// call.
    pub check_metadata: Value,
    /// Whether to return full or abbreviated lists of items in check results
    pub full_lists: bool,
    /// A cache, specific to this testable
    pub cache: Arc<RwLock<Map<String, Value>>>,
}

impl Context {
    /// Copy a context, but with a new cache
    pub fn with_new_cache(&self) -> Context {
        Context {
            skip_network: self.skip_network,
            network_timeout: self.network_timeout,
            configuration: self.configuration.clone(),
            check_metadata: self.check_metadata.clone(),
            full_lists: self.full_lists,
            cache: Arc::new(RwLock::new(Map::new())),
        }
    }

    /// Get the configuration for a particular check
    pub fn local_config(&self, check_id: &str) -> Map<String, Value> {
        self.configuration
            .get(check_id)
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default()
    }

    /// Extract a specialized context for a specific check using a configuration map
    ///
    /// This will fill in any default configuration values for the check using
    /// values from the profile.
    pub fn specialize(
        &self,
        check: &Check,
        configuration: &Map<String, serde_json::Value>,
        profile: &Profile,
    ) -> Self {
        // Start with the user's configuration.
        let mut our_copy = configuration.clone();
        // Now fill in any default configuration values for this check
        let check_config_defaults: HashMap<String, Value> = profile.defaults(check.id);
        if !check_config_defaults.is_empty() {
            if let Some(local_config) = our_copy
                .entry(check.id.to_string())
                .or_insert_with(|| Value::Object(Map::new()))
                .as_object_mut()
            {
                for (key, value) in check_config_defaults {
                    local_config.entry(key).or_insert(value);
                }
            }
        }
        Context {
            skip_network: self.skip_network,
            network_timeout: self.network_timeout,
            configuration: our_copy,
            check_metadata: check.metadata(),
            full_lists: self.full_lists,
            cache: self.cache.clone(),
        }
    }

    /// Ask a question, using the cache
    pub fn cached_question<T>(
        &self,
        key: &str,
        func: impl FnOnce() -> Result<T, String>,
        serialize: impl FnOnce(T) -> Value,
        deserialize: impl FnOnce(&Value) -> Result<T, String>,
    ) -> Result<T, String>
    where
        T: Clone,
    {
        if let Ok(cache) = self.cache.read() {
            if let Some(answer) = cache.get(key) {
                let answer_as_t: T = deserialize(answer)?;
                return Ok(answer_as_t);
            }
        }
        let answer = func()?;
        if let Ok(mut cache) = self.cache.write() {
            let answer_as_value: Value = serialize(answer.clone());
            cache.insert(key.to_string(), answer_as_value);
        }
        Ok(answer)
    }
}
