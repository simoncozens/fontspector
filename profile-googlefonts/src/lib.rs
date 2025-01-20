#![deny(clippy::unwrap_used, clippy::expect_used)]
mod checks;
pub mod constants;
mod description;
mod family;
mod license;
mod metadata;
mod metadata_copyright;
mod metadata_license;
mod metadata_subsets_correct;
mod use_typo_metrics;
use fontspector_checkapi::prelude::*;

mod network_conditions;

pub struct GoogleFonts;
impl fontspector_checkapi::Plugin for GoogleFonts {
    fn register(&self, cr: &mut Registry) -> Result<(), String> {
        let mdpb = FileType::new("METADATA.pb");
        let desc = FileType::new("DESCRIPTION.en_us.html");
        cr.register_filetype("MDPB", mdpb);
        cr.register_filetype("DESC", desc);
        cr.register_check(checks::axes_match::axes_match);
        cr.register_check(checks::color_fonts::color_fonts);
        cr.register_check(checks::fstype::googlefonts_fstype);
        cr.register_check(checks::googlefonts_weightclass::googlefonts_weightclass);
        cr.register_check(checks::name_description_max_length::name_description_max_length);
        cr.register_check(checks::render_own_name::render_own_name);
        cr.register_check(checks::tofu::googlefonts_tofu);
        cr.register_check(description::description_min_length);
        cr.register_check(description::description_eof_linebreak);
        cr.register_check(family::family_equal_codepoint_coverage);
        cr.register_check(license::name_rfn);
        cr.register_check(metadata::validate_metadatapb);
        cr.register_check(metadata::can_render_samples);
        cr.register_check(metadata_subsets_correct::metadata_subsets_correct);
        cr.register_check(metadata_copyright::metadata_copyright);
        cr.register_check(metadata_license::metadata_license);
        cr.register_check(use_typo_metrics::os2_fsselectionbit7);
        let profile = Profile::from_toml(
            r#"
include_profiles = ["universal"]
[sections]
"Article Checks" = [
    "googlefonts/article/images",
]
"Metadata Checks" = [
    "googlefonts/metadata/axisregistry_bounds",
    "googlefonts/metadata/axisregistry_valid_tags",
    "googlefonts/metadata/broken_links",
    "googlefonts/metadata/canonical_style_names",
    "googlefonts/metadata/canonical_weight_value",
    "googlefonts/metadata/can_render_samples",
    "googlefonts/metadata/category",
    "googlefonts/metadata/category_hints",
    "googlefonts/metadata/consistent_axis_enumeration",
    "googlefonts/metadata/consistent_repo_urls",
    "googlefonts/metadata/designer_profiles",
    "googlefonts/metadata/empty_designer",
    "googlefonts/metadata/escaped_strings",
    "googlefonts/metadata/family_directory_name",
    "googlefonts/metadata/familyname",
    "googlefonts/metadata/filenames",
    "googlefonts/metadata/has_regular",
    "googlefonts/metadata/includes_production_subsets",
    "googlefonts/metadata/match_filename_postscript",
    "googlefonts/metadata/match_fullname_postscript",
    "googlefonts/metadata/match_name_familyname",
    "googlefonts/metadata/match_weight_postscript",
    "googlefonts/metadata/minisite_url",
    "googlefonts/metadata/nameid/family_and_full_names",
    "googlefonts/metadata/nameid/font_name",
    "googlefonts/metadata/nameid/post_script_name",
    "googlefonts/metadata/os2_weightclass",
    "googlefonts/metadata/parses",
    "googlefonts/metadata/primary_script",
    "googlefonts/metadata/regular_is_400",
    "googlefonts/metadata/single_cjk_subset", # To merge into metadata/subsets_correct
    "googlefonts/metadata/undeclared_fonts",
    "googlefonts/metadata/unique_full_name_values",
    "googlefonts/metadata/unique_weight_style_pairs",
    "googlefonts/metadata/unreachable_subsetting",
    "googlefonts/metadata/subsets_correct", # Replacement for metadata/unsupported_subsets
    "googlefonts/metadata/valid_filename_values",
    "googlefonts/metadata/valid_full_name_values",
    "googlefonts/metadata/valid_nameid25",
    "googlefonts/metadata/valid_post_script_name_values",
    # "googlefonts/metadata/subsets_order", # Merged into metadata/parses
    # "googlefonts/metadata/menu_and_latin", # Merged into subsets/correct
    # "googlefonts/metadata/designer_values", # Merged into metadata/parses
    # "googlefonts/metadata/date_added", # Merged into metadata/parses
]
"Glyphset Checks" = [
    "googlefonts/glyphsets/shape_languages",
    "googlefonts/tofu",
]
"Description Checks" = [
    "googlefonts/description/broken_links",
    "googlefonts/description/eof_linebreak",
    "googlefonts/description/family_update",
    "googlefonts/description/git_url",
    "googlefonts/description/has_article",
    "googlefonts/description/has_unsupported_elements",
    "googlefonts/description/min_length",
    "googlefonts/description/urls",
    "googlefonts/description/valid_html",
]
"Family Checks" = [
    "googlefonts/family/equal_codepoint_coverage",
    "googlefonts/family/italics_have_roman_counterparts",
    "googlefonts/family/tnum_horizontal_metrics",
]
"Name table checks" = [
    "googlefonts/name/family_name_compliance",
    "googlefonts/name/line_breaks",
]
"Licensing Checks" = [
    "googlefonts/family/has_license",
    "googlefonts/font_copyright",
    "googlefonts/license/OFL_body_text",
    "googlefonts/license/OFL_copyright",
    "googlefonts/metadata/copyright",
    "googlefonts/metadata/license",
    "googlefonts/metadata/reserved_font_name",
    "googlefonts/name/license",
    "googlefonts/name/license_url",
    "googlefonts/name/rfn",
]
"Repository Checks" = [
    "googlefonts/repo/dirname_matches_nameid_1",
    "googlefonts/repo/fb_report",
    "googlefonts/repo/sample_image",
    "googlefonts/repo/upstream_yaml_has_required_fields",
    "googlefonts/repo/vf_has_static_fonts",
    "googlefonts/repo/zip_files",
]
"Shaping Checks" = [
    "dotted_circle",
    "shaping/collides",
    "shaping/forbidden",
    "shaping/regression",
    "soft_dotted",
]
"Outline Checks" = [
    "outline_alignment_miss",
    "outline_colinear_vectors",
    "outline_direction",
    "outline_jaggy_segments",
    "outline_semi_vertical",
    "outline_short_segments",
]
"Font File Checks" =  [
    "googlefonts/axisregistry/fvar_axis_defaults",
    "googlefonts/canonical_filename",
    "googlefonts/cjk_vertical_metrics",
    "googlefonts/cjk_vertical_metrics_regressions",
    "googlefonts/color_fonts",
    "googlefonts/epar",
    "googlefonts/font_names",
    "googlefonts/fstype",
    "googlefonts/fvar_instances",
    "googlefonts/gasp",
    "googlefonts/glyph_coverage",
    "googlefonts/has_ttfautohint_params",
    "googlefonts/meta/script_lang_tags",
    "googlefonts/name/description_max_length",
    "googlefonts/name/familyname_first_char",
    "googlefonts/name/mandatory_entries",
    "googlefonts/name/version_format",
    "googlefonts/old_ttfautohint",
    "googlefonts/os2/use_typo_metrics",
    "googlefonts/production_glyphs_similarity",
    # "googlefonts/production_encoded_glyphs",  # DISABLED
    "googlefonts/STAT",
    "googlefonts/STAT/axis_order",
    "googlefonts/STAT/axisregistry",
    "googlefonts/unitsperem",
    "googlefonts/weightclass",
    "googlefonts/varfont/bold_wght_coord",
    "googlefonts/varfont/duplicate_instance_names",
    "googlefonts/varfont/generate_static",
    "googlefonts/varfont/has_HVAR",
    "googlefonts/vendor_id",
    "googlefonts/version_bump",
    "googlefonts/vertical_metrics",
    "googlefonts/vertical_metrics_regressions",
    "googlefonts/axes_match",
]

[configuration_defaults.file_size]
WARN_SIZE = 1048576 # 1Mb
FAIL_SIZE = 9437184 # 9Mb
"#,
        )
        .map_err(|_| "Couldn't parse profile")?;

        cr.register_profile("googlefonts", profile)
    }
}
