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
        // Each testable gets its own context-specific cache.
        let testable_and_cache = testables
            .iter()
            .map(|t| (t, general_context.with_new_cache()));
        // I'm just going to cheat and use nested for loops instead of iterator madness.
        let mut order = vec![];
        let mut sections_and_checks = vec![];
        for (section_name, check_ids) in self.sections.iter() {
            for check_id in check_ids.iter() {
                if !included_excluded(check_id, include_checks, exclude_checks) {
                    continue;
                }
                if registry.checks.contains_key(check_id) {
                    sections_and_checks.push((section_name, check_id))
                } else {
                    log::warn!("Unknown check: {}", check_id);
                }
            }
        }

        for (testable, context) in testable_and_cache {
            for (section_name, check_id) in sections_and_checks.iter() {
                #[allow(clippy::unwrap_used)] // We checked for this above
                let check = registry.checks.get(check_id.as_str()).unwrap();
                if check.applies(testable, registry) {
                    let specialized_context = context.specialize(check, &configuration, self);
                    order.push((
                        section_name.to_string(),
                        testable,
                        check,
                        specialized_context,
                    ));
                }
            }
        }
        order
    }

    /// Get the default configuration for a check
    pub fn defaults(&self, check_id: &str) -> HashMap<String, Value> {
        self.configuration_defaults
            .get(check_id)
            .unwrap_or(&HashMap::new())
            .clone()
    }
}

/// Apply inclusions and exclusions to a list of checks
///
/// Returns true if the check should be included, false if it should be excluded.
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

/// A builder for creating a profile
///
/// This is a convenience builder for creating a profile in code, rather than
/// through a TOML document.
pub struct ProfileBuilder<'a> {
    /// The profile being built
    profile: Profile,
    /// Current section name
    current_section: Option<String>,
    /// Checks to be registered, when we have a registry
    checks_to_register: Vec<Check<'a>>,
}

impl<'a> ProfileBuilder<'a> {
    /// Create a new profile builder
    pub fn new() -> Self {
        ProfileBuilder {
            checks_to_register: vec![],
            profile: Profile::default(),
            current_section: None,
        }
    }

    /// Add a new section to the profile
    pub fn add_section(mut self, name: &str) -> Self {
        self.current_section = Some(name.to_string());
        if !self.profile.sections.contains_key(name) {
            self.profile.sections.insert(name.to_string(), vec![]);
        } else {
            log::warn!("Section {} already exists", name);
        }
        self
    }

    /// Add a check to the current section, registering it with the registry
    pub fn add_and_register_check(mut self, check: Check<'static>) -> Self {
        let check_id = check.id.to_string();
        if let Some(section) = &self.current_section {
            self.checks_to_register.push(check);
            #[allow(clippy::unwrap_used)] // current_section is only Some if we added that section
            self.profile
                .sections
                .get_mut(section)
                .unwrap()
                .push(check_id);
        } else {
            panic!("No section to add check to");
        }
        self
    }

    /// Include another profile
    pub fn include_profile(mut self, profile: &str) -> Self {
        self.profile.include_profiles.push(profile.to_string());
        self
    }

    /// Exclude a check
    pub fn exclude_check(mut self, check: &str) -> Self {
        self.profile.exclude_checks.push(check.to_string());
        self
    }

    /// Add an override for a check
    pub fn with_overrides(mut self, check_id: &str, overrides: Vec<Override>) -> Self {
        self.profile
            .overrides
            .insert(check_id.to_string(), overrides);
        self
    }

    /// Add configuration defaults for a check
    pub fn with_configuration_defaults(
        mut self,
        check_id: &str,
        configuration_defaults: HashMap<String, serde_json::Value>,
    ) -> Self {
        self.profile
            .configuration_defaults
            .insert(check_id.to_string(), configuration_defaults);
        self
    }

    /// Register the profile
    pub fn build(self, name: &str, registry: &mut Registry<'a>) -> Result<(), String> {
        for check in self.checks_to_register {
            registry.register_check(check);
        }
        registry.register_profile(name, self.profile)
    }
}

impl<'a> Default for ProfileBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}
