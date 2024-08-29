#![deny(clippy::unwrap_used, clippy::expect_used)]
mod checks;
use fontspector_checkapi::{Profile, Registry};

pub struct Universal;

impl fontspector_checkapi::Plugin for Universal {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        cr.register_check(checks::arabic_spacing_symbols::arabic_spacing_symbols);
        cr.register_check(checks::fvar::axis_ranges_correct);
        cr.register_check(checks::fvar::regular_coords_correct);
        cr.register_check(checks::glyphnames::valid_glyphnames);
        cr.register_check(checks::hhea::caret_slope);
        cr.register_check(checks::hhea::maxadvancewidth);
        cr.register_check(checks::name_trailing_spaces::name_trailing_spaces);
        cr.register_check(checks::name::name_empty_records);
        cr.register_check(checks::post::underline_thickness);
        cr.register_check(checks::post::post_table_version);
        cr.register_check(checks::stat::stat_axis_record);
        cr.register_check(checks::required_tables::required_tables);
        cr.register_check(checks::unwanted_tables::unwanted_tables);

        let opentype_profile = Profile::from_toml(
            r#"
[sections]
"OpenType Specification Checks" = [
    "com.google.fonts/check/family/underline_thickness",
    "com.google.fonts/check/family/panose_familytype",
    "com.google.fonts/check/family/equal_font_versions",
    "com.adobe.fonts/check/family/bold_italic_unique_for_nameid1",
    "com.adobe.fonts/check/family/max_4_fonts_per_family_name",
    "com.adobe.fonts/check/family/consistent_family_name",
    "com.adobe.fonts/check/name/postscript_vs_cff",
    "com.adobe.fonts/check/name/postscript_name_consistency",
    "com.adobe.fonts/check/name/empty_records",
    "com.google.fonts/check/name/no_copyright_on_description",
    "com.google.fonts/check/name/match_familyname_fullfont",
    "com.google.fonts/check/fvar/regular_coords_correct",
    "com.google.fonts/check/fvar/axis_ranges_correct",
    "com.google.fonts/check/varfont/stat_axis_record_for_each_axis",
    "com.google.fonts/check/loca/maxp_num_glyphs",
    "com.adobe.fonts/check/cff_ascii_strings",
    "com.adobe.fonts/check/cff_call_depth",
    "com.adobe.fonts/check/cff_deprecated_operators",
    "com.adobe.fonts/check/cff2_call_depth",
    "com.google.fonts/check/font_version",
    "com.google.fonts/check/post_table_version",
    "com.google.fonts/check/monospace",
    "com.google.fonts/check/xavgcharwidth",
    "com.adobe.fonts/check/fsselection_matches_macstyle",
    "com.google.fonts/check/unitsperem",
    "com.google.fonts/check/gdef_spacing_marks",
    "com.google.fonts/check/gdef_mark_chars",
    "com.google.fonts/check/gdef_non_mark_chars",
    "com.google.fonts/check/gpos_kerning_info",
    "com.google.fonts/check/kern_table",
    "com.google.fonts/check/glyf_unused_data",
    "com.google.fonts/check/family_naming_recommendations",
    "com.google.fonts/check/maxadvancewidth",
    "com.adobe.fonts/check/postscript_name",
    "com.google.fonts/check/points_out_of_bounds",
    "com.google.fonts/check/glyf_non_transformed_duplicate_components",
    "com.google.fonts/check/code_pages",
    "com.google.fonts/check/layout_valid_feature_tags",
    "com.google.fonts/check/layout_valid_script_tags",
    "com.google.fonts/check/layout_valid_language_tags",
    "com.google.fonts/check/italic_angle",
    "com.google.fonts/check/mac_style",
    "com.google.fonts/check/fsselection",
    "com.google.fonts/check/name/italic_names",
    "com.adobe.fonts/check/varfont/valid_axis_nameid",
    "com.adobe.fonts/check/varfont/valid_subfamily_nameid",
    "com.adobe.fonts/check/varfont/valid_postscript_nameid",
    "com.adobe.fonts/check/varfont/valid_default_instance_nameids",
    "com.adobe.fonts/check/varfont/same_size_instance_records",
    "com.adobe.fonts/check/varfont/distinct_instance_records",
    "com.adobe.fonts/check/varfont/foundry_defined_tag_name",
    "com.google.fonts/check/varfont/family_axis_ranges",
    "com.adobe.fonts/check/stat_has_axis_value_tables",
    "com.thetypefounders/check/vendor_id",
    "com.google.fonts/check/caret_slope",
    "com.google.fonts/check/italic_axis_in_stat",
    "com.google.fonts/check/italic_axis_in_stat_is_boolean",
    "com.google.fonts/check/italic_axis_last",
]
"#,
        )
        .map_err(|_| "Couldn't parse profile")?;
        cr.register_profile("opentype", opentype_profile)?;

