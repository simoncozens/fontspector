use std::sync::{Arc, RwLock};

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

    /// Extract a specialized context for a specific check using a configuration map
    ///
    /// For example, if the check is `googlefonts/has_metadata`, the configuration map
    /// will be searched for a key `googlefonts/has_metadata` and the value will be used
    /// as the configuration for this check.
    pub fn specialize(
        &self,
        check: &Check,
        configuration: &Map<String, serde_json::Value>,
        profile: &Profile,
    ) -> Self {
        let mut check_config: Map<String, Value> = profile.defaults(check.id).into_iter().collect();
        // Overlay user-provided configuration on top of that.
        if let Some(user_config) = configuration.get(check.id) {
            if let Some(user_config) = user_config.as_object() {
                for (k, v) in user_config {
                    check_config.insert(k.clone(), v.clone());
                }
            }
        }
        Context {
            skip_network: self.skip_network,
            network_timeout: self.network_timeout,
            configuration: check_config,
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
