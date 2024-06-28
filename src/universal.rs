use crate::checks::*;
use crate::Check;

pub const UNIVERSAL_PROFILE: [Check<'_>; 4] = [
    BOLD_ITALIC_UNIQUE_CHECK,
    NAME_TRAILING_SPACES_CHECK,
    UNWANTED_TABLES_CHECK,
    REQUIRED_TABLES_CHECK,
];
