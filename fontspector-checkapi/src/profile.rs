use serde::{Deserialize, Serialize};

use crate::{CheckId, Registry, StatusCode};
use std::{collections::HashMap, error::Error};

#[derive(Serialize, Deserialize)]
pub struct Override {
    code: String,
    status: StatusCode,
    reason: String,
}

#[derive(Serialize, Deserialize)]
pub struct Profile {
    pub sections: HashMap<CheckId, Vec<String>>,
    #[serde(default)]
    include_profiles: Vec<String>,
    #[serde(default)]
    exclude_checks: Vec<CheckId>,
    #[serde(default)]
    overrides: HashMap<CheckId, Vec<Override>>,
    #[serde(default)]
    configuration_defaults: HashMap<CheckId, HashMap<String, String>>,
}

impl Profile {
    pub fn from_toml(toml: &str) -> Result<Profile, toml::de::Error> {
        toml::from_str(toml)
    }

    pub fn validate(&self, registry: &Registry) -> Result<(), String> {
        // Resolve "include_profiles" and "exclude_checks" here

        // Ensure we have all the checks we need
        let mut missing_checks = vec![];
        for section in self.sections.values() {
            for check_id in section {
                if !registry.checks.iter().any(|c| c.id == *check_id) {
                    missing_checks.push(check_id.clone());
                }
            }
        }
        if missing_checks.len() > 0 {
            return Err(format!("Missing checks: {}", missing_checks.join(", ")));
        }
        Ok(())
    }
}
