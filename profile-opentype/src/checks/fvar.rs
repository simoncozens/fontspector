use std::collections::{HashMap, HashSet};

use fontspector_checkapi::{prelude::*, skip, testfont, FileTypeConvert, TestFont};
use read_fonts::types::{F2Dot14, NameId};
use skrifa::{outline::OutlinePen, MetadataProvider};

const REGULAR_COORDINATE_EXPECTATIONS: [(&str, f32); 4] = [
    ("wght", 400.0),
    ("wdth", 100.0),
    ("slnt", 0.0),
    ("ital", 0.0),
];

const REGISTERED_AXIS_TAGS: [&str; 5] = ["ital", "opsz", "slnt", "wdth", "wght"];

fn find_regular(f: TestFont) -> Option<HashMap<String, f32>> {
    let mut instance = f.named_instances().find(|(name, _loc)| name == "Regular");
    if instance.is_none() {
        instance = f.named_instances().find(|(name, _loc)| name == "Italic");
    }
    if instance.is_none() {
        // Should not happen but anyway
        instance = f
            .named_instances()
            .find(|(name, _loc)| name == "Regular Italic");
    }

    instance.map(|(_name, loc)| loc)
}

#[check(
    id = "opentype/fvar/regular_coords_correct",
    title = "Axes and named instances fall within correct ranges?",
    rationale = "According to the Open-Type spec's registered design-variation tags, instances in a variable font should have certain prescribed values.
        If a variable font has a 'wght' (Weight) axis, the valid coordinate range is 1-1000.
        If a variable font has a 'wdth' (Width) axis, the valid numeric range is strictly greater than zero.
        If a variable font has a 'slnt' (Slant) axis, then the coordinate of its 'Regular' instance is required to be 0.
        If a variable font has a 'ital' (Slant) axis, then the coordinate of its 'Regular' instance is required to be 0.",
    proposal = "https://github.com/fonttools/fontbakery/issues/2572"
)]
fn regular_coords_correct(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");
    let mut problems = vec![];
    if let Some(regular_location) = find_regular(f) {
        for (axis, expected) in REGULAR_COORDINATE_EXPECTATIONS {
            if let Some(actual) = regular_location.get(axis) {
                if *actual != expected {
                    problems.push(Status::fail(
                        &format!("{axis}-not-{expected:0}"),
                        &format!(
                            "Regular instance has {} coordinate of {}, expected {}",
                            axis, actual, expected
                        ),
                    ));
                }
            }
        }

        if let Some(actual) = regular_location.get("opsz") {
            if !(10.0..16.0).contains(actual) {
                problems.push(Status::warn(
                    "opsz",
                    &format!(
                        "Regular instance has opsz coordinate of {}, expected between 10 and 16",
                        actual
                    ),
                ));
            }
        }
    } else {
        return Ok(Status::just_one_fail(
            "no-regular-instance",
            "\"Regular\" instance not present.",
        ));
    }
    return_result(problems)
}

