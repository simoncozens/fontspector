#![deny(clippy::unwrap_used, clippy::expect_used)]
mod checks;

use fontspector_checkapi::{ProfileBuilder, Registry};

pub struct OpenType;

impl fontspector_checkapi::Plugin for OpenType {
    fn register(&self, cr: &mut Registry<'_>) -> Result<(), String> {
        let builder = ProfileBuilder::new();
        builder
            .add_section("Opentype Specification Checks")
            .add_and_register_check(checks::opentype::caret_slope)
            // We don't have enough CFF support here:
            //   - "opentype/CFF2_call_depth"
            //   - "opentype/CFF_ascii_strings"
            //   - "opentype/CFF_call_depth"
            //   - "opentype/CFF_deprecated_operators"
            .add_and_register_check(checks::opentype::code_pages)
            .add_and_register_check(checks::opentype::family::bold_italic_unique_for_nameid1)
            .add_and_register_check(checks::opentype::family::consistent_family_name)
            .add_and_register_check(checks::opentype::family::equal_font_versions)
            .add_and_register_check(checks::opentype::family::max_4_fonts_per_family_name)
            .add_and_register_check(checks::opentype::family_naming_recommendations)
            .add_and_register_check(checks::opentype::family::panose_familytype)
            .add_and_register_check(checks::opentype::family::underline_thickness)
            .add_and_register_check(checks::opentype::font_version)
            .add_and_register_check(checks::opentype::fsselection)
            .add_and_register_check(checks::opentype::fvar::axis_ranges_correct)
            .add_and_register_check(checks::opentype::fvar::regular_coords_correct)
            .add_and_register_check(checks::opentype::GDEF_mark_chars)
            .add_and_register_check(checks::opentype::GDEF_non_mark_chars)
            .add_and_register_check(checks::opentype::GDEF_spacing_marks)
            .add_and_register_check(checks::opentype::glyf_non_transformed_duplicate_components)
            .add_and_register_check(checks::opentype::glyf_unused_data)
            .add_and_register_check(checks::opentype::italic_angle)
            // .add_and_register_check(checks::opentype::kern_table) // https://github.com/googlefonts/fontations/issues/1183
            .add_and_register_check(checks::opentype::layout_valid_feature_tags)
            .add_and_register_check(checks::opentype::layout_valid_language_tags)
            .add_and_register_check(checks::opentype::layout_valid_script_tags)
            .add_and_register_check(checks::opentype::loca::maxp_num_glyphs)
            .add_and_register_check(checks::opentype::mac_style)
            .add_and_register_check(checks::opentype::maxadvancewidth)
            .add_and_register_check(checks::opentype::monospace)
            .add_and_register_check(checks::opentype::name::empty_records)
            .add_and_register_check(checks::opentype::name::match_familyname_fullfont)
            .add_and_register_check(checks::opentype::name::postscript_name_consistency)
            .add_and_register_check(checks::opentype::name::postscript_vs_cff)
            .add_and_register_check(checks::opentype::points_out_of_bounds)
            .add_and_register_check(checks::opentype::postscript_name)
            .add_and_register_check(checks::opentype::post_table_version)
            .add_and_register_check(checks::opentype::slant_direction)
            .add_and_register_check(checks::opentype::STAT::ital_axis)
            .add_and_register_check(checks::opentype::unitsperem)
            .add_and_register_check(checks::opentype::varfont::distinct_instance_records)
            .add_and_register_check(checks::opentype::varfont::family_axis_ranges)
            .add_and_register_check(checks::opentype::varfont::foundry_defined_tag_name)
            .add_and_register_check(checks::opentype::varfont::same_size_instance_records)
            .add_and_register_check(checks::opentype::varfont::STAT_axis_record_for_each_axis)
            .add_and_register_check(checks::opentype::varfont::valid_default_instance_nameids)
            .add_and_register_check(checks::opentype::varfont::valid_nameids)
            .add_and_register_check(checks::opentype::vendor_id)
            .add_and_register_check(checks::opentype::weight_class_fvar)
            .add_and_register_check(checks::opentype::xavgcharwidth)
            .build("opentype", cr)
    }
}
