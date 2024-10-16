use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{Check, CheckId, Context, Registry, StatusCode, TestableType};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Override {
    code: String,
    status: StatusCode,
    reason: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Profile {
    pub sections: IndexMap<String, Vec<CheckId>>,
    #[serde(default)]
    include_profiles: Vec<String>,
    #[serde(default)]
    exclude_checks: Vec<CheckId>,
    #[serde(default)]
    overrides: HashMap<CheckId, Vec<Override>>,
    #[serde(default)]
    configuration_defaults: HashMap<CheckId, HashMap<String, serde_json::Value>>,
}

impl Profile {
    pub fn from_toml(toml: &str) -> Result<Profile, toml::de::Error> {
        toml::from_str(toml)
    }

    pub fn validate(&mut self, registry: &Registry) -> Result<(), String> {
        // Resolve "include_profiles" and "exclude_checks" here
        for included_profile_str in self.include_profiles.iter() {
            if let Some(profile) = registry.profiles.get(included_profile_str) {
                // I want any new included sections to be at the top
                for section in profile.sections.keys().rev() {
                    if !self.sections.contains_key(section) {
                        self.sections.insert_before(0, section.clone(), vec![]);
                    }
                }
                for (section, checks) in &profile.sections {
                    #[allow(clippy::unwrap_used)] // We added all new sections just now
                    let existing_checks = self.sections.get_mut(section).unwrap();
                    for check in checks {
                        if !existing_checks.contains(check) {
                            existing_checks.push(check.clone());
                        }
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

        // #[cfg(not(debug_assertions))]
        // if !missing_checks.is_empty() {
        //     return Err(format!("Missing checks: {}", missing_checks.join(", ")));
        // }

        // #[cfg(debug_assertions)]
        // for missing in missing_checks {
        //     log::warn!("Missing check: {}", missing);
        // }

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

    pub fn check_order<'t, 'r>(
        &self,
        include_checks: &Option<Vec<String>>,
        exclude_checks: &Option<Vec<String>>,
        registry: &'r Registry<'r>,
        general_context: Context,
        configuration: Map<String, serde_json::Value>,
        testables: &'t [TestableType],
    ) -> Vec<(String, &'t TestableType<'t>, &'r Check<'r>, Context)> {
        self.sections
            .iter()
            .flat_map(|(sectionname, checknames)| {
                checknames
                    .iter()
                    .filter(|checkname| {
                        included_excluded(checkname, include_checks, exclude_checks)
                    })
                    .map(|checkname| {
                        (
                            sectionname.clone(),
                            registry.checks.get(checkname),
                            checkname,
                        )
                    })
                    .filter_map(|(sectionname, check, checkname)| {
                        let ck = check.map(|check| {
                            (
                                sectionname,
                                check,
                                general_context.specialize(check, &configuration, self),
                            )
                        });
                        if ck.is_none() {
                            log::warn!("Unknown check: {}", checkname);
                        }
                        ck
                    })
            })
            .flat_map(|(sectionname, check, context): (String, &Check, Context)| {
                testables
                    .iter()
                    .filter(|testable| check.applies(testable, registry))
                    .map(move |testable| (sectionname.clone(), testable, check, context.clone()))
            })
            .collect()
    }

    pub fn defaults(&self, check_id: &str) -> HashMap<String, Value> {
        self.configuration_defaults
            .get(check_id)
            .unwrap_or(&HashMap::new())
            .clone()
    }
}

/// Filter out checks that don't apply
fn included_excluded(
    checkname: &str,
    include_checks: &Option<Vec<String>>,
    exclude_checks: &Option<Vec<String>>,
) -> bool {
    if let Some(checkids) = &include_checks {
        if !checkids.iter().any(|id| checkname.contains(id)) {
            return false;
        }
    }
    if let Some(exclude_checkids) = &exclude_checks {
        if exclude_checkids.iter().any(|id| checkname.contains(id)) {
            return false;
        }
    }
    true
}
