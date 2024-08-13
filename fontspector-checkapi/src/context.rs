use serde_json::{Map, Value};

#[derive(Debug, Clone)]
pub struct Context {
    pub skip_network: bool,
    pub network_timeout: Option<u64>,
    pub configuration: Map<String, Value>,
}
