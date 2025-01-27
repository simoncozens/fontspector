use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use read_fonts::TableProvider;
use skrifa::string::StringId;

#[check(
    id = "googlefonts/name/familyname_first_char",
    rationale = "
        
        Font family names which start with a numeral are often not discoverable
        in Windows applications.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Make sure family name does not begin with a digit."
)]
fn familyname_first_char(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    let name = f.font().name()?;
    for record in name.name_record().iter() {
        let string = record.string(name.string_data())?;
        if record.name_id() == StringId::FAMILY_NAME
            && "0123456789"
                .chars()
                .any(|c| string.to_string().starts_with(c))
        {
            problems.push(Status::fail(
                "begins-with-digit",
                &format!("Font family name '{}' begins with a digit!", string,),
            ));
        }
    }
    return_result(problems)
}