#[check(
    id = "opentype/fvar/axis_ranges_correct",
    title = "Axes and named instances fall within correct ranges?",
    rationale = "According to the Open-Type spec's registered design-variation tags, instances in a variable font should have certain prescribed values.
        If a variable font has a 'wght' (Weight) axis, the valid coordinate range is 1-1000.
        If a variable font has a 'wdth' (Width) axis, the valid numeric range is strictly greater than zero.
        If a variable font has a 'slnt' (Slant) axis, then the coordinate of its 'Regular' instance is required to be 0.
        If a variable font has a 'ital' (Slant) axis, then the coordinate of its 'Regular' instance is required to be 0.",
    proposal = "https://github.com/fonttools/fontbakery/issues/2572"
)]
fn axis_ranges_correct(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");

    let mut problems = vec![];
    for (name, location) in f.named_instances() {
        if let Some(wght) = location.get("wght") {
            if !(1.0..=1000.0).contains(wght) {
                problems.push(Status::fail(
                    "wght-out-of-range",
                    &format!(
                        "Instance {} has wght coordinate of {}, expected between 1 and 1000",
                        name, wght
                    ),
                ));
            }
        }
        if let Some(wdth) = location.get("wdth") {
            if *wdth < 1.0 {
                problems.push(Status::fail(
                    "wdth-out-of-range",
                    &format!(
                        "Instance {} has wdth coordinate of {}, expected at least 1",
                        name, wdth
                    ),
                ));
            }
            if *wdth > 1000.0 {
                problems.push(Status::warn(
                    "wdth-greater-than-1000",
                    &format!(
                        "Instance {} has wdth coordinate of {}, which is valid but unusual",
                        name, wdth
                    ),
                ));
            }
        }
    }

    let axes = f.font().axes();
    if let Some(ital) = axes.iter().find(|axis| axis.tag() == "ital") {
        if !(ital.min_value() == 0.0 && ital.max_value() == 1.0) {
            problems.push(Status::fail(
                "invalid-ital-range",
                &format!(
                    "The range of values for the \"ital\" axis in this font is {} to {}.
                    The italic axis range must be 0 to 1, where Roman is 0 and Italic 1.
                    If you prefer a bigger variation range consider using the \"Slant\" axis instead of \"Italic\".",
                    ital.min_value(), ital.max_value()
                ),
            ));
        }
    }

    if let Some(slnt) = axes.iter().find(|axis| axis.tag() == "slnt") {
        if !(slnt.min_value() < 0.0 && slnt.max_value() >= 0.0) {
            problems.push(Status::warn(
                "unusual-slnt-range",
                &format!(
                    "The range of values for the \"slnt\" axis in this font only allows positive coordinates (from {} to {}),
                    indicating that this may be a back slanted design, which is rare. If that's not the case, then
                    the \"slnt\" axis should be a range of negative values instead.",
                    slnt.min_value(), slnt.max_value()
                ),
            ));
        }
    }
    return_result(problems)
}

#[check(
    id = "opentype/varfont/distinct_instance_records",
    title = "Validates that all of the instance records in a given font have distinct data",
    rationale = "According to the 'fvar' documentation in OpenType spec v1.9
        https://docs.microsoft.com/en-us/typography/opentype/spec/fvar

        All of the instance records in a font should have distinct coordinates
        and distinct subfamilyNameID and postScriptName ID values. If two or more
        records share the same coordinates, the same nameID values or the same
        postScriptNameID values, then all but the first can be ignored.",
    proposal = "https://github.com/fonttools/fontbakery/issues/3706"
)]
fn distinct_instance_records(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");

    let mut problems = vec![];
    let mut unique_records = HashSet::new();
    // We want to get at subfamily and postscript name IDs, so we use the lower-level
    // Skrifa API here.
    for instance in f.font().named_instances().iter() {
        let loc = instance.location();
        let coords: Vec<_> = loc.coords().to_vec();
        let subfamily_name_id = instance.subfamily_name_id();
        let postscript_name_id = instance.postscript_name_id();
        let instance_data = (coords.clone(), subfamily_name_id, postscript_name_id);
        if unique_records.contains(&instance_data) {
            let subfamily = f
                .get_name_entry_strings(subfamily_name_id)
                .next()
                .unwrap_or_else(|| format!("ID {}", subfamily_name_id));
            problems.push(Status::warn(
                &format!("repeated-instance-record:{subfamily}"),
                &format!(
                    "Instance {} with coordinates {:?} is duplicated",
                    subfamily, coords
                ),
            ));
        } else {
            unique_records.insert(instance_data);
        }
    }
    return_result(problems)
}

#[check(
    id = "opentype/varfont/family_axis_ranges",
    title = "Check that family axis ranges are identical",
    rationale = "Between members of a family (such as Roman & Italic), the ranges of variable axes must be identical.",
    proposal = "https://github.com/fonttools/fontbakery/issues/4445",
    implementation = "all"
)]
fn family_axis_ranges(c: &TestableCollection, context: &Context) -> CheckFnResult {
    let mut fonts = TTF.from_collection(c);
    fonts.retain(|f| f.is_variable_font());
    skip!(
        fonts.len() < 2,
        "not-enough-fonts",
        "Not enough variable fonts to compare"
    );
    let values: Vec<_> = fonts
        .iter()
        .map(|f| {
            let label = f
                .filename
                .file_name()
                .map(|x| x.to_string_lossy())
                .map(|x| x.to_string())
                .unwrap_or("Unknown file".to_string());
            let comparable = f
                .axis_ranges()
                .map(|(ax, min, def, max)| format!("{}={:.2}:{:.2}:{:.2}", ax, min, def, max))
                .collect::<Vec<String>>()
                .join(", ");
            (comparable.clone(), comparable, label)
        })
        .collect();
    assert_all_the_same(
        context,
        &values,
        "axis-range-mismatch",
        "Variable axis ranges not matching between font files",
    )
}

