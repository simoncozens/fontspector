#![deny(clippy::unwrap_used, clippy::expect_used)]
pub mod checks;

use fontspector_checkapi::{Profile, Registry};

pub struct Universal;

impl fontspector_checkapi::Plugin for Universal {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        cr.register_check(checks::arabic_high_hamza::arabic_high_hamza);
        cr.register_check(checks::arabic_spacing_symbols::arabic_spacing_symbols);
        cr.register_check(checks::base_has_width::base_has_width);
        cr.register_check(checks::cjk_not_enough_glyphs::cjk_not_enough_glyphs);
        cr.register_check(checks::cjk_chws_feature::cjk_chws_feature);
        cr.register_check(checks::case_mapping::case_mapping);
        cr.register_check(checks::cmap_format_12::cmap_format_12);
        cr.register_check(checks::color_cpal_brightness::color_cpal_brightness);
        cr.register_check(checks::consistent_axes::consistent_axes);
        cr.register_check(checks::contour_count::contour_count);
        cr.register_check(checks::control_chars::control_chars);
        cr.register_check(checks::empty_letters::empty_letters);
        cr.register_check(checks::empty_glyph_on_gid1_for_colrv0::empty_glyph_on_gid1_for_colrv0);
        cr.register_check(checks::family_vertical_metrics::family_vertical_metrics);
        cr.register_check(checks::family_win_ascent_and_descent::family_win_ascent_and_descent);
        cr.register_check(checks::file_size::file_size);
        cr.register_check(checks::fvar_name_entries::fvar_name_entries);
        #[cfg(not(target_family = "wasm"))]
        cr.register_check(checks::fontdata_namecheck::fontdata_namecheck);
        #[cfg(not(target_family = "wasm"))]
        cr.register_check(checks::freetype_rasterizer::freetype_rasterizer);
        cr.register_check(checks::glyphnames::valid_glyphnames);
        cr.register_check(checks::glyphset::check_rupee);
        cr.register_check(checks::gpos7::gpos7);
        cr.register_check(checks::gpos_kerning_info::gpos_kerning_info);
        cr.register_check(checks::hinting_impact::hinting_impact);
        cr.register_check(checks::integer_ppem_if_hinted::integer_ppem_if_hinted);
        cr.register_check(checks::interpolation_issues::interpolation_issues);
        cr.register_check(checks::legacy_accents::legacy_accents);
        cr.register_check(checks::ligature_carets::ligature_carets);
        cr.register_check(checks::linegaps::linegaps);
        cr.register_check(checks::math_signs_width::math_signs_width);
        cr.register_check(checks::name_family_and_style_max::family_and_style_max_length);
        cr.register_check(checks::name_italic_names::name_italic_names);
        cr.register_check(
            checks::name_no_copyright_on_description::name_no_copyright_on_description,
        );
        cr.register_check(checks::nested_components::nested_components);
        cr.register_check(checks::mandatory_avar_table::mandatory_avar_table);
        cr.register_check(checks::mandatory_glyphs::mandatory_glyphs);
        cr.register_check(checks::missing_small_caps_glyphs::missing_small_caps_glyphs);
        cr.register_check(checks::name_char_restrictions::name_char_restrictions);
        cr.register_check(checks::name_trailing_spaces::name_trailing_spaces);
        cr.register_check(checks::no_mac_entries::no_mac_entries);
        cr.register_check(checks::os2_metrics_match_hhea::os2_metrics_match_hhea);
        cr.register_check(checks::required_tables::required_tables);
        cr.register_check(checks::sfnt_version::sfnt_version);
        cr.register_check(checks::smallcaps_before_ligatures::smallcaps_before_ligatures);
        cr.register_check(checks::smart_dropout::smart_dropout);
        cr.register_check(checks::soft_hyphen::soft_hyphen);
        cr.register_check(checks::stat_in_statics::stat_in_statics);
        cr.register_check(checks::stat_strings::stat_strings);
        cr.register_check(checks::stylistic_sets::stylisticset_description);
        cr.register_check(checks::tabular_kerning::tabular_kerning);
        cr.register_check(checks::transformed_components::transformed_components);
        cr.register_check(checks::typoascender_agrave::typoascender_exceeds_agrave);
        cr.register_check(checks::typographic_family_name::typographic_family_name);
        // This check is unused, but we keep it registered in case some other profile
        // specifically wants it rather than valid_glyphnames.
        cr.register_check(checks::unique_glyphnames::unique_glyphnames);
        cr.register_check(checks::unreachable_glyphs::unreachable_glyphs);
        cr.register_check(checks::unsupported_axes::unsupported_axes);
        cr.register_check(checks::unwanted_aat_tables::unwanted_aat_tables);
        cr.register_check(checks::unwanted_tables::unwanted_tables);
        cr.register_check(checks::varfont_duplexed_axis_reflow::varfont_duplexed_axis_reflow);
        cr.register_check(checks::varfont_instances_in_order::varfont_instances_in_order);
        cr.register_check(checks::vtt_volt_data::vtt_volt_data);
        cr.register_check(checks::whitespace_ink::whitespace_ink);
        cr.register_check(checks::whitespace_glyphs::whitespace_glyphs);
        cr.register_check(checks::whitespace_widths::whitespace_widths);

