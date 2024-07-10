mod checks;

pub struct Universal;

impl fontspector_checkapi::Plugin for Universal {
    fn provide_checks(&self, cr: &mut fontspector_checkapi::CheckRegistry) {
        cr.checks.push(checks::BOLD_ITALIC_UNIQUE_CHECK);
        cr.checks.push(checks::NAME_TRAILING_SPACES_CHECK);
    }
}
pluginator::plugin_implementation!(fontspector_checkapi::Plugin, Universal);
