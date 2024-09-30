use serde_json::{Map, Value};

use crate::Check;

#[derive(Debug, Clone)]
pub struct Context {
    pub skip_network: bool,
    pub network_timeout: Option<u64>,
    pub configuration: Map<String, Value>,
    pub check_metadata: Value,
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
    ) -> Self {
        Context {
            skip_network: self.skip_network,
            network_timeout: self.network_timeout,
            configuration: configuration
                .get(check.id)
                .and_then(|x| x.as_object())
                .cloned()
                .unwrap_or_default(),
            check_metadata: check.metadata(),
        }
    }
}
