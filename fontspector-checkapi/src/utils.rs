use crate::Context;

/// Formats a list of items as a Markdown bullet list.
pub fn bullet_list(_context: &Context, items: &[&str]) -> String {
    items
        .iter()
        .map(|item| format!("* {}", item))
        .collect::<Vec<String>>()
        .join("\n")
}
