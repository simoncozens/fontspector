#![deny(clippy::unwrap_used, clippy::expect_used)]
mod checks;
use fontspector_checkapi::{Profile, Registry};

pub struct Universal;

impl fontspector_checkapi::Plugin for Universal {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        // For the OpenType profile:
        cr.register_check(checks::bold_italic_unique::bold_italic_unique);
        cr.register_check(checks::code_pages::code_pages);
        cr.register_check(checks::fvar::axis_ranges_correct);
        cr.register_check(checks::fvar::distinct_instance_records);
        cr.register_check(checks::fvar::family_axis_ranges);
        cr.register_check(checks::fvar::regular_coords_correct);
        cr.register_check(checks::fvar::same_size_instance_records);
        cr.register_check(checks::fvar::slant_direction);
        cr.register_check(checks::fvar::varfont_foundry_defined_tag_name);
        cr.register_check(checks::fvar::varfont_valid_default_instance_nameids);
        cr.register_check(checks::fvar::varfont_valid_nameids);
        cr.register_check(checks::gdef::gdef_mark_chars);
        cr.register_check(checks::gdef::gdef_spacing_marks);
        cr.register_check(checks::glyf::check_glyf_non_transformed_duplicate_components);
        cr.register_check(checks::glyf::check_point_out_of_bounds);
        cr.register_check(checks::glyf::glyf_unused_data);
        cr.register_check(checks::head::equal_font_versions);
        cr.register_check(checks::head::font_version);
        cr.register_check(checks::head::mac_style);
        cr.register_check(checks::head::unitsperem);
        cr.register_check(checks::hhea::caret_slope);
        cr.register_check(checks::hhea::maxadvancewidth);
        cr.register_check(checks::layout::layout_valid_feature_tags);
        cr.register_check(checks::layout::layout_valid_language_tags);
        cr.register_check(checks::layout::layout_valid_script_tags);
        cr.register_check(checks::name::check_name_match_familyname_fullfont);
        cr.register_check(checks::name::check_name_no_copyright_on_description);
        cr.register_check(checks::name::consistent_family_name);
        cr.register_check(checks::name::family_max_4_fonts_per_family_name);
        cr.register_check(checks::name::family_naming_recommendations);
        cr.register_check(checks::name::name_italic_names);
        cr.register_check(checks::name::postscript_name);
        cr.register_check(checks::os2::check_vendor_id);
        cr.register_check(checks::os2::fsselection);
        cr.register_check(checks::os2::panose_familytype);
        cr.register_check(checks::os2::xavgcharwidth);
        cr.register_check(checks::post::post_table_version);
        cr.register_check(checks::post::underline_thickness);
        cr.register_check(checks::stat::stat_axis_record);
        cr.register_check(checks::stat::stat_has_axis_value_tables);
        cr.register_check(checks::stat::ital_axis);
        cr.register_check(checks::stat::weight_class_fvar);

        let opentype_profile = Profile::from_toml(
            r#"
[sections]
"OpenType Specification Checks" = [
    # Checks which we have definitely ported already
    "opentype/fvar/regular_coords_correct",
    "opentype/maxadvancewidth",
    "opentype/caret_slope",
    "opentype/name/empty_records",
    "opentype/family/underline_thickness",
    "opentype/post_table_version",
    "opentype/varfont/stat_axis_record_for_each_axis",
    "opentype/family/bold_italic_unique_for_nameid1",
    "opentype/font_version",
    "opentype/mac_style",
    "opentype/family/equal_font_versions",
    "opentype/unitsperem",
    "opentype/fsselection",
    "opentype/family/panose_familytype",
    "opentype/fvar/axis_ranges_correct",
    "opentype/glyf_unused_data",
    "opentype/points_out_of_bounds",
    "opentype/glyf_non_transformed_duplicate_components",
    "opentype/name/no_copyright_on_description",
    "opentype/varfont/distinct_instance_records",
    "opentype/varfont/family_axis_ranges",
    "opentype/varfont/foundry_defined_tag_name",
    "opentype/varfont/same_size_instance_records",
    "opentype/varfont/valid_nameids",
    "opentype/stat_has_axis_value_tables",
    "opentype/layout_valid_feature_tags",
    "opentype/layout_valid_language_tags",
    "opentype/layout_valid_script_tags",
    "opentype/weight_class_fvar",
    "opentype/vendor_id",
    "opentype/name/match_familyname_fullfont",
    "opentype/varfont/valid_default_instance_nameids",
    "opentype/xavgcharwidth",
    "opentype/family/max_4_fonts_per_family_name",
    "opentype/postscript_name",
    "opentype/family_naming_recommendations",
    "opentype/name/italic_names",
    "opentype/stat/ital_axis",
    "opentype/family/consistent_family_name",

    # Checks left to port
    "opentype/cff2_call_depth",
    "opentype/cff_ascii_strings",
    "opentype/cff_call_depth",
    "opentype/cff_deprecated_operators",
    "opentype/code_pages",

    # Checks we don't need because they have been integrated into other checks
    # "opentype/dsig", (unwanted_tables)
    # "opentype/varfont/ital_range", (opentype/fvar/axis_ranges_correct)
    # "opentype/varfont/wdth_valid_range", (above)
    # "opentype/varfont/wght_valid_range", (above)
    # "opentype/varfont/slnt_range", (above)
    # "opentype/varfont/regular_ital_coord", (opentype/fvar/regular_coords_correct)
    # "opentype/varfont/regular_opsz_coord",
    # "opentype/varfont/regular_slnt_coord",
    # "opentype/varfont/regular_wdth_coord",
    # "opentype/varfont/regular_wght_coord",
    # "opentype/fsselection_matches_macstyle", (merged into opentype/fsselection)
    # "opentype/varfont/valid_axis_nameid", (merged into opentype/varfont/valid_nameids)
    # "opentype/varfont/valid_postscript_nameid", (above)
    # "opentype/varfont/valid_subfamily_nameid", (above)
    # "opentype/italic_axis_in_stat", (merged into opentype/stat/ital_axis)
    # "opentype/italic_axis_in_stat_is_boolean", (above)
    # "opentype/italic_axis_last", (above)

    # Checks I haven't got around to classifying yet
    "opentype/gdef_mark_chars",
    "opentype/gdef_non_mark_chars",
    "opentype/gdef_spacing_marks",
    "opentype/gpos_kerning_info",
    "opentype/italic_angle",
    "opentype/kern_table",
    "opentype/loca/maxp_num_glyphs",
    "opentype/monospace",
    "opentype/name/postscript_name_consistency",
    "opentype/name/postscript_vs_cff",
    "opentype/slant_direction",
]
"#,
        )
        .map_err(|_| "Couldn't parse profile")?;
        cr.register_profile("opentype", opentype_profile)?;

