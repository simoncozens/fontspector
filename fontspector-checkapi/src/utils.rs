use std::fmt::Display;

use crate::{CheckFnResult, Context, Status};

/// Formats a list of items as a Markdown bullet list.
pub fn bullet_list<I>(_context: &Context, items: I) -> String
where
    I: IntoIterator,
    I::Item: Display,
{
    items
        .into_iter()
        .map(|item| format!("* {}", item))
        .collect::<Vec<String>>()
        .join("\n")
}

/// Asserts that all the values in a list are the same.
///
/// Each value is passed as a tuple of three values:
/// * The element to compare
/// * A displayable value to include in the message
/// * A label for the value to include in the message
///
/// For example:
///
/// ```rust,ignore
///     &[
///      (0b00000001, "Italic", "FontA.ttf"),
///      (0b00000001, "Italic", "FontB.ttf"),
///      (0b00100000, "Bold",   "FontC.ttf"),
///     ]
/// ```
///
/// The values are compared for equality.
/// If they are not equal, a failure status is returned, with a message listing all the values.
/// If they are, a pass status is returned.
pub fn assert_all_the_same<T, U, V>(
    _context: &Context,
    values: &[(T, U, V)],
    code: &str,
    message_start: &str,
) -> CheckFnResult
where
    T: Eq,
    U: Display,
    V: Display,
{
    let ok = values.iter().all(|(a, _, _)| a == &values[0].0);
    if ok {
        Ok(Status::just_one_pass())
    } else {
        let message = format!(
            "{}\n\nThe following values were found:\n\n{}",
            message_start,
            bullet_list(
                _context,
                values.iter().map(|(_, a, b)| format!("{}: {}", a, b))
            )
        );
        Ok(Status::just_one_fail(code, &message))
    }
}