#[check(
    id = "opentype/varfont/foundry_defined_tag_name",
    title = "Validate foundry-defined design-variation axis tag names.",
    rationale = "According to the OpenType spec's syntactic requirements for
    foundry-defined design-variation axis tags available at
    https://learn.microsoft.com/en-us/typography/opentype/spec/dvaraxisreg

    Foundry-defined tags must begin with an uppercase letter
    and must use only uppercase letters or digits.",
    proposal = "https://github.com/fonttools/fontbakery/issues/4043"
)]
fn varfont_foundry_defined_tag_name(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");
    let mut problems = vec![];
    for axis in f.font().axes().iter() {
        let tag = axis.tag().to_string();
        if REGISTERED_AXIS_TAGS.contains(&tag.as_str()) {
            continue;
        }
        if REGISTERED_AXIS_TAGS.contains(&tag.to_lowercase().as_str()) {
            problems.push(Status::warn("foundry-defined-similar-registered-name",
                &format!("Foundry-defined axis tag {} is similar to a registered tag name {}, consider renaming. If this tag was meant to be a registered tag, please use all lowercase letters in the tag name.", tag, tag.to_lowercase())
            ));
        }
        // Axis tag must be uppercase and contain only uppercase letters or digits
        if !tag
            .chars()
            .next()
            .map(|c| c.is_ascii_uppercase())
            .unwrap_or(false)
        {
            problems.push(Status::fail(
                "invalid-foundry-defined-tag-first-letter",
                &format!(
                    "Foundry-defined axis tag {} must begin with an uppercase letter",
                    tag
                ),
            ))
        } else if !tag
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            problems.push(Status::fail("invalid-foundry-defined-tag-chars",
                &format!("Foundry-defined axis tag {} must begin with an uppercase letter and contain only uppercase letters or digits.", tag)
            ));
        }
    }
    return_result(problems)
}

#[check(
    id = "opentype/varfont/same_size_instance_records",
    title = "Validates that all of the instance records in a given font have the same size",
    rationale = "According to the 'fvar' documentation in OpenType spec v1.9
        https://docs.microsoft.com/en-us/typography/opentype/spec/fvar

        All of the instance records in a given font must be the same size, with
        all either including or omitting the postScriptNameID field. [...]
        If the value is 0xFFFF, then the value is ignored, and no PostScript name
        equivalent is provided for the instance.",
    proposal = "https://github.com/fonttools/fontbakery/issues/3705"
)]
fn same_size_instance_records(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");
    skip!(
        f.font().named_instances().is_empty(),
        "no-instance-records",
        "Font has no instance records."
    );
    let has_or_hasnt_postscriptname: HashSet<bool> = f
        .font()
        .named_instances()
        .iter()
        .map(|ni| {
            ni.postscript_name_id().is_none() ||
            // Work around https://github.com/googlefonts/fontations/issues/1204
            ni.postscript_name_id() == Some(NameId::new(0xFFFF))
        })
        .collect();
    Ok(if has_or_hasnt_postscriptname.len() > 1 {
        Status::just_one_fail(
            "different-size-instance-records",
            "Instance records don't all have the same size.",
        )
    } else {
        Status::just_one_pass()
    })
}

