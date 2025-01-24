#![deny(clippy::unwrap_used, clippy::expect_used)]
pub mod checks;

use fontspector_checkapi::{ProfileBuilder, Registry};
use serde_json::json;
use std::collections::HashMap;

pub struct Universal;

impl fontspector_checkapi::Plugin for Universal {
    fn register(&self, cr: &mut Registry<'_>) -> Result<(), String> {
        let builder = ProfileBuilder::new()
            .include_profile("opentype")
            .add_section("Universal Profile Checks")
            .add_and_register_check(checks::alt_caron)
            .add_and_register_check(checks::arabic_high_hamza)
            .add_and_register_check(checks::arabic_spacing_symbols)
            .add_and_register_check(checks::base_has_width)
            .add_and_register_check(checks::case_mapping)
            .add_and_register_check(checks::cjk_chws_feature)
            .add_and_register_check(checks::cjk_not_enough_glyphs)
            .add_and_register_check(checks::cmap_format_12)
            .add_and_register_check(checks::color_cpal_brightness)
            .add_and_register_check(checks::contour_count)
            .add_and_register_check(checks::control_chars)
            .add_and_register_check(checks::empty_glyph_on_gid1_for_colrv0)
            .add_and_register_check(checks::empty_letters)
            .add_and_register_check(checks::family_vertical_metrics)
            .add_and_register_check(checks::family_win_ascent_and_descent)
            .add_and_register_check(checks::file_size)
            .with_configuration_defaults(
                "file_size",
                HashMap::from([
                    ("WARN_SIZE".to_string(), json!(1048576)), // 1Mb
                    ("FAIL_SIZE".to_string(), json!(9437184)), // 9Mb
                ]),
            );

        #[cfg(not(target_family = "wasm"))]
        let builder = builder
            .add_and_register_check(checks::fontdata_namecheck)
            .add_and_register_check(checks::freetype_rasterizer);

        builder
            .add_and_register_check(checks::fvar_name_entries)
            .add_and_register_check(checks::gpos7)
            .add_and_register_check(checks::gpos_kerning_info)
            .add_and_register_check(checks::hinting_impact)
            .add_and_register_check(checks::integer_ppem_if_hinted)
            .add_and_register_check(checks::interpolation_issues)
            .add_and_register_check(checks::legacy_accents)
            .add_and_register_check(checks::ligature_carets)
            .add_and_register_check(checks::linegaps)
            .add_and_register_check(checks::mandatory_avar_table)
            .add_and_register_check(checks::mandatory_glyphs)
            .add_and_register_check(checks::math_signs_width)
            .add_and_register_check(checks::missing_small_caps_glyphs)
            .add_and_register_check(checks::name::char_restrictions)
            .add_and_register_check(checks::name::family_and_style_max_length)
            .add_and_register_check(checks::name::no_copyright_on_description)
            .add_and_register_check(checks::name::italic_names)
            .add_and_register_check(checks::name::trailing_spaces)
            .add_and_register_check(checks::nested_components)
            .add_and_register_check(checks::no_mac_entries)
            .add_and_register_check(checks::os2_metrics_match_hhea)
            .add_and_register_check(checks::required_tables)
            .add_and_register_check(checks::rupee)
            .add_and_register_check(checks::sfnt_version)
            .add_and_register_check(checks::smallcaps_before_ligatures)
            .add_and_register_check(checks::smart_dropout)
            .add_and_register_check(checks::soft_hyphen)
            .add_and_register_check(checks::STAT_in_statics)
            .add_and_register_check(checks::STAT_strings)
            .add_and_register_check(checks::stylisticset_description)
            .add_and_register_check(checks::tabular_kerning)
            .add_and_register_check(checks::transformed_components)
            .add_and_register_check(checks::typoascender_exceeds_Agrave)
            .add_and_register_check(checks::typographic_family_name)
            .add_and_register_check(checks::unique_glyphnames)
            .add_and_register_check(checks::unreachable_glyphs)
            .add_and_register_check(checks::unwanted_aat_tables)
            .add_and_register_check(checks::unwanted_tables)
            .add_and_register_check(checks::valid_glyphnames)
            .add_and_register_check(checks::consistent_axes)
            .add_and_register_check(checks::varfont_duplexed_axis_reflow)
            .add_and_register_check(checks::varfont_instances_in_order)
            .add_and_register_check(checks::unsupported_axes)
            .add_and_register_check(checks::vtt_volt_data)
            .add_and_register_check(checks::whitespace_glyphs)
            .add_and_register_check(checks::whitespace_ink)
            .add_and_register_check(checks::whitespace_widths)
            .build("universal", cr)

        //  Checks which don't make sense any more
        //  "family/single_directory", # Fontspector assumes families are in a directory
        //  "ots", # ots checks need to be directly integrated
        //  "ttx_roundtrip", # What's ttx? :-)
        //  "fontspector_version", # we'll just do this once at the start of the program, doesn't make sense for web
        //  "kerning_for_non_ligated_sequences", # I just think this is a bad check
        //  "unique_glyphnames", # valid_glyphnames also checks for uniqueness
        //  "name/italic_names", # GF-specific: https://github.com/fonttools/fontbakery/issues/4971

        //  Checks left to port
        //  "caps_vertically_centered",  # Disabled: issue #4274
        //  "inconsistencies_between_fvar_STAT",  # https://github.com/simoncozens/fontspector/issues/30
        //  "overlapping_path_segments", # This is now an outline check, right?
        //  "varfont/bold_wght_coord",
        //   "varfont/duplicate_instance_names",
        // ]
        // # Source checks are a good idea and we can do them with norad, but let's hold them
        // # over for another version.
        // #"UFO Sources" = [
        // #    "designspace_has_consistent_codepoints",
        // #    "designspace_has_consistent_glyphset",
        // #    "designspace_has_consistent_groups",
        // #    "designspace_has_default_master",
        // #    "designspace_has_sources",
        // #    "ufolint",
        // #    # "ufo_consistent_curve_type",  # FIXME (orphan check) https://github.com/fonttools/fontbakery/pull/4809
        // #    "ufo_features_default_languagesystem",
        // #    # "ufo_no_open_corners",  # FIXME (orphan check) https://github.com/fonttools/fontbakery/pull/4809
        // #    "ufo_recommended_fields",
        // #    "ufo_required_fields",
        // #    "ufo_unnecessary_fields",
        // #]
    }
}
