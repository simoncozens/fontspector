use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{Check, CheckId, Context, Registry, StatusCode, TestableType};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
/// An override directive
///
/// Profiles may choose to override the status of a check if the vendor
/// decides that they disagree about the severity of a particular problem.
/// To do this, they match against a result code, and provide a new status
/// and a reason for the override.
pub struct Override {
    /// Result code to match against
    code: String,
    /// Overridden status code
    status: StatusCode,
    /// Reason for the override
    reason: String,
}

#[derive(Serialize, Deserialize, Default)]
/// A check profile
///
/// A check profile is a collection of checks that are run together. Vendors
/// define the list of checks that they are interested in running when QA'ing
/// a font, and organise them into sections. They can also override the status
/// of checks, and provide configuration values which are passed to checks to
/// customise their behaviour.
///
/// Profiles are written in the TOML markup language, and can either be baked
/// into fontbakery, provided through plugin modules (this is needed when
/// a vendor profile contains check implementations not contained in the fontbakery
/// core), or provided as a separate TOML file on the command line.
pub struct Profile {
    /// Checks to run
    ///
    /// The list of checks to be run is organised into a number of named
    /// sections, with a list of check IDs in each section.
    pub sections: IndexMap<String, Vec<CheckId>>,
    #[serde(default)]
    /// The list of profiles to include
    ///
    /// Other profiles can be included by name. For example, the `universal`
    /// profile is a superset of the `opentype` provide and includes all its
    /// checks. Vendors may use this functionality to automatically include
    /// all future checks that get added to a particular profile; alternatively,
    /// they can list all the checks they want to run manually to avoid
    /// surprises when new checks are implemented on profiles they intend to use.
    include_profiles: Vec<String>,
    #[serde(default)]
    /// Checks to exclude from included profiles
    ///
    /// When including a profile, it is possible to exclude certain checks
    /// from that profile. This is useful when a vendor wants to use a profile
    /// in part, excluding certain checks.
    exclude_checks: Vec<CheckId>,
    #[serde(default)]
    /// Overrides
    ///
    /// Override the severity of included checks. See [Override] for more information.
    overrides: HashMap<CheckId, Vec<Override>>,
    #[serde(default)]
    /// Configuration defaults
    ///
    /// Each check is passed a [Context] object, which (among other things) contains
    /// a configuration map. This configuration map is built up from the defaults
    /// provided in the profile, and then overlaid with any user-provided configuration.
    /// For example, the `file_size` check can be first configured by the vendor
    /// by providing `WARN_SIZE` and `FAIL_SIZE` values in the profile; the user
    /// can then override these values on the command line.
    configuration_defaults: HashMap<CheckId, HashMap<String, serde_json::Value>>,
}

impl Profile {
    /// Create a new profile
    pub fn from_toml(toml: &str) -> Result<Profile, toml::de::Error> {
        toml::from_str(toml)
    }

    /// Validate a profile
    ///
    /// This function checks that all the checks in the profile are known to the
    /// registry, resolving any included profiles and excluded checks, and that
    /// any filetypes used in checks are known to the registry.
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

    /// Determine a check order based on the profile
    ///
    /// This applies any user-provided command line configuration (include/exclude checks)
    /// and works out which checks apply to the set of [TestableType]s provided.
    /// It returns everything needed to run each check, in order.
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

    /// Get the default configuration for a check
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