#[check(
    id = "opentype/varfont/valid_nameids",
    title = "Validates that all of the name IDs in an instance record are within the correct range",
    rationale = r#"
        According to the 'fvar' documentation in OpenType spec v1.9
        https://docs.microsoft.com/en-us/typography/opentype/spec/fvar

        The axisNameID field provides a name ID that can be used to obtain strings
        from the 'name' table that can be used to refer to the axis in application
        user interfaces. The name ID must be greater than 255 and less than 32768.

        The postScriptNameID field provides a name ID that can be used to obtain
        strings from the 'name' table that can be treated as equivalent to name
        ID 6 (PostScript name) strings for the given instance. Values of 6 and
        "undefined" can be used; otherwise, values must be greater than 255 and
        less than 32768.

        The subfamilyNameID field provides a name ID that can be used to obtain
        strings from the 'name' table that can be treated as equivalent to name
        ID 17 (typographic subfamily) strings for the given instance. Values of
        2 or 17 can be used; otherwise, values must be greater than 255 and less
        than 32768.
    "#,
    proposal = "https://github.com/fonttools/fontbakery/issues/3703"
)]
fn varfont_valid_nameids(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");
    let mut problems = vec![];
    let valid_nameid = |n: NameId| (256..32768).contains(&n.to_u16());
    let valid_subfamily_nameid = |n: NameId| matches!(n.to_u16(), 2 | 17 | 256..32768);

    // Do the axes first
    for axis in f.font().axes().iter() {
        let axis_name_id = axis.name_id();
        if !valid_nameid(axis_name_id) {
            problems.push(Status::fail(
                &format!("invalid-axis-nameid:{axis_name_id}"),
                &format!(
                    "Axis name ID {} ({}) is out of range. It must be greater than 255 and less than 32768.",
                    axis_name_id, f.get_name_entry_strings(axis_name_id).next().unwrap_or_default()
                ),
            ));
        }
    }

    for instance in f.font().named_instances().iter() {
        let subfamily_name_id = instance.subfamily_name_id();
        if let Some(n) = instance.postscript_name_id() {
            if n != NameId::new(6) && !valid_nameid(n) {
                problems.push(Status::fail(
                        &format!("invalid-postscript-nameid:{}", n.to_u16()),
                        &format!(
                            "PostScript name ID {} ({}) is out of range. It must be greater than 255 and less than 32768, or 6 or 0xFFFF.",
                            n, f.get_name_entry_strings(n).next().unwrap_or_default()
                        ),
                    ));
            }
        }
        if !valid_subfamily_nameid(subfamily_name_id) {
            problems.push(Status::fail(
                &format!("invalid-subfamily-nameid:{}", subfamily_name_id.to_u16()),
                &format!(
                    "Instance subfamily name ID {} ({}) is out of range. It must be greater than 255 and less than 32768.",
                    subfamily_name_id, f.get_name_entry_strings(subfamily_name_id).next().unwrap_or_default()
                ),
            ));
        }
    }
    return_result(problems)
}

#[check(
    id = "opentype/varfont/valid_default_instance_nameids",
    title = "Validates that when an instance record is included for the default instance, its subfamilyNameID value is set to a name ID whose string is equal to the string of either name ID 2 or 17, and its postScriptNameID value is set to a name ID whose string is equal to the string of name ID 6.",
    rationale = r#"
        According to the 'fvar' documentation in OpenType spec v1.9.1
        https://docs.microsoft.com/en-us/typography/opentype/spec/fvar

        The default instance of a font is that instance for which the coordinate
        value of each axis is the defaultValue specified in the corresponding
        variation axis record. An instance record is not required for the default
        instance, though an instance record can be provided. When enumerating named
        instances, the default instance should be enumerated even if there is no
        corresponding instance record. If an instance record is included for the
        default instance (that is, an instance record has coordinates set to default
        values), then the nameID value should be set to either 2 or 17 or to a
        name ID with the same value as name ID 2 or 17. Also, if a postScriptNameID is
        included in instance records, and the postScriptNameID value should be set
        to 6 or to a name ID with the same value as name ID 6.
    "#,
    proposal = "https://github.com/fonttools/fontbakery/issues/3708"
)]
fn varfont_valid_default_instance_nameids(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");
    let mut problems = vec![];
    let has_a_postscriptname = f
        .font()
        .named_instances()
        .iter()
        .any(|ni| ni.postscript_name_id().is_some());
    let name2 = f
        .get_name_entry_strings(NameId::new(2))
        .next()
        .unwrap_or_default();
    let name6 = f
        .get_name_entry_strings(NameId::new(6))
        .next()
        .unwrap_or_default();
    let name17 = f
        .get_name_entry_strings(NameId::new(17))
        .next()
        .unwrap_or_default();
    let font_subfamily_name = if !name17.is_empty() {
        name17.clone()
    } else {
        name2.clone()
    };
    let default_coords = vec![F2Dot14::from_f32(0.0); f.font().axes().len()];
    for (index, instance) in f.font().named_instances().iter().enumerate() {
        if instance.location().coords() != default_coords {
            continue;
        }
        let subfamily_name = f
            .get_name_entry_strings(instance.subfamily_name_id())
            .next()
            .unwrap_or_else(|| format!("instance {}", index + 1));
        let postscript_name = instance
            .postscript_name_id()
            .and_then(|n| f.get_name_entry_strings(n).next())
            .unwrap_or("None".to_string());
        if !name17.is_empty() && subfamily_name != font_subfamily_name {
            problems.push(Status::fail("invalid-default-instance-subfamily-name", &format!(
                "{} instance has the same coordinates as the default instance; its subfamily name should be {}.\n\nNote: It is alternatively possible that Name ID 17 is incorrect, and should be set to the default instance subfamily name, {}, rather than '{}'. If the default instance is {}, NameID 17 is probably the problem.",
                subfamily_name, font_subfamily_name, font_subfamily_name, name17, subfamily_name
            )))
        }
        if name17.is_empty() && subfamily_name != font_subfamily_name {
            problems.push(Status::fail("invalid-default-instance-subfamily-name", &format!(
                "{} instance has the same coordinates as the default instance; its subfamily name should be {}.\n\nNote: If the default instance really is meant to be called {}, the problem may be that the font lacks NameID 17, which should probably be present and set to {}.",
                subfamily_name, font_subfamily_name, subfamily_name, subfamily_name
            )))
        }
        if has_a_postscriptname && postscript_name != name6 {
            problems.push(Status::fail("invalid-default-instance-postscript-name", &format!(
                "{} instance has the same coordinates as the default instance; its postscript name should be {} instead of {}.",
                subfamily_name, name6, postscript_name
            )));
        }
    }
    return_result(problems)
}

