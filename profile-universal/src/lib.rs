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
    "opentype/caret_slope",
    "opentype/cff2_call_depth",
    "opentype/cff_ascii_strings",
    "opentype/cff_call_depth",
    "opentype/cff_deprecated_operators",
    "opentype/code_pages",
    "opentype/family/bold_italic_unique_for_nameid1",
    "opentype/family/consistent_family_name",
    "opentype/family/equal_font_versions",
    "opentype/family/max_4_fonts_per_family_name",
    "opentype/family_naming_recommendations",
    "opentype/family/panose_familytype",
    "opentype/family/underline_thickness",
    "opentype/font_version",
    "opentype/fsselection",
    "opentype/fsselection_matches_macstyle",
    "opentype/fvar/axis_ranges_correct",
    "opentype/fvar/regular_coords_correct",
    "opentype/gdef_mark_chars",
    "opentype/gdef_non_mark_chars",
    "opentype/gdef_spacing_marks",
    "opentype/glyf_non_transformed_duplicate_components",
    "opentype/glyf_unused_data",
    "opentype/gpos_kerning_info",
    "opentype/italic_angle",
    "opentype/italic_axis_in_stat",
    "opentype/italic_axis_in_stat_is_boolean",
    "opentype/italic_axis_last",
    "opentype/kern_table",
    "opentype/layout_valid_feature_tags",
    "opentype/layout_valid_language_tags",
    "opentype/layout_valid_script_tags",
    "opentype/loca/maxp_num_glyphs",
    "opentype/mac_style",
    "opentype/maxadvancewidth",
    "opentype/monospace",
    "opentype/name/empty_records",
    "opentype/name/italic_names",
    "opentype/name/match_familyname_fullfont",
    "opentype/name/no_copyright_on_description",
    "opentype/name/postscript_name_consistency",
    "opentype/name/postscript_vs_cff",
    "opentype/points_out_of_bounds",
    "opentype/postscript_name",
    "opentype/post_table_version",
    "opentype/stat_has_axis_value_tables",
    "opentype/unitsperem",
    "opentype/varfont/distinct_instance_records",
    "opentype/varfont/family_axis_ranges",
    "opentype/varfont/foundry_defined_tag_name",
    "opentype/varfont/same_size_instance_records",
    "opentype/varfont/stat_axis_record_for_each_axis",
    "opentype/varfont/valid_axis_nameid",
    "opentype/varfont/valid_default_instance_nameids",
    "opentype/varfont/valid_postscript_nameid",
    "opentype/varfont/valid_subfamily_nameid",
    "opentype/vendor_id",
    "opentype/xavgcharwidth",
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
#    "superfamily/list",
#    "superfamily/vertical_metrics",
#]
#"UFO Sources" = [
#    # FIXME (orphan check): "ufo_consistent_curve_type",
#    # FIXME (orphan check): "ufo_no_open_corners",
#    #                       https://github.com/fonttools/fontbakery/pull/4809
#    "designspace_has_consistent_codepoints",
#    "designspace_has_consistent_glyphset",
#    "designspace_has_consistent_groups",
#    "designspace_has_default_master",
#    "designspace_has_sources",
#    "ufo_features_default_languagesystem",
#    "ufolint",
#    "ufo_recommended_fields",
#    "ufo_required_fields",
#    "ufo_unnecessary_fields",
#]
"Universal Profile Checks" = [
    "alt_caron",
    "arabic_high_hamza",
    "arabic_spacing_symbols",
    # "caps_vertically_centered",  # Disabled: issue #4274
    "case_mapping",
    "cjk_chws_feature",
    "contour_count",
    "family/single_directory",
    "family/vertical_metrics",
    "family/win_ascent_and_descent",
    "fontbakery_version",
    "freetype_rasterizer",
    "gpos7",
    "gsub/smallcaps_before_ligatures",
    "interpolation_issues",
    "legacy_accents",
    "linegaps",
    "mandatory_glyphs",
    "math_signs_width",
    "name/trailing_spaces",
    "os2_metrics_match_hhea",
    "ots",
    "required_tables",
    "rupee",
    "sfnt_version",
    "soft_hyphen",
    "STAT_in_statics",
    "STAT_strings",
    "tabular_kerning",
    "transformed_components",
    "ttx_roundtrip",
    "typoascender_exceeds_Agrave",
    "unique_glyphnames",
    "unreachable_glyphs",
    "unwanted_tables",
    "valid_glyphnames",
    "whitespace_glyphnames",
    "whitespace_glyphs",
    "whitespace_ink",
    "whitespace_widths",
]
"#,
        )
        .map_err(|_| "Couldn't parse profile")?;
        cr.register_profile("universal", universal_profile)
    }
}
