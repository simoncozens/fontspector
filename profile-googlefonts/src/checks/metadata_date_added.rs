use crate::metadata::{family_proto, FamilyProto};
use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use skrifa::MetadataProvider;

#[check(
    id = "googlefonts/metadata/date_added",
    rationale = "
        
        The date_added field must not be empty or malformed.

        Expected format is \"YYYY-MM-DD\"
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4729",
    title = "Validate 'date_added' field on METADATA.pb.",
    implementation = "all"
)]
fn metadata_date_added(c: &TestableCollection, _context: &Context) -> CheckFnResult {
    let mut problems = vec![];
    let mdpb = c
        .get_file("METADATA.pb")
        .ok_or_else(|| CheckError::skip("no-mdpb", "No METADATA.pb file found"))?;
    let msg = family_proto(mdpb)?;

    if msg.date_added.as_ref().is_none() || msg.date_added().to_string().is_empty() {
         problems.push(
             Status::fail(
                 "empty",
                 "The date_added field is missing or is empty"
             )
         );
    } else {
         let date_added = msg.date_added().to_string();
         let elements: Vec<_> = date_added.split("-").collect();
         if ! (
             elements.len() == 3  // year, month and day
             && elements[0].len() == 4  //  4 digit year
             && elements[1].len() == 2  //  2 digit month
             && elements[2].len() == 2  //  2 digit day
             && elements[0].parse::<i32>().is_ok()  // year must be a number
             && elements[1].parse::<i32>().is_ok()  // month must be a number
             && elements[2].parse::<i32>().is_ok()  // day must be a number
             && (1..12).contains(&elements[1].parse::<i32>()?)  // from january to december
             && (1..31).contains(&elements[2].parse::<i32>()?)  // acceptable month days
         ) {
             problems.push(
                 Status::fail(
                     "malformed",
                     &format!(
                         "The date_added field has invalid format.\
                          It should be YYYY-MM-DD instead of '{}'",
                         date_added,
                     )
                 )
             );
        }
    }
    return_result(problems)
}
