#![deny(clippy::unwrap_used, clippy::expect_used)]
mod checks;
use fontspector_checkapi::{Profile, Registry};

pub struct Universal;

impl fontspector_checkapi::Plugin for Universal {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        cr.register_check(checks::arabic_spacing_symbols::arabic_spacing_symbols);
        cr.register_check(checks::glyphnames::valid_glyphnames);
        cr.register_check(checks::name_no_copyright_on_description::name_no_copyright_on_description);
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
    "name/no_copyright_on_description",
    "name/trailing_spaces",
    "required_tables",

    # Checks left to port

    # Checks we don't need because they have been integrated into other checks
    # "whitespace_glyphnames", integrated into valid_glyphnames

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
