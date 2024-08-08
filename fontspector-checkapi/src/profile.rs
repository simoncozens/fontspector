use serde::{Deserialize, Serialize};

use crate::{CheckId, Registry, StatusCode};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Override {
    code: String,
    status: StatusCode,
    reason: String,
}

#[derive(Serialize, Deserialize, Default)]
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

    pub fn validate(&mut self, registry: &Registry) -> Result<(), String> {
        // Resolve "include_profiles" and "exclude_checks" here
        for included_profile_str in self.include_profiles.iter() {
            if let Some(profile) = registry.profiles.get(included_profile_str) {
                for (section, checks) in &profile.sections {
                    if let Some(existing_checks) = self.sections.get_mut(section) {
                        for check in checks {
                            if !existing_checks.contains(check) {
                                existing_checks.push(check.clone());
                            }
                        }
                    } else {
                        self.sections.insert(section.clone(), checks.clone());
                    }
                }
            } else {
                return Err(format!("Unknown profile: {}", included_profile_str));
            }
        }

        // Ensure we have all the checks we need
        let mut missing_checks = vec![];
        for section in self.sections.values() {
            for check_id in section {
                if !registry.checks.contains_key(check_id) {
                    missing_checks.push(check_id.clone());
                }
            }
        }
        if !missing_checks.is_empty() {
            return Err(format!("Missing checks: {}", missing_checks.join(", ")));
        }
        for check in registry.checks.values() {
            if !registry.filetypes.contains_key(check.applies_to) {
                return Err(format!(
                    "Check {} applies to unknown filetype {}",
                    check.id, check.applies_to
                ));
            }
        }
        Ok(())
    }
}