        let universal_profile = Profile::from_toml(
            r#"
include_profiles = ["opentype"]
[sections]
#"Superfamily Checks" = [
#    "com.google.fonts/check/superfamily/list",
#    "com.google.fonts/check/superfamily/vertical_metrics",
#]
#"UFO Sources" = [
#    # FIXME (orphan check): "com.daltonmaag/check/consistent_curve_type",
#    #                       https://github.com/fonttools/fontbakery/pull/4809
#    "com.google.fonts/check/designspace_has_sources",
#    "com.google.fonts/check/designspace_has_default_master",
#    "com.google.fonts/check/designspace_has_consistent_glyphset",
#    "com.google.fonts/check/designspace_has_consistent_codepoints",
#    "com.thetypefounders/check/features_default_languagesystem",
#    # FIXME (orphan check): "com.daltonmaag/check/no_open_corners",
#    "com.daltonmaag/check/ufolint",
#    "com.daltonmaag/check/ufo_required_fields",
#    "com.daltonmaag/check/ufo_recommended_fields",
#    "com.daltonmaag/check/ufo_unnecessary_fields",
#    "com.daltonmaag/check/designspace_has_consistent_groups",
#]
"Universal Profile Checks" = [
    "com.google.fonts/check/alt_caron",
    "com.google.fonts/check/arabic_high_hamza",
    "com.google.fonts/check/arabic_spacing_symbols",
    # "com.google.fonts/check/caps_vertically_centered",  # Disabled: issue #4274
    "com.google.fonts/check/case_mapping",
    "com.google.fonts/check/cjk_chws_feature",
    "com.google.fonts/check/contour_count",
    "com.google.fonts/check/family/single_directory",
    "com.google.fonts/check/family/vertical_metrics",
    "com.google.fonts/check/family/win_ascent_and_descent",
    "com.google.fonts/check/fontbakery_version",
    "com.adobe.fonts/check/freetype_rasterizer",
    "com.google.fonts/check/gpos7",
    "com.google.fonts/check/gsub/smallcaps_before_ligatures",
    "com.google.fonts/check/interpolation_issues",
    "com.google.fonts/check/legacy_accents",
    "com.google.fonts/check/linegaps",
    "com.google.fonts/check/mandatory_glyphs",
    "com.google.fonts/check/math_signs_width",
    "com.google.fonts/check/name/trailing_spaces",
    "com.google.fonts/check/os2_metrics_match_hhea",
    "com.google.fonts/check/ots",
    "com.google.fonts/check/required_tables",
    "com.google.fonts/check/rupee",
    "com.adobe.fonts/check/sfnt_version",
    "com.google.fonts/check/soft_hyphen",
    "com.google.fonts/check/STAT_in_statics",
    "com.google.fonts/check/STAT_strings",
    "com.google.fonts/check/tabular_kerning",
    "com.google.fonts/check/transformed_components",
    "com.google.fonts/check/ttx_roundtrip",
    "com.arrowtype.fonts/check/typoascender_exceeds_Agrave",
    "com.google.fonts/check/unique_glyphnames",
    "com.google.fonts/check/unreachable_glyphs",
    "com.google.fonts/check/unwanted_tables",
    "com.google.fonts/check/valid_glyphnames",
    "com.google.fonts/check/whitespace_glyphnames",
    "com.google.fonts/check/whitespace_glyphs",
    "com.google.fonts/check/whitespace_ink",
    "com.google.fonts/check/whitespace_widths",
]
"#,
        )
        .map_err(|_| "Couldn't parse profile")?;
        cr.register_profile("universal", universal_profile)
    }
}