        let universal_profile = Profile::from_toml(
            r#"
include_profiles = ["opentype"]
[sections]

# Superfamilies are kind of dead with the advent of VFs
#"Superfamily Checks" = [
#    "superfamily/list",
#    "superfamily/vertical_metrics",
#]

# Source checks are a good idea and we can do them with norad, but let's hold them
# over for another version.
#"UFO Sources" = [
#    "designspace_has_consistent_codepoints",
#    "designspace_has_consistent_glyphset",
#    "designspace_has_consistent_groups",
#    "designspace_has_default_master",
#    "designspace_has_sources",
#    "ufolint",
#    # "ufo_consistent_curve_type",  # FIXME (orphan check) https://github.com/fonttools/fontbakery/pull/4809
#    "ufo_features_default_languagesystem",
#    # "ufo_no_open_corners",  # FIXME (orphan check) https://github.com/fonttools/fontbakery/pull/4809
#    "ufo_recommended_fields",
#    "ufo_required_fields",
#    "ufo_unnecessary_fields",
#]

"Universal Profile Checks" = [
    # Checks which we have definitely ported already
    "alt_caron",
    "arabic_high_hamza",
    "arabic_spacing_symbols",
    "base_has_width",
    "case_mapping",
    "cjk_chws_feature",
    "cjk_not_enough_glyphs",
    "cmap/format_12",
    "color_cpal_brightness",
    "contour_count",
    "control_chars",
    "empty_glyph_on_gid1_for_colrv0",
    "empty_letters",
    "family/vertical_metrics",
    "family/win_ascent_and_descent",
    "file_size",
    "fontdata_namecheck",
    "freetype_rasterizer",
    "fvar_name_entries",
    "gpos7",
    "gpos_kerning_info",
    "hinting_impact",
    "integer_ppem_if_hinted",
    "interpolation_issues",
    "legacy_accents",
    "ligature_carets",
    "linegaps",
    "mandatory_avar_table",
    "mandatory_glyphs",
    "math_signs_width",
    "missing_small_caps_glyphs",
    "name/char_restrictions",
    "name/family_and_style_max_length",
    "name/no_copyright_on_description",
    "name/trailing_spaces",
    "nested_components",
    "no_mac_entries",
    "os2_metrics_match_hhea",
    "required_tables",
    "rupee",
    "sfnt_version",
    "smallcaps_before_ligatures",
    "smart_dropout",
    "soft_hyphen",
    "STAT_in_statics",
    "STAT_strings",
    "stylisticset_description",
    "tabular_kerning",
    "transformed_components",
    "typoascender_exceeds_Agrave",
    "typographic_family_name",
    "unreachable_glyphs",
    "unwanted_aat_tables",
    "unwanted_tables",
    "valid_glyphnames",
    "varfont/consistent_axes",
    "varfont/duplexed_axis_reflow",
    "varfont/instances_in_order",
    "varfont/unsupported_axes",
    "vtt_volt_data",
    "whitespace_glyphs",
    "whitespace_ink",
    "whitespace_widths",

    # Checks which don't make sense any more
    # "family/single_directory", # Fontspector assumes families are in a directory
    # "ots", # ots checks need to be directly integrated
    # "ttx_roundtrip", # What's ttx? :-)
    # "vttclean", # merged into unwanted_tables
    # "fontspector_version", # we'll just do this once at the start of the program, doesn't make sense for web
    # "kerning_for_non_ligated_sequences", # I just think this is a bad check
    # "unique_glyphnames", # valid_glyphnames also checks for uniqueness
    # "name/italic_names", # GF-specific: https://github.com/fonttools/fontbakery/issues/4971

    # Checks left to port
    # "caps_vertically_centered",  # Disabled: issue #4274
    "inconsistencies_between_fvar_STAT",  # https://github.com/simoncozens/fontspector/issues/30
    "overlapping_path_segments",
    "varfont/bold_wght_coord",
    "varfont/duplicate_instance_names",
]

[configuration_defaults.file_size]
WARN_SIZE = 1048576 # 1Mb
FAIL_SIZE = 9437184 # 9Mb
"#,
        )
        .map_err(|_| "Couldn't parse profile")?;
        cr.register_profile("universal", universal_profile)
    }
}
