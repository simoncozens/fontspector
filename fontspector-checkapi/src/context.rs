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
        }
    }
}