struct XDeltaPen {
    highest_point: Option<(f32, f32)>,
    lowest_point: Option<(f32, f32)>,
}

impl XDeltaPen {
    fn new() -> Self {
        XDeltaPen {
            highest_point: None,
            lowest_point: None,
        }
    }

    fn update(&mut self, x: f32, y: f32) {
        if let Some((_hx, hy)) = self.highest_point {
            if y > hy {
                self.highest_point = Some((x, y));
            }
        } else {
            self.highest_point = Some((x, y));
        }
        if let Some((_lx, ly)) = self.lowest_point {
            if y < ly {
                self.lowest_point = Some((x, y));
            }
        } else {
            self.lowest_point = Some((x, y));
        }
    }

    fn x_delta(&self) -> f32 {
        if let (Some((hx, _)), Some((lx, _))) = (self.highest_point, self.lowest_point) {
            hx - lx
        } else {
            0.0
        }
    }
}

impl OutlinePen for XDeltaPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.update(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.update(x, y);
    }
    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.update(cx0, cy0);
        self.update(x, y);
    }
    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.update(cx0, cy0);
        self.update(cx1, cy1);
        self.update(x, y);
    }
    fn close(&mut self) {}
}

#[check(
    id = "opentype/slant_direction",
    rationale = "
        The 'slnt' axis values are defined as negative values for a clockwise (right)
        lean, and positive values for counter-clockwise lean. This is counter-intuitive
        for many designers who are used to think of a positive slant as a lean to
        the right.

        This check ensures that the slant axis direction is consistent with the specs.

        https://docs.microsoft.com/en-us/typography/opentype/spec/dvaraxistag_slnt
    ",
    proposal = "https://github.com/fonttools/fontbakery/pull/3910",
    title = "Checking direction of slnt axis angles"
)]
fn slant_direction(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    skip!(!f.is_variable_font(), "not-variable", "Not a variable font");
    let (_a, slnt_min, _dflt, slnt_max) = f
        .axis_ranges()
        .find(|(a, _min, _dflt, _max)| a == "slnt")
        .ok_or_else(|| CheckError::skip("no-slnt", "No 'slnt' axis found"))?;
    let h_id = f
        .font()
        .charmap()
        .map('H')
        .ok_or_else(|| CheckError::skip("no-H", "No H glyph in font"))?;
    // Get outline at slnt_max
    let mut max_pen = XDeltaPen::new();
    f.draw_glyph(h_id, &mut max_pen, vec![("slnt", slnt_max)])?;
    let mut min_pen = XDeltaPen::new();
    f.draw_glyph(h_id, &mut min_pen, vec![("slnt", slnt_min)])?;
    if min_pen.x_delta() > max_pen.x_delta() {
        Ok(Status::just_one_pass())
    } else {
        Ok(Status::just_one_fail(
            "positive-value-for-clockwise-lean",
            "The right-leaning glyphs have a positive 'slnt' axis value, which is likely a mistake. It needs to be negative to lean rightwards.",
        ))
    }
}
