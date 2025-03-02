use std::sync::LazyLock;

use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use hashbrown::HashSet;
use read_fonts::TableProvider;

const VENDOR_IDS_FILE: &str = include_str!("../../../resources/vendor_ids.txt");
static VENDOR_IDS: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    VENDOR_IDS_FILE
        .lines()
        .filter(|line| !line.is_empty())
        .collect()
});

const BAD_VIDS: [&str; 4] = ["UKWN", "ukwn", "PfEd", "PYRS"];
const SUGGEST_MICROSOFT_VENDORLIST_WEBSITE: &str = "If you registered it recently, then it's safe to ignore this warning message. Otherwise, you should set it to your own unique 4 character code, and register it with Microsoft at https://www.microsoft.com/typography/links/vendorlist.aspx\n";

#[check(
    id = "googlefonts/vendor_id",
    rationale = "
        
        Microsoft keeps a list of font vendors and their respective contact info. This
        list is updated regularly and is indexed by a 4-char \"Vendor ID\" which is
        stored in the achVendID field of the OS/2 table.

        Registering your ID is not mandatory, but it is a good practice since some
        applications may display the type designer / type foundry contact info on some
        dialog and also because that info will be visible on Microsoft's website:

        https://docs.microsoft.com/en-us/typography/vendors/

        This check verifies whether or not a given font's vendor ID is registered in
        that list or if it has some of the default values used by the most common
        font editors.

        Each new FontBakery release includes a cached copy of that list of vendor IDs.
        If you registered recently, you're safe to ignore warnings emitted by this
        check, since your ID will soon be included in one of our upcoming releases.
    
    ",
    proposal = "https://github.com/fonttools/fontbakery/issues/3943",
    proposal = "https://github.com/fonttools/fontbakery/issues/4829",
    title = "Checking OS/2 achVendID."
)]
fn vendor_id(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    let font_vendor_id = f.font().os2()?.ach_vend_id().to_string();
    if font_vendor_id.is_empty() {
        problems.push(Status::warn(
            "not-set",
            &format!(
                "OS/2 VendorID is not set.\n{}",
                SUGGEST_MICROSOFT_VENDORLIST_WEBSITE
            ),
        ));
    } else if BAD_VIDS.contains(&font_vendor_id.as_str()) {
        problems.push(Status::warn(
            "bad",
            &format!(
                "OS/2 VendorID is '{}', a font editor default.\n{}",
                font_vendor_id, SUGGEST_MICROSOFT_VENDORLIST_WEBSITE
            ),
        ));
    } else if !VENDOR_IDS.contains(&font_vendor_id.as_str()) {
        problems.push(Status::warn(
            "unknown",
            &format!(
                "OS/2 VendorID value '{}' is not yet recognized.\n{}",
                font_vendor_id, SUGGEST_MICROSOFT_VENDORLIST_WEBSITE
            ),
        ));
    }
    return_result(problems)
}
