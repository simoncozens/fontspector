use serde_json::{Map, Value};

use crate::{cache::Cache, Check, Profile};

#[derive(Debug, Clone, Default)]
pub struct Context {
    pub skip_network: bool,
    pub network_timeout: Option<u64>,
    pub configuration: Map<String, Value>,
    pub check_metadata: Value,
    pub full_lists: bool,
    pub font_cache: Cache,
}

impl Context {
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
            font_cache: self.font_cache.clone(),
        }
    }
}
