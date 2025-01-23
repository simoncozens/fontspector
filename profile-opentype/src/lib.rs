#![deny(clippy::unwrap_used, clippy::expect_used)]
mod bezglyph;
mod checks;

use fontspector_checkapi::{ProfileBuilder, Registry};

pub struct OpenType;

impl fontspector_checkapi::Plugin for OpenType {
    fn register(&self, cr: &mut Registry<'_>) -> Result<(), String> {
        let builder = ProfileBuilder::new();
        builder
            .add_section("Opentype Specification Checks")
            .add_and_register_check(checks::hhea::caret_slope)
            .add_and_register_check(checks::name::family_naming_recommendations)
            .add_and_register_check(checks::bold_italic_unique::bold_italic_unique)
            .add_and_register_check(checks::name::consistent_family_name)
            .add_and_register_check(checks::head::equal_font_versions)
            .add_and_register_check(checks::name::family_max_4_fonts_per_family_name)
            .add_and_register_check(checks::os2::panose_familytype)
            .add_and_register_check(checks::post::underline_thickness)
            .add_and_register_check(checks::head::font_version)
            .add_and_register_check(checks::os2::fsselection)
            .add_and_register_check(checks::fvar::axis_ranges_correct)
            .add_and_register_check(checks::fvar::regular_coords_correct)
            .add_and_register_check(checks::gdef::gdef_mark_chars)
            .add_and_register_check(checks::gdef::gdef_spacing_marks)
            .add_and_register_check(checks::glyf::check_glyf_non_transformed_duplicate_components)
            .add_and_register_check(checks::glyf::glyf_unused_data)
            .add_and_register_check(checks::layout::layout_valid_feature_tags)
            .add_and_register_check(checks::layout::layout_valid_language_tags)
            .add_and_register_check(checks::layout::layout_valid_script_tags)
            .add_and_register_check(checks::head::mac_style)
            .add_and_register_check(checks::hhea::maxadvancewidth)
            .add_and_register_check(checks::name::name_empty_records)
            .add_and_register_check(checks::name::check_name_match_familyname_fullfont)
            .add_and_register_check(checks::name::postscript_name_consistency)
            .add_and_register_check(checks::name::name_postscript_vs_cff)
            .add_and_register_check(checks::glyf::check_point_out_of_bounds)
            .add_and_register_check(checks::name::postscript_name)
            .add_and_register_check(checks::post::post_table_version)
            .add_and_register_check(checks::fvar::slant_direction)
            .add_and_register_check(checks::stat::ital_axis)
            .add_and_register_check(checks::head::unitsperem)
            .add_and_register_check(checks::fvar::distinct_instance_records)
            .add_and_register_check(checks::fvar::family_axis_ranges)
            .add_and_register_check(checks::fvar::varfont_foundry_defined_tag_name)
            .add_and_register_check(checks::fvar::same_size_instance_records)
            .add_and_register_check(checks::stat::stat_axis_record)
            .add_and_register_check(checks::fvar::varfont_valid_nameids)
            .add_and_register_check(checks::fvar::varfont_valid_default_instance_nameids)
            .add_and_register_check(checks::os2::check_vendor_id)
            .add_and_register_check(checks::stat::weight_class_fvar)
            .add_and_register_check(checks::os2::xavgcharwidth)
            .add_and_register_check(checks::code_pages::code_pages)
            .add_and_register_check(checks::italic_angle::italic_angle)
            .add_and_register_check(checks::loca_maxp::loca_maxp_num_glyphs)
            .add_and_register_check(checks::monospaced::monospace)
            .add_and_register_check(checks::gdef_non_mark_chars::gdef_non_mark_chars)
            // Blocked
            // "opentype/kern_table", # https://github.com/googlefonts/fontations/issues/1183
            // "opentype/cff2_call_depth", # We don't have enough CFF support here
            // "opentype/cff_ascii_strings",
            // "opentype/cff_call_depth",
            // "opentype/cff_deprecated_operators",
            // Checks we don't need because they have been integrated into other checks
            // "opentype/dsig", (unwanted_tables)
            // "opentype/varfont/ital_range", (opentype/fvar/axis_ranges_correct)
            // "opentype/varfont/slnt_range",
            // "opentype/varfont/regular_ital_coord", (opentype/fvar/regular_coords_correct)
            // "opentype/varfont/regular_opsz_coord",
            // "opentype/varfont/regular_slnt_coord",
            // "opentype/varfont/regular_wdth_coord",
            // "opentype/varfont/regular_wght_coord",
            // "opentype/varfont/valid_axis_nameid", (now in valid_nameids)
            // "opentype/varfont/valid_postscript_nameid", (now in valid_nameids)
            // "opentype/varfont/valid_subfamily_nameid", (above)
            // "opentype/varfont/wdth_valid_range", (now in opentype/fvar/axis_ranges_correct)
            // "opentype/varfont/wght_valid_range", (above)
            // "opentype/fsselection_matches_macstyle" (merged into opentype/fsselection)
            // "opentype/italic_axis_in_stat", (now opentype/stat/ital_axis)
            // "opentype/italic_axis_in_stat_is_boolean", # (above)
            // "opentype/italic_axis_last", # (above)
            .build("opentype", cr)
    }
}