        // For the Universal profile:
        cr.register_check(checks::arabic_spacing_symbols::arabic_spacing_symbols);
        cr.register_check(checks::glyphnames::valid_glyphnames);
        cr.register_check(checks::name::name_empty_records);
        cr.register_check(checks::name_trailing_spaces::name_trailing_spaces);
        cr.register_check(checks::required_tables::required_tables);
        cr.register_check(checks::unwanted_tables::unwanted_tables);

        let universal_profile = Profile::from_toml(
            r#"
include_profiles = ["opentype"]
[sections]
"Superfamily Checks" = [
    "superfamily/list",
    "superfamily/vertical_metrics",
]
"UFO Sources" = [
    "designspace_has_consistent_codepoints",
    "designspace_has_consistent_glyphset",
    "designspace_has_consistent_groups",
    "designspace_has_default_master",
    "designspace_has_sources",
    "ufolint",
    # "ufo_consistent_curve_type",  # FIXME (orphan check) https://github.com/fonttools/fontbakery/pull/4809
    "ufo_features_default_languagesystem",
    # "ufo_no_open_corners",  # FIXME (orphan check) https://github.com/fonttools/fontbakery/pull/4809
    "ufo_recommended_fields",
    "ufo_required_fields",
    "ufo_unnecessary_fields",
]
"Universal Profile Checks" = [
    # Checks which we have definitely ported already
    "arabic_spacing_symbols",
    "valid_glyphnames",
    "name/trailing_spaces",
    "required_tables",

    # Checks left to port

    # Checks we don't need because they have been integrated into other checks
    "whitespace_glyphnames", # integrated into valid_glyphnames

    # Checks I haven't got around to classifying yet

    "alt_caron",
    "arabic_high_hamza",
    # "caps_vertically_centered",  # Disabled: issue #4274
    "case_mapping",
    "cjk_chws_feature",
    "cjk_not_enough_glyphs",
    "cmap/format_12",
    "color_cpal_brightness",
    "colorfont_tables",
    "contour_count",
    "empty_glyph_on_gid1_for_colrv0",
    "empty_letters",
    "family/control_chars",
    "family/single_directory",
    "family/vertical_metrics",
    "family/win_ascent_and_descent",
    "fvar_name_entries",
    "file_size",
    "fontbakery_version",
    "fontdata_namecheck",
    "freetype_rasterizer",
    "glyf_nested_components",
    "gpos7",
    "gsub/smallcaps_before_ligatures",
    "hinting_impact",
    "inconsistencies_between_fvar_stat",
    "integer_ppem_if_hinted",
    "interpolation_issues",
    "legacy_accents",
    "ligature_carets",
    "linegaps",
    "kerning_for_non_ligated_sequences",
    "mandatory_avar_table",
    "mandatory_glyphs",
    "math_signs_width",
    "missing_small_caps_glyphs",
    "name/ascii_only_entries",
    "name/family_and_style_max_length",
    "no_debugging_tables",
    "no_mac_entries",
    "os2_metrics_match_hhea",
    "ots",
    "render_own_name",
    "rupee",
    "sfnt_version",
    "smart_dropout",
    "soft_hyphen",
    "STAT_in_statics",
    "STAT_strings",
    "stylisticset_description",
    "tabular_kerning",
    "transformed_components",
    "ttx_roundtrip",
    "typoascender_exceeds_Agrave",
    "typographic_family_name",
    "unique_glyphnames",
    "unreachable_glyphs",
    "unwanted_aat_tables",
    "unwanted_tables",
    "varfont/consistent_axes",
    "varfont/duplexed_axis_reflow",
    "varfont/instances_in_order",
    "varfont/unsupported_axes",
    "vtt_volt_data",  # very similar to vttclean, may be a good idea to merge them.
    "vttclean",
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
