#![deny(clippy::unwrap_used, clippy::expect_used)]
mod bezglyph;
mod checks;

use fontspector_checkapi::{Profile, Registry};

pub struct OpenType;

impl fontspector_checkapi::Plugin for OpenType {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        cr.register_check(checks::alt_caron::alt_caron);
        cr.register_check(checks::bold_italic_unique::bold_italic_unique);
        cr.register_check(checks::code_pages::code_pages);
        cr.register_check(checks::fvar::axis_ranges_correct);
        cr.register_check(checks::fvar::distinct_instance_records);
        cr.register_check(checks::fvar::regular_coords_correct);
        cr.register_check(checks::fvar::same_size_instance_records);
        cr.register_check(checks::fvar::family_axis_ranges);
        cr.register_check(checks::fvar::varfont_foundry_defined_tag_name);
        cr.register_check(checks::fvar::varfont_valid_default_instance_nameids);
        cr.register_check(checks::fvar::varfont_valid_nameids);
        cr.register_check(checks::fvar::slant_direction);
        cr.register_check(checks::gdef::gdef_mark_chars);
        cr.register_check(checks::gdef::gdef_spacing_marks);
        cr.register_check(checks::gdef_non_mark_chars::gdef_non_mark_chars);
        cr.register_check(checks::glyf::check_glyf_non_transformed_duplicate_components);
        cr.register_check(checks::glyf::check_point_out_of_bounds);
        cr.register_check(checks::glyf::glyf_unused_data);
        cr.register_check(checks::head::equal_font_versions);
        cr.register_check(checks::head::font_version);
        cr.register_check(checks::head::mac_style);
        cr.register_check(checks::head::unitsperem);
        cr.register_check(checks::hhea::caret_slope);
        cr.register_check(checks::hhea::maxadvancewidth);
        cr.register_check(checks::italic_angle::italic_angle);
        cr.register_check(checks::layout::layout_valid_feature_tags);
        cr.register_check(checks::layout::layout_valid_language_tags);
        cr.register_check(checks::layout::layout_valid_script_tags);
        cr.register_check(checks::loca_maxp::loca_maxp_num_glyphs);
        cr.register_check(checks::monospaced::monospace);
        cr.register_check(checks::name::check_name_match_familyname_fullfont);
        cr.register_check(checks::name::consistent_family_name);
        cr.register_check(checks::name::family_max_4_fonts_per_family_name);
        cr.register_check(checks::name::family_naming_recommendations);
        cr.register_check(checks::name::name_empty_records);
        cr.register_check(checks::name::postscript_name);
        cr.register_check(checks::name::postscript_name_consistency);
        cr.register_check(checks::name::name_postscript_vs_cff);
        cr.register_check(checks::os2::check_vendor_id);
        cr.register_check(checks::os2::fsselection);
        cr.register_check(checks::os2::panose_familytype);
        cr.register_check(checks::os2::xavgcharwidth);
        cr.register_check(checks::post::post_table_version);
        cr.register_check(checks::post::underline_thickness);
        cr.register_check(checks::stat::ital_axis);
        cr.register_check(checks::stat::stat_axis_record);
        cr.register_check(checks::stat::stat_has_axis_value_tables);
        cr.register_check(checks::stat::weight_class_fvar);

        let opentype_profile = Profile::from_toml(
            r#"
[sections]
"OpenType Specification Checks" = [
    # Checks which we have definitely ported already
    "opentype/caret_slope",
    "opentype/family_naming_recommendations",
    "opentype/family/bold_italic_unique_for_nameid1",
    "opentype/family/consistent_family_name",
    "opentype/family/equal_font_versions",
    "opentype/family/max_4_fonts_per_family_name",
    "opentype/family/panose_familytype",
    "opentype/family/underline_thickness",
    "opentype/font_version",
    "opentype/fsselection",
    "opentype/fvar/axis_ranges_correct",
    "opentype/fvar/regular_coords_correct",
    "opentype/gdef_mark_chars",
    "opentype/gdef_spacing_marks",
    "opentype/glyf_non_transformed_duplicate_components",
    "opentype/glyf_unused_data",
    "opentype/layout_valid_feature_tags",
    "opentype/layout_valid_language_tags",
    "opentype/layout_valid_script_tags",
    "opentype/mac_style",
    "opentype/maxadvancewidth",
    "opentype/name/empty_records",
    "opentype/name/match_familyname_fullfont",
    "opentype/name/postscript_name_consistency",
    "opentype/name/postscript_vs_cff",
    "opentype/points_out_of_bounds",
    "opentype/postscript_name",
    "opentype/post_table_version",
    "opentype/slant_direction",
    "opentype/stat_has_axis_value_tables",
    "opentype/stat/ital_axis",
    "opentype/unitsperem",
    "opentype/varfont/distinct_instance_records",
    "opentype/varfont/family_axis_ranges",
    "opentype/varfont/foundry_defined_tag_name",
    "opentype/varfont/same_size_instance_records",
    "opentype/varfont/stat_axis_record_for_each_axis",
    "opentype/varfont/valid_nameids",
    "opentype/varfont/valid_default_instance_nameids",
    "opentype/vendor_id",
    "opentype/weight_class_fvar",
    "opentype/xavgcharwidth",
    "opentype/code_pages",
    "opentype/italic_angle",
    "opentype/loca/maxp_num_glyphs",
    "opentype/monospace",
    "opentype/gdef_non_mark_chars",
    "opentype/gpos_kerning_info",

    # Blocked
    # "opentype/kern_table", # https://github.com/googlefonts/fontations/issues/1183
    # "opentype/cff2_call_depth", # We don't have enough CFF support here
    # "opentype/cff_ascii_strings",
    # "opentype/cff_call_depth",
    # "opentype/cff_deprecated_operators",

    # Checks we don't need because they have been integrated into other checks
    # "opentype/dsig", (unwanted_tables)
    # "opentype/varfont/ital_range", (opentype/fvar/axis_ranges_correct)
    # "opentype/varfont/slnt_range",
    # "opentype/varfont/regular_ital_coord", (opentype/fvar/regular_coords_correct)
    # "opentype/varfont/regular_opsz_coord",
    # "opentype/varfont/regular_slnt_coord",
    # "opentype/varfont/regular_wdth_coord",
    # "opentype/varfont/regular_wght_coord",
    # "opentype/varfont/valid_axis_nameid", (now in valid_nameids)
    # "opentype/varfont/valid_postscript_nameid", (now in valid_nameids)
    # "opentype/varfont/valid_subfamily_nameid", (above)
    # "opentype/varfont/wdth_valid_range", (now in opentype/fvar/axis_ranges_correct)
    # "opentype/varfont/wght_valid_range", (above)
    # "opentype/fsselection_matches_macstyle" (merged into opentype/fsselection)
    # "opentype/italic_axis_in_stat", (now opentype/stat/ital_axis)
    # "opentype/italic_axis_in_stat_is_boolean", # (above)
    # "opentype/italic_axis_last", # (above)
 ]
"#,
        )
        .map_err(|_| "Couldn't parse profile")?;
        cr.register_profile("opentype", opentype_profile)
    }
}
